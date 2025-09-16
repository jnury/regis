// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::collections::HashMap;
use tauri::{command, Manager, AppHandle};
use tracing::{debug, info, warn, error, instrument};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};
use tracing_appender::rolling::{RollingFileAppender, Rotation};

// Configuration structures

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoggingConfig {
    pub level: String,
    pub enabled: bool,
    pub console_output: bool,
    pub file_output: bool,
    pub log_directory: String,
    pub max_file_size_mb: u64,
    pub max_files: usize,
    pub log_format: String,
    pub component_levels: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UIConfig {
    pub theme: String,
    pub startup_behavior: String,
    pub minimize_to_tray: bool,
    pub confirm_exit: bool,
    pub remember_window_state: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SecurityConfig {
    pub auto_logout_minutes: u32,
    pub remember_auth: bool,
    pub ssl_verify: bool,
    pub timeout_seconds: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConnectionConfig {
    pub auto_connect_single_target: bool,
    pub connection_timeout_seconds: u32,
    pub retry_attempts: u32,
    pub retry_delay_seconds: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RdpConfig {
    pub auto_launch: bool,
    pub preferred_client: String,
    pub fullscreen: bool,
    pub resolution: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AdvancedConfig {
    pub debug_mode: bool,
    pub developer_tools: bool,
    pub crash_reporting: bool,
    pub telemetry: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub logging: LoggingConfig,
    pub ui: UIConfig,
    pub security: SecurityConfig,
    pub connection: ConnectionConfig,
    pub rdp: RdpConfig,
    pub advanced: AdvancedConfig,
}

// Server structures (existing)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Server {
    pub id: String,
    pub name: String,
    pub url: String,
    pub description: String,
    pub environment: String,
    pub region: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub servers: Vec<Server>,
}

// Global state for configuration
pub struct AppState {
    pub config: Config,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            logging: LoggingConfig {
                level: "info".to_string(),
                enabled: true,
                console_output: true,
                file_output: true,
                log_directory: "auto".to_string(),
                max_file_size_mb: 10,
                max_files: 5,
                log_format: "structured".to_string(),
                component_levels: HashMap::new(),
            },
            ui: UIConfig {
                theme: "system".to_string(),
                startup_behavior: "show_window".to_string(),
                minimize_to_tray: true,
                confirm_exit: false,
                remember_window_state: true,
            },
            security: SecurityConfig {
                auto_logout_minutes: 60,
                remember_auth: true,
                ssl_verify: true,
                timeout_seconds: 30,
            },
            connection: ConnectionConfig {
                auto_connect_single_target: true,
                connection_timeout_seconds: 10,
                retry_attempts: 3,
                retry_delay_seconds: 2,
            },
            rdp: RdpConfig {
                auto_launch: true,
                preferred_client: "auto".to_string(),
                fullscreen: false,
                resolution: "auto".to_string(),
            },
            advanced: AdvancedConfig {
                debug_mode: false,
                developer_tools: false,
                crash_reporting: true,
                telemetry: false,
            },
        }
    }
}

// Platform-specific log directory resolution
fn get_log_directory(config_dir: &str) -> Result<PathBuf, String> {
    let log_dir = if config_dir == "auto" {
        if let Some(mut dir) = dirs::data_local_dir() {
            // Windows: C:\Users\{user}\AppData\Local\Regis\logs
            dir.push("Regis");
            dir.push("logs");
            dir
        } else if let Some(mut dir) = dirs::home_dir() {
            // macOS/Linux: ~/.regis/logs
            dir.push(".regis");
            dir.push("logs");
            dir
        } else {
            // Fallback to temp directory
            let mut temp_dir = std::env::temp_dir();
            temp_dir.push("regis");
            temp_dir.push("logs");
            temp_dir
        }
    } else {
        PathBuf::from(config_dir)
    };

    // Create directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(&log_dir) {
        return Err(format!("Failed to create log directory {:?}: {}", log_dir, e));
    }

    Ok(log_dir)
}

// Initialize logging system
#[instrument]
fn init_logging(config: &LoggingConfig) -> Result<(), String> {
    if !config.enabled {
        println!("Logging disabled by configuration");
        return Ok(());
    }

    let filter = match config.level.as_str() {
        "debug" => EnvFilter::new("debug"),
        "info" => EnvFilter::new("info"),
        "warn" => EnvFilter::new("warn"),
        "error" => EnvFilter::new("error"),
        _ => EnvFilter::new("info"),
    };

    let registry = tracing_subscriber::registry().with(filter);

    // Build the subscriber with optional layers
    if config.console_output && config.file_output {
        // Both console and file logging
        let log_dir = get_log_directory(&config.log_directory)?;
        let file_appender = RollingFileAppender::new(Rotation::DAILY, &log_dir, "regis.log");

        let console_layer = fmt::layer()
            .with_target(true)
            .with_thread_ids(true)
            .with_level(true)
            .with_ansi(true);

        let file_layer = fmt::layer()
            .with_writer(file_appender)
            .with_target(true)
            .with_thread_ids(false)
            .with_level(true)
            .with_ansi(false)
            .compact();

        registry.with(console_layer).with(file_layer).init();
        info!("File logging initialized in: {:?}", log_dir);
    } else if config.console_output {
        // Console only
        let console_layer = fmt::layer()
            .with_target(true)
            .with_thread_ids(true)
            .with_level(true)
            .with_ansi(true);

        registry.with(console_layer).init();
    } else if config.file_output {
        // File only
        let log_dir = get_log_directory(&config.log_directory)?;
        let file_appender = RollingFileAppender::new(Rotation::DAILY, &log_dir, "regis.log");

        let file_layer = fmt::layer()
            .with_writer(file_appender)
            .with_target(true)
            .with_thread_ids(false)
            .with_level(true)
            .with_ansi(false)
            .compact();

        registry.with(file_layer).init();
        info!("File logging initialized in: {:?}", log_dir);
    } else {
        // No logging layers, just the registry
        registry.init();
    }

    info!("Logging system initialized");
    info!("Log level: {}", config.level);
    info!("Console output: {}", config.console_output);
    info!("File output: {}", config.file_output);

    Ok(())
}

// Load configuration with resource fallback system
#[instrument(skip(app))]
fn load_resource_with_fallback(app: &AppHandle, filename: &str) -> Result<String, String> {
    debug!("Loading resource: {}", filename);

    // Try to resolve the resource using Tauri's resource resolver
    let resource_result = app
        .path()
        .resolve(filename, tauri::path::BaseDirectory::Resource)
        .and_then(|path| {
            debug!("Resolved resource path: {:?}", path);
            fs::read_to_string(&path).map_err(|e| {
                warn!("Failed to read resolved path {:?}: {}", path, e);
                tauri::Error::Io(e)
            })
        });

    match resource_result {
        Ok(content) => {
            info!("Successfully loaded {} from resources", filename);
            Ok(content)
        }
        Err(e) => {
            warn!("Failed to load {} from resources: {:?}", filename, e);

            // Fallback: try multiple paths for development
            let mut fallback_content = None;
            let mut last_error = String::new();

            // Try: resource directory (direct path)
            if let Ok(resource_dir) = app.path().resource_dir() {
                let resource_path = resource_dir.join(filename);
                debug!("Trying resource directory path: {:?}", resource_path);

                match fs::read_to_string(&resource_path) {
                    Ok(content) => {
                        fallback_content = Some(content);
                        info!("Successfully read {} from resource directory", filename);
                    }
                    Err(e) => {
                        last_error = format!("Resource dir failed: {}", e);
                        debug!("Resource dir failed: {}", e);

                        // Try the _up_ subdirectory where resources are actually bundled
                        let resource_up_path = resource_dir.join("_up_").join(filename);
                        debug!("Trying resource _up_ directory path: {:?}", resource_up_path);

                        match fs::read_to_string(&resource_up_path) {
                            Ok(content) => {
                                fallback_content = Some(content);
                                info!("Successfully read {} from resource _up_ directory", filename);
                            }
                            Err(e2) => {
                                last_error = format!("{} | Resource _up_ dir failed: {}", last_error, e2);
                                debug!("Resource _up_ dir failed: {}", e2);
                            }
                        }
                    }
                }
            }

            // Try: relative to project root (development)
            if fallback_content.is_none() {
                let project_path = std::path::Path::new("../").join(filename);
                debug!("Trying project path: {:?}", project_path);

                match fs::read_to_string(&project_path) {
                    Ok(content) => {
                        fallback_content = Some(content);
                        info!("Successfully read {} from project directory", filename);
                    }
                    Err(e) => {
                        last_error = format!("{} | Project dir failed: {}", last_error, e);
                        debug!("Project dir failed: {}", e);
                    }
                }
            }

            // Try: current directory
            if fallback_content.is_none() {
                let current_path = std::path::Path::new(filename);
                debug!("Trying current path: {:?}", current_path);

                match fs::read_to_string(current_path) {
                    Ok(content) => {
                        fallback_content = Some(content);
                        info!("Successfully read {} from current directory", filename);
                    }
                    Err(e) => {
                        last_error = format!("{} | Current dir failed: {}", last_error, e);
                        debug!("Current dir failed: {}", e);
                    }
                }
            }

            fallback_content.ok_or_else(|| {
                let error_msg = format!(
                    "Failed to find {} in any location. Resource error: {:?}. Fallback errors: {}",
                    filename, e, last_error
                );
                error!("{}", error_msg);
                error_msg
            })
        }
    }
}

// Load configuration
#[instrument(skip(app))]
fn load_config(app: &AppHandle) -> Config {
    debug!("Loading application configuration");

    match load_resource_with_fallback(app, "config.json") {
        Ok(config_content) => {
            match serde_json::from_str::<Config>(&config_content) {
                Ok(config) => {
                    info!("Successfully loaded and parsed config.json");
                    debug!("Config: {:?}", config);
                    config
                }
                Err(e) => {
                    error!("Failed to parse config.json: {}. Using default configuration.", e);
                    Config::default()
                }
            }
        }
        Err(e) => {
            error!("Failed to load config.json: {}. Using default configuration.", e);
            Config::default()
        }
    }
}

// Tauri commands
#[command]
#[instrument(skip(app))]
async fn load_servers(app: AppHandle) -> Result<Vec<Server>, String> {
    info!("Loading servers from servers.json");

    let config_content = load_resource_with_fallback(&app, "servers.json")?;

    // Parse the JSON content
    let config: ServerConfig = serde_json::from_str(&config_content)
        .map_err(|e| {
            let error_msg = format!("Failed to parse servers.json: {}", e);
            error!("{}", error_msg);
            error_msg
        })?;

    info!("Successfully loaded {} servers", config.servers.len());
    debug!("Servers: {:?}", config.servers);

    Ok(config.servers)
}

#[command]
#[instrument]
async fn log_from_frontend(level: String, component: String, message: String, data: Option<String>) -> Result<(), String> {
    let log_data = data.unwrap_or_default();

    match level.as_str() {
        "debug" => debug!(component = %component, data = %log_data, "{}", message),
        "info" => info!(component = %component, data = %log_data, "{}", message),
        "warn" => warn!(component = %component, data = %log_data, "{}", message),
        "error" => error!(component = %component, data = %log_data, "{}", message),
        _ => info!(component = %component, data = %log_data, "{}", message),
    }

    Ok(())
}

#[command]
#[instrument(skip(app))]
async fn get_config(app: AppHandle) -> Result<Config, String> {
    debug!("Frontend requested application configuration");

    let state = app.state::<AppState>();
    debug!("Returning configuration to frontend");
    Ok(state.config.clone())
}

fn main() {
    // Early logging setup with minimal configuration
    println!("Starting Regis application...");

    tauri::Builder::default()
        .setup(|app| {
            println!("Initializing application...");

            // Load configuration
            let config = load_config(app.handle());

            // Initialize logging system
            if let Err(e) = init_logging(&config.logging) {
                eprintln!("Failed to initialize logging: {}", e);
                println!("Continuing with console logging only");
            }

            // Log startup information
            info!("=== Regis Application Starting ===");
            info!("Version: {}", app.package_info().version);
            info!("Debug mode: {}", config.advanced.debug_mode);

            // Store configuration in app state
            app.manage(AppState { config });

            info!("Application initialization completed successfully");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            load_servers,
            log_from_frontend,
            get_config
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}