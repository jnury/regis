// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::collections::HashMap;
use std::process::Stdio;
use tauri::{command, Manager, AppHandle};
use tracing::{debug, info, warn, error, instrument};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use tokio::process::Command;
use regex::Regex;
use std::sync::{Arc, Mutex};
use chrono;
use keyring::Entry;
use reqwest;
use url::Url;

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
pub struct BoundaryConfig {
    pub cli_path: String,
    pub auto_detect: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub logging: LoggingConfig,
    pub ui: UIConfig,
    pub security: SecurityConfig,
    pub connection: ConnectionConfig,
    pub rdp: RdpConfig,
    pub advanced: AdvancedConfig,
    pub boundary: BoundaryConfig,
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
    pub boundary_cli_path: Option<String>, // Optional per-server CLI path override
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub servers: Vec<Server>,
}

// Boundary CLI execution structures
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BoundaryCommandResult {
    pub success: bool,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub command: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BoundaryAuthMethod {
    pub id: String,
    pub name: String,
    pub method_type: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BoundaryScope {
    pub id: String,
    pub name: String,
    pub scope_type: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BoundaryTarget {
    pub id: String,
    pub name: String,
    pub target_type: String,
    pub description: String,
    pub address: Option<String>,
    pub default_port: Option<u16>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BoundarySessionAuthorization {
    pub authorization_token: String,
    pub session_id: String,
    pub target_id: String,
    pub user_id: String,
    pub host_id: Option<String>,
    pub scope_id: String,
    pub created_time: String,
    pub expiration_time: Option<String>,
    pub connection_limit: i32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BoundaryConnection {
    pub session_id: String,
    pub target_id: String,
    pub target_name: String,
    pub connection_type: String,
    pub local_address: String,
    pub local_port: u16,
    pub status: String,
    pub created_time: String,
    pub expiration_time: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ConnectionType {
    SSH,
    RDP,
    TCP,
    HTTP,
}

// RDP client detection structures
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RdpClientInfo {
    pub name: String,
    pub executable_path: String,
    pub client_type: String,
    pub platform: String,
    pub version: Option<String>,
    pub supports_fullscreen: bool,
    pub supports_resolution: bool,
    pub supports_credentials: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DetectedRdpClients {
    pub clients: Vec<RdpClientInfo>,
    pub default_client: Option<String>,
    pub platform: String,
}

// Token storage structures
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StoredToken {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<String>,
    pub server_id: String,
    pub user_id: String,
    pub scope_id: String,
    pub created_at: String,
}

// OIDC Authentication Structures
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OIDCAuthRequest {
    pub server_id: String,
    pub auth_method_id: String,
    pub scope_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OIDCAuthProgress {
    pub status: String, // "started", "waiting_for_browser", "completing", "completed", "failed"
    pub message: String,
    pub auth_url: Option<String>,
    pub progress_percent: u8,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OIDCAuthResult {
    pub success: bool,
    pub token: Option<StoredToken>,
    pub error: Option<String>,
    pub scopes: Option<Vec<BoundaryScope>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenMetadata {
    pub server_id: String,
    pub user_id: String,
    pub scope_id: String,
    pub created_at: String,
    pub expires_at: Option<String>,
}

// Session monitoring structures
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionHealth {
    pub session_id: String,
    pub status: String,
    pub last_check: String,
    pub response_time_ms: Option<u64>,
    pub error_count: u32,
    pub consecutive_failures: u32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionMonitoringStats {
    pub total_sessions: usize,
    pub active_sessions: usize,
    pub failed_sessions: usize,
    pub monitoring_enabled: bool,
    pub last_check: String,
}

// Global state for configuration and connections
pub struct AppState {
    pub config: Config,
    pub active_connections: Arc<Mutex<Vec<BoundaryConnection>>>,
    pub session_health: Arc<Mutex<HashMap<String, SessionHealth>>>,
    pub monitoring_enabled: Arc<Mutex<bool>>,
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("config", &"Config{...}")
            .field("active_connections", &"Arc<Mutex<Vec<BoundaryConnection>>>")
            .field("session_health", &"Arc<Mutex<HashMap<String, SessionHealth>>>")
            .field("monitoring_enabled", &"Arc<Mutex<bool>>")
            .finish()
    }
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
            boundary: BoundaryConfig {
                cli_path: "boundary-cli/boundary_0.19.3_darwin_arm64/boundary".to_string(),
                auto_detect: false,
            },
        }
    }
}

impl Config {
    // Merge user config over system config
    fn merge_with_user_config(&mut self, user_config: Config) {
        debug!("Merging user configuration overrides");

        // Merge logging config
        if user_config.logging.level != self.logging.level {
            debug!("User override: logging.level = {}", user_config.logging.level);
            self.logging.level = user_config.logging.level;
        }
        if user_config.logging.enabled != self.logging.enabled {
            debug!("User override: logging.enabled = {}", user_config.logging.enabled);
            self.logging.enabled = user_config.logging.enabled;
        }
        if user_config.logging.console_output != self.logging.console_output {
            debug!("User override: logging.console_output = {}", user_config.logging.console_output);
            self.logging.console_output = user_config.logging.console_output;
        }
        if user_config.logging.file_output != self.logging.file_output {
            debug!("User override: logging.file_output = {}", user_config.logging.file_output);
            self.logging.file_output = user_config.logging.file_output;
        }
        if user_config.logging.log_directory != self.logging.log_directory {
            debug!("User override: logging.log_directory = {}", user_config.logging.log_directory);
            self.logging.log_directory = user_config.logging.log_directory;
        }
        if user_config.logging.max_file_size_mb != self.logging.max_file_size_mb {
            debug!("User override: logging.max_file_size_mb = {}", user_config.logging.max_file_size_mb);
            self.logging.max_file_size_mb = user_config.logging.max_file_size_mb;
        }
        if user_config.logging.max_files != self.logging.max_files {
            debug!("User override: logging.max_files = {}", user_config.logging.max_files);
            self.logging.max_files = user_config.logging.max_files;
        }
        if user_config.logging.log_format != self.logging.log_format {
            debug!("User override: logging.log_format = {}", user_config.logging.log_format);
            self.logging.log_format = user_config.logging.log_format;
        }
        // Merge component_levels (extend the HashMap)
        for (component, level) in user_config.logging.component_levels {
            debug!("User override: logging.component_levels.{} = {}", component, level);
            self.logging.component_levels.insert(component, level);
        }

        // Merge UI config
        if user_config.ui.theme != self.ui.theme {
            debug!("User override: ui.theme = {}", user_config.ui.theme);
            self.ui.theme = user_config.ui.theme;
        }
        if user_config.ui.startup_behavior != self.ui.startup_behavior {
            debug!("User override: ui.startup_behavior = {}", user_config.ui.startup_behavior);
            self.ui.startup_behavior = user_config.ui.startup_behavior;
        }
        if user_config.ui.minimize_to_tray != self.ui.minimize_to_tray {
            debug!("User override: ui.minimize_to_tray = {}", user_config.ui.minimize_to_tray);
            self.ui.minimize_to_tray = user_config.ui.minimize_to_tray;
        }
        if user_config.ui.confirm_exit != self.ui.confirm_exit {
            debug!("User override: ui.confirm_exit = {}", user_config.ui.confirm_exit);
            self.ui.confirm_exit = user_config.ui.confirm_exit;
        }
        if user_config.ui.remember_window_state != self.ui.remember_window_state {
            debug!("User override: ui.remember_window_state = {}", user_config.ui.remember_window_state);
            self.ui.remember_window_state = user_config.ui.remember_window_state;
        }

        // Merge security config
        if user_config.security.auto_logout_minutes != self.security.auto_logout_minutes {
            debug!("User override: security.auto_logout_minutes = {}", user_config.security.auto_logout_minutes);
            self.security.auto_logout_minutes = user_config.security.auto_logout_minutes;
        }
        if user_config.security.remember_auth != self.security.remember_auth {
            debug!("User override: security.remember_auth = {}", user_config.security.remember_auth);
            self.security.remember_auth = user_config.security.remember_auth;
        }
        if user_config.security.ssl_verify != self.security.ssl_verify {
            debug!("User override: security.ssl_verify = {}", user_config.security.ssl_verify);
            self.security.ssl_verify = user_config.security.ssl_verify;
        }
        if user_config.security.timeout_seconds != self.security.timeout_seconds {
            debug!("User override: security.timeout_seconds = {}", user_config.security.timeout_seconds);
            self.security.timeout_seconds = user_config.security.timeout_seconds;
        }

        // Merge connection config
        if user_config.connection.auto_connect_single_target != self.connection.auto_connect_single_target {
            debug!("User override: connection.auto_connect_single_target = {}", user_config.connection.auto_connect_single_target);
            self.connection.auto_connect_single_target = user_config.connection.auto_connect_single_target;
        }
        if user_config.connection.connection_timeout_seconds != self.connection.connection_timeout_seconds {
            debug!("User override: connection.connection_timeout_seconds = {}", user_config.connection.connection_timeout_seconds);
            self.connection.connection_timeout_seconds = user_config.connection.connection_timeout_seconds;
        }
        if user_config.connection.retry_attempts != self.connection.retry_attempts {
            debug!("User override: connection.retry_attempts = {}", user_config.connection.retry_attempts);
            self.connection.retry_attempts = user_config.connection.retry_attempts;
        }
        if user_config.connection.retry_delay_seconds != self.connection.retry_delay_seconds {
            debug!("User override: connection.retry_delay_seconds = {}", user_config.connection.retry_delay_seconds);
            self.connection.retry_delay_seconds = user_config.connection.retry_delay_seconds;
        }

        // Merge RDP config
        if user_config.rdp.auto_launch != self.rdp.auto_launch {
            debug!("User override: rdp.auto_launch = {}", user_config.rdp.auto_launch);
            self.rdp.auto_launch = user_config.rdp.auto_launch;
        }
        if user_config.rdp.preferred_client != self.rdp.preferred_client {
            debug!("User override: rdp.preferred_client = {}", user_config.rdp.preferred_client);
            self.rdp.preferred_client = user_config.rdp.preferred_client;
        }
        if user_config.rdp.fullscreen != self.rdp.fullscreen {
            debug!("User override: rdp.fullscreen = {}", user_config.rdp.fullscreen);
            self.rdp.fullscreen = user_config.rdp.fullscreen;
        }
        if user_config.rdp.resolution != self.rdp.resolution {
            debug!("User override: rdp.resolution = {}", user_config.rdp.resolution);
            self.rdp.resolution = user_config.rdp.resolution;
        }

        // Merge advanced config
        if user_config.advanced.debug_mode != self.advanced.debug_mode {
            debug!("User override: advanced.debug_mode = {}", user_config.advanced.debug_mode);
            self.advanced.debug_mode = user_config.advanced.debug_mode;
        }
        if user_config.advanced.developer_tools != self.advanced.developer_tools {
            debug!("User override: advanced.developer_tools = {}", user_config.advanced.developer_tools);
            self.advanced.developer_tools = user_config.advanced.developer_tools;
        }
        if user_config.advanced.crash_reporting != self.advanced.crash_reporting {
            debug!("User override: advanced.crash_reporting = {}", user_config.advanced.crash_reporting);
            self.advanced.crash_reporting = user_config.advanced.crash_reporting;
        }
        if user_config.advanced.telemetry != self.advanced.telemetry {
            debug!("User override: advanced.telemetry = {}", user_config.advanced.telemetry);
            self.advanced.telemetry = user_config.advanced.telemetry;
        }

        // Merge boundary config
        if user_config.boundary.cli_path != self.boundary.cli_path {
            debug!("User override: boundary.cli_path = {}", user_config.boundary.cli_path);
            self.boundary.cli_path = user_config.boundary.cli_path;
        }
        if user_config.boundary.auto_detect != self.boundary.auto_detect {
            debug!("User override: boundary.auto_detect = {}", user_config.boundary.auto_detect);
            self.boundary.auto_detect = user_config.boundary.auto_detect;
        }
    }
}

// Platform-specific user profile directory resolution
fn get_user_profile_directory() -> Result<PathBuf, String> {
    let user_dir = if let Some(mut dir) = dirs::home_dir() {
        // All platforms: ~/.regis/
        dir.push(".regis");
        dir
    } else {
        return Err("Failed to determine user home directory".to_string());
    };

    // Create directory if it doesn't exist
    if let Err(e) = fs::create_dir_all(&user_dir) {
        return Err(format!("Failed to create user profile directory {:?}: {}", user_dir, e));
    }

    info!("User profile directory resolved: {:?}", user_dir);
    Ok(user_dir)
}

// Get user configuration file path
fn get_user_config_path() -> Result<PathBuf, String> {
    let mut user_dir = get_user_profile_directory()?;
    user_dir.push("config.json");
    Ok(user_dir)
}

// Get user servers file path
fn get_user_servers_path() -> Result<PathBuf, String> {
    let mut user_dir = get_user_profile_directory()?;
    user_dir.push("servers.json");
    Ok(user_dir)
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

// Resolve the correct Boundary CLI path for a server
fn get_boundary_cli_path(server: &Server, config: &Config) -> String {
    // Use server-specific path if provided, otherwise use global config
    server.boundary_cli_path.clone().unwrap_or_else(|| config.boundary.cli_path.clone())
}

// Execute Boundary CLI command with comprehensive logging
#[instrument]
async fn execute_boundary_command(
    cli_path: &str,
    args: Vec<&str>,
    server_addr: Option<&str>,
) -> Result<BoundaryCommandResult, String> {
    let command_str = format!("{} {}", cli_path, args.join(" "));
    info!("Executing Boundary CLI command: {}", command_str);
    debug!("CLI path: {}", cli_path);
    debug!("Arguments: {:?}", args);

    if let Some(addr) = server_addr {
        debug!("Server address: {}", addr);
    }

    // Build the command
    let mut cmd = Command::new(cli_path);
    cmd.args(&args);

    // Add server address if provided
    if let Some(addr) = server_addr {
        cmd.arg("-addr").arg(addr);
    }

    // Configure stdio
    cmd.stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null());

    // Execute the command
    match cmd.output().await {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            let success = output.status.success();
            let exit_code = output.status.code().unwrap_or(-1);

            let result = BoundaryCommandResult {
                success,
                exit_code,
                stdout: stdout.clone(),
                stderr: stderr.clone(),
                command: command_str.clone(),
            };

            if success {
                info!("Boundary CLI command succeeded with exit code: {}", exit_code);
                debug!("Command output: {}", stdout);
            } else {
                error!("Boundary CLI command failed with exit code: {}", exit_code);
                error!("Command error: {}", stderr);
                debug!("Command output: {}", stdout);
            }

            Ok(result)
        }
        Err(e) => {
            let error_msg = format!("Failed to execute Boundary CLI command '{}': {}", command_str, e);
            error!("{}", error_msg);
            Err(error_msg)
        }
    }
}

// Check if Boundary CLI is available and working
#[instrument]
async fn verify_boundary_cli(cli_path: &str) -> Result<bool, String> {
    info!("Verifying Boundary CLI at path: {}", cli_path);

    match execute_boundary_command(cli_path, vec!["version"], None).await {
        Ok(result) => {
            if result.success {
                info!("Boundary CLI verification successful");
                debug!("Boundary CLI version: {}", result.stdout.trim());
                Ok(true)
            } else {
                let error_msg = format!("Boundary CLI verification failed: {}", result.stderr);
                error!("{}", error_msg);
                Err(error_msg)
            }
        }
        Err(e) => {
            let error_msg = format!("Boundary CLI verification failed: {}", e);
            error!("{}", error_msg);
            Err(error_msg)
        }
    }
}

// Discover available auth methods from a Boundary server
#[instrument]
async fn discover_auth_methods(cli_path: &str, server_addr: &str) -> Result<Vec<BoundaryAuthMethod>, String> {
    info!("Discovering auth methods from server: {}", server_addr);

    let result = execute_boundary_command(
        cli_path,
        vec!["auth-methods", "list", "-format", "json"],
        Some(server_addr),
    ).await?;

    if !result.success {
        let error_msg = format!("Failed to discover auth methods: {}", result.stderr);
        error!("{}", error_msg);
        return Err(error_msg);
    }

    // Parse JSON output
    match serde_json::from_str::<serde_json::Value>(&result.stdout) {
        Ok(json) => {
            let mut auth_methods = Vec::new();

            if let Some(items) = json["items"].as_array() {
                for item in items {
                    let auth_method = BoundaryAuthMethod {
                        id: item["id"].as_str().unwrap_or("").to_string(),
                        name: item["name"].as_str().unwrap_or("").to_string(),
                        method_type: item["type"].as_str().unwrap_or("").to_string(),
                        description: item["description"].as_str().unwrap_or("").to_string(),
                    };
                    auth_methods.push(auth_method);
                }
            }

            info!("Discovered {} auth methods", auth_methods.len());
            debug!("Auth methods: {:?}", auth_methods);
            Ok(auth_methods)
        }
        Err(e) => {
            let error_msg = format!("Failed to parse auth methods JSON: {}", e);
            error!("{}", error_msg);
            Err(error_msg)
        }
    }
}

// Discover available scopes from a Boundary server
#[instrument]
async fn discover_scopes(cli_path: &str, server_addr: &str) -> Result<Vec<BoundaryScope>, String> {
    info!("Discovering scopes from server: {}", server_addr);

    let result = execute_boundary_command(
        cli_path,
        vec!["scopes", "list", "-format", "json"],
        Some(server_addr),
    ).await?;

    if !result.success {
        let error_msg = format!("Failed to discover scopes: {}", result.stderr);
        error!("{}", error_msg);
        return Err(error_msg);
    }

    // Parse JSON output
    match serde_json::from_str::<serde_json::Value>(&result.stdout) {
        Ok(json) => {
            let mut scopes = Vec::new();

            if let Some(items) = json["items"].as_array() {
                for item in items {
                    let scope = BoundaryScope {
                        id: item["id"].as_str().unwrap_or("").to_string(),
                        name: item["name"].as_str().unwrap_or("").to_string(),
                        scope_type: item["type"].as_str().unwrap_or("").to_string(),
                        description: item["description"].as_str().unwrap_or("").to_string(),
                    };
                    scopes.push(scope);
                }
            }

            info!("Discovered {} scopes", scopes.len());
            debug!("Scopes: {:?}", scopes);
            Ok(scopes)
        }
        Err(e) => {
            let error_msg = format!("Failed to parse scopes JSON: {}", e);
            error!("{}", error_msg);
            Err(error_msg)
        }
    }
}

// Discover OIDC-specific authentication capabilities
#[instrument]
async fn discover_oidc_auth_methods(cli_path: &str, server_addr: &str) -> Result<Vec<BoundaryAuthMethod>, String> {
    info!("Discovering OIDC auth methods from server: {}", server_addr);

    // Get all auth methods first
    let all_auth_methods = discover_auth_methods(cli_path, server_addr).await?;

    // Filter for OIDC methods only
    let oidc_methods: Vec<BoundaryAuthMethod> = all_auth_methods
        .into_iter()
        .filter(|method| method.method_type.eq_ignore_ascii_case("oidc"))
        .collect();

    info!("Found {} OIDC auth methods", oidc_methods.len());
    debug!("OIDC auth methods: {:?}", oidc_methods);

    if oidc_methods.is_empty() {
        warn!("No OIDC authentication methods found on server: {}", server_addr);
        return Err("No OIDC authentication methods available on this server".to_string());
    }

    Ok(oidc_methods)
}

// Check if server supports OIDC authentication
#[instrument]
async fn verify_oidc_support(cli_path: &str, server_addr: &str) -> Result<bool, String> {
    info!("Verifying OIDC support for server: {}", server_addr);

    match discover_oidc_auth_methods(cli_path, server_addr).await {
        Ok(oidc_methods) => {
            let supports_oidc = !oidc_methods.is_empty();
            if supports_oidc {
                info!("Server {} supports OIDC authentication with {} method(s)", server_addr, oidc_methods.len());
            } else {
                warn!("Server {} does not support OIDC authentication", server_addr);
            }
            Ok(supports_oidc)
        }
        Err(e) => {
            error!("Failed to verify OIDC support for server {}: {}", server_addr, e);
            Ok(false) // Return false instead of error - server just doesn't support OIDC
        }
    }
}

// Discover available targets from a Boundary server
#[instrument]
async fn discover_targets(cli_path: &str, server_addr: &str, scope_id: Option<&str>) -> Result<Vec<BoundaryTarget>, String> {
    info!("Discovering targets from server: {}", server_addr);

    let mut args = vec!["targets", "list", "-format", "json"];

    // Add scope filter if provided
    if let Some(scope) = scope_id {
        info!("Filtering targets for scope: {}", scope);
        args.push("-scope-id");
        args.push(scope);
    }

    let result = execute_boundary_command(cli_path, args, Some(server_addr)).await?;

    if !result.success {
        let error_msg = format!("Failed to discover targets: {}", result.stderr);
        error!("{}", error_msg);
        return Err(error_msg);
    }

    // Parse JSON output
    match serde_json::from_str::<serde_json::Value>(&result.stdout) {
        Ok(json) => {
            let mut targets = Vec::new();

            if let Some(items) = json["items"].as_array() {
                for item in items {
                    let target = BoundaryTarget {
                        id: item["id"].as_str().unwrap_or("").to_string(),
                        name: item["name"].as_str().unwrap_or("").to_string(),
                        target_type: item["type"].as_str().unwrap_or("").to_string(),
                        description: item["description"].as_str().unwrap_or("").to_string(),
                        address: item["address"].as_str().map(|s| s.to_string()),
                        default_port: item["default_port"].as_u64().map(|p| p as u16),
                    };
                    targets.push(target);
                }
            }

            info!("Discovered {} targets", targets.len());
            debug!("Targets: {:?}", targets);
            Ok(targets)
        }
        Err(e) => {
            let error_msg = format!("Failed to parse targets JSON: {}", e);
            error!("{}", error_msg);
            Err(error_msg)
        }
    }
}

// Discover targets for all available scopes (when scope not specified)
#[instrument]
async fn discover_all_targets(cli_path: &str, server_addr: &str) -> Result<Vec<BoundaryTarget>, String> {
    info!("Discovering targets from all scopes on server: {}", server_addr);

    // First, get all scopes
    let scopes = discover_scopes(cli_path, server_addr).await?;
    let mut all_targets = Vec::new();

    // Collect targets from each scope
    for scope in scopes {
        info!("Discovering targets in scope: {} ({})", scope.name, scope.id);

        match discover_targets(cli_path, server_addr, Some(&scope.id)).await {
            Ok(mut scope_targets) => {
                info!("Found {} targets in scope {}", scope_targets.len(), scope.name);
                all_targets.append(&mut scope_targets);
            }
            Err(e) => {
                warn!("Failed to discover targets in scope {}: {}", scope.name, e);
                // Continue with other scopes instead of failing completely
            }
        }
    }

    info!("Discovered {} total targets across all scopes", all_targets.len());
    debug!("All targets: {:?}", all_targets);
    Ok(all_targets)
}

// Authorize a session for a specific target
#[instrument]
async fn authorize_session(cli_path: &str, server_addr: &str, target_id: &str, host_id: Option<&str>) -> Result<BoundarySessionAuthorization, String> {
    info!("Authorizing session for target: {}", target_id);

    let mut args = vec!["targets", "authorize-session", "-id", target_id, "-format", "json"];

    // Add host ID if specified
    if let Some(host) = host_id {
        info!("Targeting specific host: {}", host);
        args.push("-host-id");
        args.push(host);
    }

    let result = execute_boundary_command(cli_path, args, Some(server_addr)).await?;

    if !result.success {
        let error_msg = format!("Failed to authorize session for target {}: {}", target_id, result.stderr);
        error!("{}", error_msg);
        return Err(error_msg);
    }

    // Parse JSON output
    match serde_json::from_str::<serde_json::Value>(&result.stdout) {
        Ok(json) => {
            let authorization = BoundarySessionAuthorization {
                authorization_token: json["authorization_token"].as_str().unwrap_or("").to_string(),
                session_id: json["session_id"].as_str().unwrap_or("").to_string(),
                target_id: json["target_id"].as_str().unwrap_or("").to_string(),
                user_id: json["user_id"].as_str().unwrap_or("").to_string(),
                host_id: json["host_id"].as_str().map(|s| s.to_string()),
                scope_id: json["scope_id"].as_str().unwrap_or("").to_string(),
                created_time: json["created_time"].as_str().unwrap_or("").to_string(),
                expiration_time: json["expiration_time"].as_str().map(|s| s.to_string()),
                connection_limit: json["connection_limit"].as_i64().unwrap_or(-1) as i32,
            };

            info!("Session authorized successfully: {}", authorization.session_id);
            debug!("Authorization details: {:?}", authorization);
            Ok(authorization)
        }
        Err(e) => {
            let error_msg = format!("Failed to parse session authorization JSON: {}", e);
            error!("{}", error_msg);
            Err(error_msg)
        }
    }
}

// Establish connection using authorization token
#[instrument]
async fn establish_connection(
    cli_path: &str,
    authorization: &BoundarySessionAuthorization,
    connection_type: ConnectionType,
    target_name: &str,
) -> Result<BoundaryConnection, String> {
    info!("Establishing {} connection for session: {}", format!("{:?}", connection_type).to_lowercase(), authorization.session_id);

    let type_str = match connection_type {
        ConnectionType::SSH => "ssh",
        ConnectionType::RDP => "rdp",
        ConnectionType::TCP => "tcp",
        ConnectionType::HTTP => "http",
    };

    let args = vec![
        "connect",
        type_str,
        "-authz-token",
        &authorization.authorization_token,
    ];

    let result = execute_boundary_command(cli_path, args, None).await?;

    if !result.success {
        let error_msg = format!("Failed to establish {} connection: {}", type_str, result.stderr);
        error!("{}", error_msg);
        return Err(error_msg);
    }

    // Parse the connection information from output
    let (local_address, local_port) = parse_connection_info(&result.stdout)?;

    let connection = BoundaryConnection {
        session_id: authorization.session_id.clone(),
        target_id: authorization.target_id.clone(),
        target_name: target_name.to_string(),
        connection_type: type_str.to_string(),
        local_address,
        local_port,
        status: "active".to_string(),
        created_time: chrono::Utc::now().to_rfc3339(),
        expiration_time: authorization.expiration_time.clone(),
    };

    info!("Connection established successfully: {}:{}", connection.local_address, connection.local_port);
    debug!("Connection details: {:?}", connection);
    Ok(connection)
}

// Parse connection information from CLI output
fn parse_connection_info(output: &str) -> Result<(String, u16), String> {
    debug!("Parsing connection info from output: {}", output);

    // Look for patterns like "Address: 127.0.0.1" and "Port: 61991"
    let address_regex = Regex::new(r"Address:\s+([^\s]+)").map_err(|e| format!("Address regex error: {}", e))?;
    let port_regex = Regex::new(r"Port:\s+(\d+)").map_err(|e| format!("Port regex error: {}", e))?;

    let address = address_regex
        .captures(output)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
        .unwrap_or_else(|| "127.0.0.1".to_string());

    let port = port_regex
        .captures(output)
        .and_then(|caps| caps.get(1))
        .and_then(|m| m.as_str().parse::<u16>().ok())
        .unwrap_or_else(|| {
            warn!("Could not parse port from output, using default 0");
            0
        });

    if port == 0 {
        return Err("Failed to parse valid port from connection output".to_string());
    }

    info!("Parsed connection info: {}:{}", address, port);
    Ok((address, port))
}

// Terminate an active connection
#[instrument]
async fn terminate_connection(connection: &BoundaryConnection) -> Result<(), String> {
    info!("Terminating connection for session: {}", connection.session_id);

    // For now, we'll handle connection termination by killing the process
    // In a more sophisticated implementation, we might track process IDs
    warn!("Connection termination not fully implemented - manual cleanup may be required");

    Ok(())
}

// Check if a command/executable exists in the system
async fn check_command_exists(command: &str) -> bool {
    match Command::new("which")
        .arg(command)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await
    {
        Ok(status) => status.success(),
        Err(_) => false,
    }
}

// Check if a file exists at a specific path
fn check_file_exists(path: &str) -> bool {
    std::path::Path::new(path).exists()
}

// Get the current platform
fn get_current_platform() -> String {
    #[cfg(target_os = "windows")]
    return "windows".to_string();

    #[cfg(target_os = "macos")]
    return "macos".to_string();

    #[cfg(target_os = "linux")]
    return "linux".to_string();

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    return "unknown".to_string();
}

// Detect RDP clients on Windows
#[instrument]
async fn detect_windows_rdp_clients() -> Vec<RdpClientInfo> {
    info!("Detecting Windows RDP clients");
    let mut clients = Vec::new();

    // Windows built-in RDP client (mstsc)
    if check_command_exists("mstsc").await {
        clients.push(RdpClientInfo {
            name: "Microsoft Terminal Services Client".to_string(),
            executable_path: "mstsc".to_string(),
            client_type: "builtin".to_string(),
            platform: "windows".to_string(),
            version: None,
            supports_fullscreen: true,
            supports_resolution: true,
            supports_credentials: true,
        });
        info!("Found Windows built-in RDP client (mstsc)");
    }

    // Check for other Windows RDP clients in common locations
    let other_clients = vec![
        ("Royal TS", "C:\\Program Files\\Royal TS V6\\RoyalTS.exe", "third_party"),
        ("Remote Desktop Manager", "C:\\Program Files\\Devolutions\\Remote Desktop Manager\\RemoteDesktopManager.exe", "third_party"),
        ("Jump Desktop", "C:\\Program Files\\Jump Desktop\\JumpDesktop.exe", "third_party"),
    ];

    for (name, path, client_type) in other_clients {
        if check_file_exists(path) {
            clients.push(RdpClientInfo {
                name: name.to_string(),
                executable_path: path.to_string(),
                client_type: client_type.to_string(),
                platform: "windows".to_string(),
                version: None,
                supports_fullscreen: true,
                supports_resolution: true,
                supports_credentials: true,
            });
            info!("Found Windows RDP client: {}", name);
        }
    }

    info!("Detected {} Windows RDP clients", clients.len());
    clients
}

// Detect RDP clients on macOS
#[instrument]
async fn detect_macos_rdp_clients() -> Vec<RdpClientInfo> {
    info!("Detecting macOS RDP clients");
    let mut clients = Vec::new();

    // Common macOS RDP client paths
    let macos_clients = vec![
        (
            "Microsoft Remote Desktop",
            "/Applications/Microsoft Remote Desktop.app/Contents/MacOS/Microsoft Remote Desktop",
            "microsoft"
        ),
        (
            "Royal TSX",
            "/Applications/Royal TSX.app/Contents/MacOS/Royal TSX",
            "third_party"
        ),
        (
            "Jump Desktop",
            "/Applications/Jump Desktop.app/Contents/MacOS/Jump Desktop",
            "third_party"
        ),
        (
            "Screens for Organizations",
            "/Applications/Screens for Organizations.app/Contents/MacOS/Screens for Organizations",
            "third_party"
        ),
        (
            "Remote Desktop Scanner",
            "/Applications/Remote Desktop Scanner.app/Contents/MacOS/Remote Desktop Scanner",
            "third_party"
        ),
    ];

    for (name, path, client_type) in macos_clients {
        if check_file_exists(path) {
            clients.push(RdpClientInfo {
                name: name.to_string(),
                executable_path: path.to_string(),
                client_type: client_type.to_string(),
                platform: "macos".to_string(),
                version: None,
                supports_fullscreen: true,
                supports_resolution: true,
                supports_credentials: true,
            });
            info!("Found macOS RDP client: {}", name);
        }
    }

    info!("Detected {} macOS RDP clients", clients.len());
    clients
}

// Detect RDP clients on Linux
#[instrument]
async fn detect_linux_rdp_clients() -> Vec<RdpClientInfo> {
    info!("Detecting Linux RDP clients");
    let mut clients = Vec::new();

    // Common Linux RDP clients
    let linux_clients = vec![
        ("xfreerdp", "xfreerdp", "freerdp"),
        ("rdesktop", "rdesktop", "rdesktop"),
        ("remmina", "remmina", "remmina"),
        ("vinagre", "vinagre", "vinagre"),
        ("tsclient", "tsclient", "tsclient"),
    ];

    for (name, command, client_type) in linux_clients {
        if check_command_exists(command).await {
            clients.push(RdpClientInfo {
                name: name.to_string(),
                executable_path: command.to_string(),
                client_type: client_type.to_string(),
                platform: "linux".to_string(),
                version: None,
                supports_fullscreen: true,
                supports_resolution: true,
                supports_credentials: true,
            });
            info!("Found Linux RDP client: {}", name);
        }
    }

    info!("Detected {} Linux RDP clients", clients.len());
    clients
}

// Detect all available RDP clients on the current platform
#[instrument]
async fn detect_rdp_clients() -> Result<DetectedRdpClients, String> {
    let platform = get_current_platform();
    info!("Detecting RDP clients for platform: {}", platform);

    let clients = match platform.as_str() {
        "windows" => detect_windows_rdp_clients().await,
        "macos" => detect_macos_rdp_clients().await,
        "linux" => detect_linux_rdp_clients().await,
        _ => {
            warn!("Unsupported platform for RDP client detection: {}", platform);
            Vec::new()
        }
    };

    // Determine default client based on platform preferences
    let default_client = if !clients.is_empty() {
        match platform.as_str() {
            "windows" => {
                // Prefer built-in mstsc on Windows
                clients.iter()
                    .find(|c| c.client_type == "builtin")
                    .or_else(|| clients.first())
                    .map(|c| c.name.clone())
            },
            "macos" => {
                // Prefer Microsoft Remote Desktop on macOS
                clients.iter()
                    .find(|c| c.name.contains("Microsoft Remote Desktop"))
                    .or_else(|| clients.first())
                    .map(|c| c.name.clone())
            },
            "linux" => {
                // Prefer xfreerdp on Linux
                clients.iter()
                    .find(|c| c.name == "xfreerdp")
                    .or_else(|| clients.first())
                    .map(|c| c.name.clone())
            },
            _ => clients.first().map(|c| c.name.clone()),
        }
    } else {
        None
    };

    let result = DetectedRdpClients {
        clients,
        default_client,
        platform,
    };

    info!("RDP client detection completed: {} clients found, default: {:?}",
          result.clients.len(), result.default_client);
    debug!("Detected RDP clients: {:?}", result);

    Ok(result)
}

// Launch an RDP client with connection details
#[instrument]
async fn launch_rdp_client(
    client_info: &RdpClientInfo,
    connection: &BoundaryConnection,
    config: &RdpConfig,
) -> Result<(), String> {
    info!("Launching RDP client: {} for connection {}:{}",
          client_info.name, connection.local_address, connection.local_port);

    let platform = get_current_platform();
    let mut cmd = Command::new(&client_info.executable_path);

    match platform.as_str() {
        "windows" if client_info.name.contains("Microsoft Terminal Services Client") => {
            // Windows mstsc command line arguments
            cmd.arg(&format!("{}:{}", connection.local_address, connection.local_port));

            if config.fullscreen {
                cmd.arg("/f");
            }

            if config.resolution != "auto" {
                cmd.arg("/w").arg(config.resolution.split('x').next().unwrap_or("1920"));
                cmd.arg("/h").arg(config.resolution.split('x').nth(1).unwrap_or("1080"));
            }
        },
        "macos" if client_info.name.contains("Microsoft Remote Desktop") => {
            // Microsoft Remote Desktop for macOS uses different arguments
            cmd.arg("rdp://").arg(&format!("{}:{}", connection.local_address, connection.local_port));
        },
        "linux" if client_info.name == "xfreerdp" => {
            // xfreerdp command line arguments
            cmd.arg(&format!("/v:{}:{}", connection.local_address, connection.local_port));

            if config.fullscreen {
                cmd.arg("/f");
            }

            if config.resolution != "auto" {
                cmd.arg(&format!("/size:{}", config.resolution));
            }
        },
        "linux" if client_info.name == "rdesktop" => {
            // rdesktop command line arguments
            cmd.arg(&format!("{}:{}", connection.local_address, connection.local_port));

            if config.fullscreen {
                cmd.arg("-f");
            }

            if config.resolution != "auto" {
                cmd.arg("-g").arg(&config.resolution);
            }
        },
        "linux" if client_info.name == "remmina" => {
            // remmina can take an RDP connection string
            cmd.arg(&format!("rdp://{}:{}", connection.local_address, connection.local_port));
        },
        _ => {
            // Generic fallback - just pass the address and port
            cmd.arg(&format!("{}:{}", connection.local_address, connection.local_port));
        }
    }

    info!("Executing RDP client command: {:?}", cmd);

    match cmd.spawn() {
        Ok(mut child) => {
            info!("RDP client launched successfully with PID: {:?}", child.id());

            // Don't wait for the child process to complete, as RDP clients typically run independently
            tokio::spawn(async move {
                match child.wait().await {
                    Ok(status) => {
                        info!("RDP client exited with status: {}", status);
                    }
                    Err(e) => {
                        error!("Error waiting for RDP client: {}", e);
                    }
                }
            });

            Ok(())
        }
        Err(e) => {
            let error_msg = format!("Failed to launch RDP client '{}': {}", client_info.name, e);
            error!("{}", error_msg);
            Err(error_msg)
        }
    }
}

// Create a keychain entry for a specific server and user
fn create_keychain_entry(server_id: &str, user_id: &str) -> Result<Entry, String> {
    let service = "regis-boundary-client";
    let account = format!("{}@{}", user_id, server_id);

    Entry::new(service, &account).map_err(|e| {
        format!("Failed to create keychain entry for {}: {}", account, e)
    })
}

// Store authentication token in system keychain
#[instrument]
async fn store_auth_token(token: &StoredToken) -> Result<(), String> {
    info!("Storing authentication token for user {} on server {}",
          token.user_id, token.server_id);

    let entry = create_keychain_entry(&token.server_id, &token.user_id)?;

    // Serialize the token data to JSON for storage
    let token_json = serde_json::to_string(token).map_err(|e| {
        format!("Failed to serialize token for storage: {}", e)
    })?;

    // Store the serialized token in keychain
    entry.set_password(&token_json).map_err(|e| {
        let error_msg = format!("Failed to store token in keychain: {}", e);
        error!("{}", error_msg);
        error_msg
    })?;

    info!("Authentication token stored successfully in keychain");
    debug!("Token metadata: server_id={}, user_id={}, scope_id={}, expires_at={:?}",
           token.server_id, token.user_id, token.scope_id, token.expires_at);

    Ok(())
}

// Retrieve authentication token from system keychain
#[instrument]
async fn retrieve_auth_token(server_id: &str, user_id: &str) -> Result<StoredToken, String> {
    info!("Retrieving authentication token for user {} on server {}", user_id, server_id);

    let entry = create_keychain_entry(server_id, user_id)?;

    // Retrieve the token from keychain
    let token_json = entry.get_password().map_err(|e| {
        let error_msg = format!("Failed to retrieve token from keychain: {}", e);
        debug!("{}", error_msg);
        error_msg
    })?;

    // Deserialize the token data from JSON
    let token: StoredToken = serde_json::from_str(&token_json).map_err(|e| {
        let error_msg = format!("Failed to deserialize stored token: {}", e);
        error!("{}", error_msg);
        error_msg
    })?;

    // Check if token has expired
    if let Some(expires_at) = &token.expires_at {
        let now = chrono::Utc::now();
        match chrono::DateTime::parse_from_rfc3339(expires_at) {
            Ok(expiry_time) => {
                if now > expiry_time {
                    let error_msg = format!("Stored token has expired (expired at: {})", expires_at);
                    warn!("{}", error_msg);
                    return Err(error_msg);
                }
            }
            Err(e) => {
                warn!("Failed to parse token expiry time '{}': {}", expires_at, e);
            }
        }
    }

    info!("Authentication token retrieved successfully from keychain");
    debug!("Token metadata: server_id={}, user_id={}, scope_id={}, expires_at={:?}",
           token.server_id, token.user_id, token.scope_id, token.expires_at);

    Ok(token)
}

// Delete authentication token from system keychain
#[instrument]
async fn delete_auth_token(server_id: &str, user_id: &str) -> Result<(), String> {
    info!("Deleting authentication token for user {} on server {}", user_id, server_id);

    let entry = create_keychain_entry(server_id, user_id)?;

    entry.delete_password().map_err(|e| {
        let error_msg = format!("Failed to delete token from keychain: {}", e);
        error!("{}", error_msg);
        error_msg
    })?;

    info!("Authentication token deleted successfully from keychain");
    Ok(())
}

// Check if authentication token exists in keychain
#[instrument]
async fn token_exists_in_keychain(server_id: &str, user_id: &str) -> bool {
    debug!("Checking if token exists for user {} on server {}", user_id, server_id);

    match create_keychain_entry(server_id, user_id) {
        Ok(entry) => {
            match entry.get_password() {
                Ok(_) => {
                    debug!("Token found in keychain");
                    true
                }
                Err(_) => {
                    debug!("No token found in keychain");
                    false
                }
            }
        }
        Err(e) => {
            debug!("Failed to create keychain entry: {}", e);
            false
        }
    }
}

// List all stored tokens (metadata only, not actual tokens)
#[instrument]
async fn list_stored_token_metadata() -> Result<Vec<TokenMetadata>, String> {
    info!("Listing stored token metadata");

    // Note: The keyring crate doesn't provide a way to enumerate all entries,
    // so we'll need to maintain a separate index or return empty for now.
    // In a production implementation, you might want to store an index file
    // in the user's config directory that tracks which tokens are stored.

    warn!("Token enumeration not fully implemented - returning empty list");
    debug!("Consider implementing a token index file for production use");

    Ok(Vec::new())
}

// Check if a stored token is expired or will expire soon
#[instrument]
async fn is_token_expired_or_expiring(token: &StoredToken, threshold_minutes: u32) -> Result<bool, String> {
    let expires_at = match &token.expires_at {
        Some(expiry_str) => expiry_str,
        None => {
            debug!("Token has no expiration time - assuming valid");
            return Ok(false);
        }
    };

    let expiry_time = chrono::DateTime::parse_from_rfc3339(expires_at)
        .map_err(|e| format!("Failed to parse token expiry time '{}': {}", expires_at, e))?;

    let now = chrono::Utc::now();
    let threshold = chrono::Duration::minutes(threshold_minutes as i64);
    let expiry_threshold = expiry_time - threshold;

    let is_expiring = now >= expiry_threshold;

    if is_expiring {
        info!("Token for user {} on server {} is expired or expiring soon (expires: {}, threshold: {} minutes)",
              token.user_id, token.server_id, expires_at, threshold_minutes);
    } else {
        debug!("Token for user {} on server {} is still valid (expires: {})",
               token.user_id, token.server_id, expires_at);
    }

    Ok(is_expiring)
}

// Validate current token by checking its status via Boundary CLI
#[instrument]
async fn validate_token_with_cli(
    cli_path: &str,
    server_addr: &str,
    token: &StoredToken
) -> Result<bool, String> {
    debug!("Validating token for user {} on server {} via CLI", token.user_id, token.server_id);

    // Set environment variable for the CLI to use this token
    std::env::set_var("BOUNDARY_TOKEN", &token.access_token);

    // Try a simple command that requires authentication
    let result = execute_boundary_command(
        cli_path,
        vec!["auth-tokens", "list", "-format", "json"],
        Some(server_addr),
    ).await;

    // Clean up environment variable
    std::env::remove_var("BOUNDARY_TOKEN");

    match result {
        Ok(cmd_result) => {
            if cmd_result.success {
                debug!("Token validation successful");
                Ok(true)
            } else {
                warn!("Token validation failed: {}", cmd_result.stderr);
                Ok(false)
            }
        }
        Err(e) => {
            warn!("Token validation error: {}", e);
            Ok(false)
        }
    }
}

// Trigger re-authentication for an expired token
#[instrument]
async fn trigger_reauthentication(
    cli_path: &str,
    server_addr: &str,
    auth_method_id: &str,
    server_id: &str,
) -> Result<StoredToken, String> {
    info!("Triggering re-authentication for server {} with auth method {}", server_id, auth_method_id);

    // For OIDC authentication, the CLI handles the flow automatically
    let result = execute_boundary_command(
        cli_path,
        vec!["authenticate", "oidc", "-auth-method-id", auth_method_id, "-format", "json"],
        Some(server_addr),
    ).await?;

    if !result.success {
        let error_msg = format!("Re-authentication failed: {}", result.stderr);
        error!("{}", error_msg);
        return Err(error_msg);
    }

    // Parse the authentication result
    let auth_info: serde_json::Value = serde_json::from_str(&result.stdout)
        .map_err(|e| format!("Failed to parse authentication response: {}", e))?;

    // Extract token information
    let access_token = auth_info["token"].as_str()
        .or_else(|| {
            // If token is not in output, try to get it from CLI config
            warn!("Token not found in auth response, will attempt to retrieve from CLI config");
            None
        })
        .unwrap_or_default()
        .to_string();

    let user_id = auth_info["user_id"].as_str().unwrap_or("").to_string();
    let expiration_time = auth_info["expiration_time"].as_str().map(|s| s.to_string());

    // If we couldn't get the token from the response, try the CLI config
    let final_token = if access_token.is_empty() {
        // Try to get token from CLI keyring
        let token_result = execute_boundary_command(
            cli_path,
            vec!["config", "get-token"],
            None,
        ).await?;

        if token_result.success {
            token_result.stdout.trim().to_string()
        } else {
            return Err("Failed to retrieve token after authentication".to_string());
        }
    } else {
        access_token
    };

    let new_token = StoredToken {
        access_token: final_token,
        refresh_token: None, // Boundary doesn't use refresh tokens
        expires_at: expiration_time,
        server_id: server_id.to_string(),
        user_id,
        scope_id: "global".to_string(), // Default scope
        created_at: chrono::Utc::now().to_rfc3339(),
    };

    info!("Re-authentication successful for server {}", server_id);
    Ok(new_token)
}

// Check and refresh token if needed
#[instrument]
async fn check_and_refresh_token(
    cli_path: &str,
    server_addr: &str,
    auth_method_id: &str,
    server_id: &str,
    user_id: &str,
    threshold_minutes: u32,
) -> Result<Option<StoredToken>, String> {
    debug!("Checking token status for user {} on server {}", user_id, server_id);

    // Try to retrieve current token
    let current_token = match retrieve_auth_token(server_id, user_id).await {
        Ok(token) => token,
        Err(_) => {
            info!("No stored token found for user {} on server {}", user_id, server_id);
            return Ok(None);
        }
    };

    // Check if token is expired or expiring soon
    let needs_refresh = is_token_expired_or_expiring(&current_token, threshold_minutes).await?;

    if !needs_refresh {
        // Double-check with CLI validation
        if validate_token_with_cli(cli_path, server_addr, &current_token).await? {
            debug!("Token is valid, no refresh needed");
            return Ok(Some(current_token));
        } else {
            info!("Token failed CLI validation, triggering re-authentication");
        }
    }

    // Token needs refresh - trigger re-authentication
    info!("Token expired or invalid, triggering re-authentication");
    let new_token = trigger_reauthentication(cli_path, server_addr, auth_method_id, server_id).await?;

    // Store the new token
    store_auth_token(&new_token).await?;

    info!("Token refresh completed successfully");
    Ok(Some(new_token))
}

// Auto-logout functionality - remove expired tokens
#[instrument]
async fn auto_logout_expired_tokens(max_age_minutes: u32) -> Result<Vec<String>, String> {
    info!("Performing auto-logout check for tokens older than {} minutes", max_age_minutes);

    // Note: This is a simplified implementation since keyring doesn't provide enumeration
    // In a production implementation, you would maintain an index of stored tokens

    let logged_out_servers = Vec::new();

    // This is a placeholder - in reality you would:
    // 1. Maintain a list/index of active tokens
    // 2. Check each token's expiry
    // 3. Remove expired tokens from keychain
    // 4. Return list of servers that were logged out

    warn!("Auto-logout functionality needs token enumeration - consider implementing token index");

    Ok(logged_out_servers)
}

// Start background token monitoring service
#[instrument]
async fn start_token_monitoring(
    app_state: Arc<AppState>,
    check_interval_seconds: u64,
    refresh_threshold_minutes: u32,
) {
    info!("Starting background token monitoring service");
    info!("Check interval: {}s, Refresh threshold: {}min", check_interval_seconds, refresh_threshold_minutes);

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(check_interval_seconds));

        loop {
            interval.tick().await;

            // Check if monitoring is still enabled
            let monitoring_enabled = {
                let enabled = app_state.monitoring_enabled.lock().unwrap();
                *enabled
            };

            if !monitoring_enabled {
                debug!("Token monitoring stopped");
                break;
            }

            // Perform auto-logout check
            let auto_logout_minutes = app_state.config.security.auto_logout_minutes;
            match auto_logout_expired_tokens(auto_logout_minutes).await {
                Ok(logged_out) => {
                    if !logged_out.is_empty() {
                        info!("Auto-logout completed for {} servers: {:?}", logged_out.len(), logged_out);
                    }
                }
                Err(e) => {
                    error!("Auto-logout check failed: {}", e);
                }
            }

            debug!("Token monitoring cycle completed");
        }
    });

    info!("Background token monitoring started");
}

// Check session health by verifying connection is still responsive
#[instrument]
async fn check_session_health(
    cli_path: &str,
    server_addr: &str,
    session_id: &str,
) -> Result<SessionHealth, String> {
    debug!("Checking health for session: {}", session_id);

    let start_time = std::time::Instant::now();
    let timestamp = chrono::Utc::now().to_rfc3339();

    // Use Boundary CLI to check session status
    let result = execute_boundary_command(
        cli_path,
        vec!["sessions", "read", "-id", session_id, "-format", "json"],
        Some(server_addr),
    ).await;

    let (status, response_time, error_count) = match result {
        Ok(cmd_result) => {
            if cmd_result.success {
                let response_time = start_time.elapsed().as_millis() as u64;

                // Try to parse the session info to get more details
                match serde_json::from_str::<serde_json::Value>(&cmd_result.stdout) {
                    Ok(session_info) => {
                        let session_status = session_info["status"].as_str().unwrap_or("unknown");
                        debug!("Session {} status from CLI: {}", session_id, session_status);

                        if session_status == "active" {
                            ("healthy".to_string(), Some(response_time), 0)
                        } else {
                            (format!("session_status_{}", session_status), Some(response_time), 0)
                        }
                    }
                    Err(e) => {
                        warn!("Failed to parse session info JSON: {}", e);
                        ("healthy".to_string(), Some(response_time), 0)
                    }
                }
            } else {
                warn!("Session health check failed: {}", cmd_result.stderr);
                ("unhealthy".to_string(), None, 1)
            }
        }
        Err(e) => {
            warn!("Session health check error: {}", e);
            ("error".to_string(), None, 1)
        }
    };

    let health = SessionHealth {
        session_id: session_id.to_string(),
        status,
        last_check: timestamp,
        response_time_ms: response_time,
        error_count,
        consecutive_failures: if error_count > 0 { 1 } else { 0 },
    };

    debug!("Session health check result: {:?}", health);
    Ok(health)
}

// Monitor all active sessions and update their health status
#[instrument]
async fn monitor_active_sessions(
    app_state: &AppState,
) -> Result<SessionMonitoringStats, String> {
    info!("Monitoring active sessions");

    let monitoring_enabled = {
        let enabled = app_state.monitoring_enabled.lock().unwrap();
        *enabled
    };

    if !monitoring_enabled {
        debug!("Session monitoring is disabled");
        return Ok(SessionMonitoringStats {
            total_sessions: 0,
            active_sessions: 0,
            failed_sessions: 0,
            monitoring_enabled: false,
            last_check: chrono::Utc::now().to_rfc3339(),
        });
    }

    let connections = {
        let active_connections = app_state.active_connections.lock().unwrap();
        active_connections.clone()
    };

    let mut health_checks = Vec::new();
    let mut stats = SessionMonitoringStats {
        total_sessions: connections.len(),
        active_sessions: 0,
        failed_sessions: 0,
        monitoring_enabled: true,
        last_check: chrono::Utc::now().to_rfc3339(),
    };

    for connection in &connections {
        // This is a placeholder - in a real implementation, you would need
        // the CLI path and server address for each connection
        // For now, we'll create a dummy health check
        let health = SessionHealth {
            session_id: connection.session_id.clone(),
            status: if connection.status == "active" {
                "healthy".to_string()
            } else {
                "unhealthy".to_string()
            },
            last_check: chrono::Utc::now().to_rfc3339(),
            response_time_ms: Some(50), // Mock response time
            error_count: 0,
            consecutive_failures: 0,
        };

        if health.status == "healthy" {
            stats.active_sessions += 1;
        } else {
            stats.failed_sessions += 1;
        }

        health_checks.push(health);
    }

    // Update the session health tracking
    {
        let mut session_health = app_state.session_health.lock().unwrap();
        for health in health_checks {
            session_health.insert(health.session_id.clone(), health);
        }
    }

    info!("Session monitoring completed: {} total, {} active, {} failed",
          stats.total_sessions, stats.active_sessions, stats.failed_sessions);

    Ok(stats)
}

// Start periodic session monitoring
#[instrument]
async fn start_session_monitoring(app_state: Arc<AppState>) {
    info!("Starting periodic session monitoring");

    // Enable monitoring
    {
        let mut enabled = app_state.monitoring_enabled.lock().unwrap();
        *enabled = true;
    }

    // Spawn a background task for periodic monitoring
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));

        loop {
            interval.tick().await;

            let monitoring_enabled = {
                let enabled = app_state.monitoring_enabled.lock().unwrap();
                *enabled
            };

            if !monitoring_enabled {
                debug!("Session monitoring stopped");
                break;
            }

            match monitor_active_sessions(&app_state).await {
                Ok(stats) => {
                    debug!("Session monitoring cycle completed: {:?}", stats);
                }
                Err(e) => {
                    error!("Session monitoring failed: {}", e);
                }
            }
        }
    });

    info!("Session monitoring started");
}

// Stop session monitoring
#[instrument]
async fn stop_session_monitoring(app_state: &AppState) -> Result<(), String> {
    info!("Stopping session monitoring");

    {
        let mut enabled = app_state.monitoring_enabled.lock().unwrap();
        *enabled = false;
    }

    info!("Session monitoring stopped");
    Ok(())
}

// Get current session health for a specific session
#[instrument]
async fn get_session_health(
    app_state: &AppState,
    session_id: &str,
) -> Result<SessionHealth, String> {
    debug!("Getting health for session: {}", session_id);

    let session_health = app_state.session_health.lock().unwrap();
    session_health
        .get(session_id)
        .cloned()
        .ok_or_else(|| format!("No health data found for session: {}", session_id))
}

// Get monitoring statistics for all sessions
#[instrument]
async fn get_monitoring_stats(app_state: &AppState) -> Result<SessionMonitoringStats, String> {
    debug!("Getting session monitoring statistics");

    let connections = {
        let active_connections = app_state.active_connections.lock().unwrap();
        active_connections.len()
    };

    let (active_count, failed_count) = {
        let session_health = app_state.session_health.lock().unwrap();
        let mut active = 0;
        let mut failed = 0;

        for health in session_health.values() {
            if health.status == "healthy" {
                active += 1;
            } else {
                failed += 1;
            }
        }

        (active, failed)
    };

    let monitoring_enabled = {
        let enabled = app_state.monitoring_enabled.lock().unwrap();
        *enabled
    };

    Ok(SessionMonitoringStats {
        total_sessions: connections,
        active_sessions: active_count,
        failed_sessions: failed_count,
        monitoring_enabled,
        last_check: chrono::Utc::now().to_rfc3339(),
    })
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

// Load configuration with user overrides
#[instrument(skip(app))]
fn load_config(app: &AppHandle) -> Config {
    debug!("Loading application configuration with user overrides");

    // Step 1: Load system configuration
    let mut config = match load_resource_with_fallback(app, "config.json") {
        Ok(config_content) => {
            match serde_json::from_str::<Config>(&config_content) {
                Ok(parsed_config) => {
                    info!("Successfully loaded and parsed system config.json");
                    parsed_config
                }
                Err(e) => {
                    error!("Failed to parse system config.json: {}. Using default configuration.", e);
                    Config::default()
                }
            }
        }
        Err(e) => {
            error!("Failed to load system config.json: {}. Using default configuration.", e);
            Config::default()
        }
    };

    // Step 2: Try to load user configuration overrides
    match get_user_config_path() {
        Ok(user_config_path) => {
            debug!("Checking for user config at: {:?}", user_config_path);

            if user_config_path.exists() {
                info!("Found user configuration file: {:?}", user_config_path);

                match fs::read_to_string(&user_config_path) {
                    Ok(user_config_content) => {
                        match serde_json::from_str::<Config>(&user_config_content) {
                            Ok(user_config) => {
                                info!("Successfully loaded and parsed user config overrides");
                                config.merge_with_user_config(user_config);
                                info!("User configuration overrides applied successfully");
                            }
                            Err(e) => {
                                error!("Failed to parse user config.json: {}. Continuing with system config only.", e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to read user config.json: {}. Continuing with system config only.", e);
                    }
                }
            } else {
                debug!("No user configuration file found. Using system config only.");
            }
        }
        Err(e) => {
            error!("Failed to determine user config path: {}. Continuing with system config only.", e);
        }
    }

    debug!("Final merged configuration: {:?}", config);
    config
}

// Tauri commands
#[command]
#[instrument(skip(app))]
async fn load_servers(app: AppHandle) -> Result<Vec<Server>, String> {
    info!("Loading servers from system and user configurations");

    // Step 1: Load system servers
    let mut all_servers = Vec::new();

    match load_resource_with_fallback(&app, "servers.json") {
        Ok(config_content) => {
            match serde_json::from_str::<ServerConfig>(&config_content) {
                Ok(config) => {
                    info!("Successfully loaded {} system servers", config.servers.len());
                    debug!("System servers: {:?}", config.servers);
                    all_servers.extend(config.servers);
                }
                Err(e) => {
                    let error_msg = format!("Failed to parse system servers.json: {}", e);
                    error!("{}", error_msg);
                    return Err(error_msg);
                }
            }
        }
        Err(e) => {
            let error_msg = format!("Failed to load system servers.json: {}", e);
            error!("{}", error_msg);
            return Err(error_msg);
        }
    }

    // Step 2: Try to load user servers (additive)
    match get_user_servers_path() {
        Ok(user_servers_path) => {
            debug!("Checking for user servers at: {:?}", user_servers_path);

            if user_servers_path.exists() {
                info!("Found user servers file: {:?}", user_servers_path);

                match fs::read_to_string(&user_servers_path) {
                    Ok(user_servers_content) => {
                        match serde_json::from_str::<ServerConfig>(&user_servers_content) {
                            Ok(user_config) => {
                                info!("Successfully loaded {} user servers", user_config.servers.len());
                                debug!("User servers: {:?}", user_config.servers);

                                // Add user servers to the list (additive merge)
                                all_servers.extend(user_config.servers);
                                info!("User servers added successfully");
                            }
                            Err(e) => {
                                error!("Failed to parse user servers.json: {}. Continuing with system servers only.", e);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to read user servers.json: {}. Continuing with system servers only.", e);
                    }
                }
            } else {
                debug!("No user servers file found. Using system servers only.");
            }
        }
        Err(e) => {
            error!("Failed to determine user servers path: {}. Continuing with system servers only.", e);
        }
    }

    info!("Total servers loaded: {}", all_servers.len());
    debug!("All servers: {:?}", all_servers);

    Ok(all_servers)
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

#[command]
#[instrument(skip(app))]
async fn verify_boundary_cli_command(app: AppHandle, server_id: String) -> Result<bool, String> {
    info!("Verifying Boundary CLI for server: {}", server_id);

    let state = app.state::<AppState>();

    // Load servers to find the specific server
    let servers = load_servers(app.clone()).await?;
    let server = servers
        .iter()
        .find(|s| s.id == server_id)
        .ok_or_else(|| format!("Server with id '{}' not found", server_id))?;

    let cli_path = get_boundary_cli_path(server, &state.config);
    info!("Using CLI path: {}", cli_path);

    verify_boundary_cli(&cli_path).await
}

#[command]
#[instrument(skip(app))]
async fn discover_auth_methods_command(app: AppHandle, server_id: String) -> Result<Vec<BoundaryAuthMethod>, String> {
    info!("Discovering auth methods for server: {}", server_id);

    let state = app.state::<AppState>();

    // Load servers to find the specific server
    let servers = load_servers(app.clone()).await?;
    let server = servers
        .iter()
        .find(|s| s.id == server_id)
        .ok_or_else(|| format!("Server with id '{}' not found", server_id))?;

    let cli_path = get_boundary_cli_path(server, &state.config);
    info!("Using CLI path: {} for server: {}", cli_path, server.url);

    discover_auth_methods(&cli_path, &server.url).await
}

#[command]
#[instrument(skip(app))]
async fn discover_scopes_command(app: AppHandle, server_id: String) -> Result<Vec<BoundaryScope>, String> {
    info!("Discovering scopes for server: {}", server_id);

    let state = app.state::<AppState>();

    // Load servers to find the specific server
    let servers = load_servers(app.clone()).await?;
    let server = servers
        .iter()
        .find(|s| s.id == server_id)
        .ok_or_else(|| format!("Server with id '{}' not found", server_id))?;

    let cli_path = get_boundary_cli_path(server, &state.config);
    info!("Using CLI path: {} for server: {}", cli_path, server.url);

    discover_scopes(&cli_path, &server.url).await
}

#[command]
#[instrument(skip(app))]
async fn discover_oidc_auth_methods_command(app: AppHandle, server_id: String) -> Result<Vec<BoundaryAuthMethod>, String> {
    info!("Discovering OIDC auth methods for server: {}", server_id);

    let state = app.state::<AppState>();

    // Load servers to find the specific server
    let servers = load_servers(app.clone()).await?;
    let server = servers
        .iter()
        .find(|s| s.id == server_id)
        .ok_or_else(|| format!("Server with id '{}' not found", server_id))?;

    let cli_path = get_boundary_cli_path(server, &state.config);
    info!("Using CLI path: {} for server: {}", cli_path, server.url);

    discover_oidc_auth_methods(&cli_path, &server.url).await
}

#[command]
#[instrument(skip(app))]
async fn verify_oidc_support_command(app: AppHandle, server_id: String) -> Result<bool, String> {
    info!("Verifying OIDC support for server: {}", server_id);

    let state = app.state::<AppState>();

    // Load servers to find the specific server
    let servers = load_servers(app.clone()).await?;
    let server = servers
        .iter()
        .find(|s| s.id == server_id)
        .ok_or_else(|| format!("Server with id '{}' not found", server_id))?;

    let cli_path = get_boundary_cli_path(server, &state.config);
    info!("Using CLI path: {} for server: {}", cli_path, server.url);

    verify_oidc_support(&cli_path, &server.url).await
}

#[command]
#[instrument(skip(app))]
async fn discover_targets_command(app: AppHandle, server_id: String, scope_id: Option<String>) -> Result<Vec<BoundaryTarget>, String> {
    info!("Discovering targets for server: {} in scope: {:?}", server_id, scope_id);

    let state = app.state::<AppState>();

    // Load servers to find the specific server
    let servers = load_servers(app.clone()).await?;
    let server = servers
        .iter()
        .find(|s| s.id == server_id)
        .ok_or_else(|| format!("Server with id '{}' not found", server_id))?;

    let cli_path = get_boundary_cli_path(server, &state.config);
    info!("Using CLI path: {} for server: {}", cli_path, server.url);

    discover_targets(&cli_path, &server.url, scope_id.as_deref()).await
}

#[command]
#[instrument(skip(app))]
async fn discover_all_targets_command(app: AppHandle, server_id: String) -> Result<Vec<BoundaryTarget>, String> {
    info!("Discovering all targets for server: {}", server_id);

    let state = app.state::<AppState>();

    // Load servers to find the specific server
    let servers = load_servers(app.clone()).await?;
    let server = servers
        .iter()
        .find(|s| s.id == server_id)
        .ok_or_else(|| format!("Server with id '{}' not found", server_id))?;

    let cli_path = get_boundary_cli_path(server, &state.config);
    info!("Using CLI path: {} for server: {}", cli_path, server.url);

    discover_all_targets(&cli_path, &server.url).await
}

#[command]
#[instrument(skip(app))]
async fn authorize_session_command(app: AppHandle, server_id: String, target_id: String, host_id: Option<String>) -> Result<BoundarySessionAuthorization, String> {
    info!("Authorizing session for target: {} on server: {}", target_id, server_id);

    let state = app.state::<AppState>();

    // Load servers to find the specific server
    let servers = load_servers(app.clone()).await?;
    let server = servers
        .iter()
        .find(|s| s.id == server_id)
        .ok_or_else(|| format!("Server with id '{}' not found", server_id))?;

    let cli_path = get_boundary_cli_path(server, &state.config);
    info!("Using CLI path: {} for server: {}", cli_path, server.url);

    authorize_session(&cli_path, &server.url, &target_id, host_id.as_deref()).await
}

#[command]
#[instrument(skip(app))]
async fn establish_connection_command(
    app: AppHandle,
    server_id: String,
    authorization: BoundarySessionAuthorization,
    connection_type: String,
    target_name: String,
) -> Result<BoundaryConnection, String> {
    info!("Establishing {} connection for session: {}", connection_type, authorization.session_id);

    let state = app.state::<AppState>();

    // Load servers to find the specific server
    let servers = load_servers(app.clone()).await?;
    let server = servers
        .iter()
        .find(|s| s.id == server_id)
        .ok_or_else(|| format!("Server with id '{}' not found", server_id))?;

    let cli_path = get_boundary_cli_path(server, &state.config);

    // Parse connection type
    let conn_type = match connection_type.to_lowercase().as_str() {
        "ssh" => ConnectionType::SSH,
        "rdp" => ConnectionType::RDP,
        "tcp" => ConnectionType::TCP,
        "http" => ConnectionType::HTTP,
        _ => return Err(format!("Unsupported connection type: {}", connection_type)),
    };

    let connection = establish_connection(&cli_path, &authorization, conn_type, &target_name).await?;

    // Add to active connections
    {
        let mut active_connections = state.active_connections.lock().unwrap();
        active_connections.push(connection.clone());
    }

    info!("Connection established and tracked: {}:{}", connection.local_address, connection.local_port);
    Ok(connection)
}

#[command]
#[instrument(skip(app))]
async fn get_active_connections_command(app: AppHandle) -> Result<Vec<BoundaryConnection>, String> {
    let state = app.state::<AppState>();
    let active_connections = state.active_connections.lock().unwrap();
    Ok(active_connections.clone())
}

#[command]
#[instrument(skip(app))]
async fn terminate_connection_command(app: AppHandle, session_id: String) -> Result<(), String> {
    info!("Terminating connection for session: {}", session_id);

    let state = app.state::<AppState>();

    // Find and remove the connection
    let connection = {
        let mut active_connections = state.active_connections.lock().unwrap();
        let index = active_connections
            .iter()
            .position(|conn| conn.session_id == session_id)
            .ok_or_else(|| format!("Connection with session id '{}' not found", session_id))?;

        active_connections.remove(index)
    };

    // Terminate the connection
    terminate_connection(&connection).await?;

    info!("Connection terminated and removed from tracking: {}", session_id);
    Ok(())
}

#[command]
#[instrument]
async fn detect_rdp_clients_command() -> Result<DetectedRdpClients, String> {
    info!("Frontend requested RDP client detection");
    detect_rdp_clients().await
}

#[command]
#[instrument(skip(app))]
async fn launch_rdp_client_command(
    app: AppHandle,
    session_id: String,
    client_name: Option<String>
) -> Result<(), String> {
    info!("Frontend requested RDP client launch for session: {}", session_id);

    let state = app.state::<AppState>();

    // Find the connection
    let connection = {
        let active_connections = state.active_connections.lock().unwrap();
        active_connections
            .iter()
            .find(|conn| conn.session_id == session_id)
            .cloned()
            .ok_or_else(|| format!("Connection with session id '{}' not found", session_id))?
    };

    // Detect available RDP clients
    let detected_clients = detect_rdp_clients().await?;

    if detected_clients.clients.is_empty() {
        return Err("No RDP clients found on this system".to_string());
    }

    // Choose the client to use
    let client_to_use = if let Some(requested_client) = client_name {
        // User specified a particular client
        detected_clients.clients
            .iter()
            .find(|c| c.name == requested_client)
            .ok_or_else(|| format!("Requested RDP client '{}' not found", requested_client))?
    } else {
        // Use default client or first available
        let default_name = detected_clients.default_client
            .as_ref()
            .unwrap_or(&detected_clients.clients[0].name);

        detected_clients.clients
            .iter()
            .find(|c| c.name == *default_name)
            .unwrap_or(&detected_clients.clients[0])
    };

    info!("Using RDP client: {}", client_to_use.name);

    // Launch the RDP client
    launch_rdp_client(client_to_use, &connection, &state.config.rdp).await
}

#[command]
#[instrument]
async fn store_auth_token_command(token: StoredToken) -> Result<(), String> {
    info!("Frontend requested token storage for user {} on server {}",
          token.user_id, token.server_id);
    store_auth_token(&token).await
}

#[command]
#[instrument]
async fn retrieve_auth_token_command(server_id: String, user_id: String) -> Result<StoredToken, String> {
    info!("Frontend requested token retrieval for user {} on server {}", user_id, server_id);
    retrieve_auth_token(&server_id, &user_id).await
}

#[command]
#[instrument]
async fn delete_auth_token_command(server_id: String, user_id: String) -> Result<(), String> {
    info!("Frontend requested token deletion for user {} on server {}", user_id, server_id);
    delete_auth_token(&server_id, &user_id).await
}

#[command]
#[instrument]
async fn token_exists_command(server_id: String, user_id: String) -> Result<bool, String> {
    debug!("Frontend checking if token exists for user {} on server {}", user_id, server_id);
    Ok(token_exists_in_keychain(&server_id, &user_id).await)
}

#[command]
#[instrument]
async fn list_stored_tokens_command() -> Result<Vec<TokenMetadata>, String> {
    info!("Frontend requested list of stored token metadata");
    list_stored_token_metadata().await
}

#[command]
#[instrument(skip(app))]
async fn refresh_auth_token_command(
    app: AppHandle,
    server_id: String,
    user_id: String
) -> Result<StoredToken, String> {
    info!("Frontend requested token refresh for user {} on server {}", user_id, server_id);

    let state = app.state::<AppState>();

    // Retrieve the current stored token
    let current_token = retrieve_auth_token(&server_id, &user_id).await?;

    // Find the server configuration
    let servers = load_servers(app.clone()).await?;
    let server = servers
        .iter()
        .find(|s| s.id == server_id)
        .ok_or_else(|| format!("Server with id '{}' not found", server_id))?;

    let cli_path = get_boundary_cli_path(server, &state.config);

    // Get auth method for re-authentication
    let auth_methods = discover_oidc_auth_methods(&cli_path, &server.url).await?;
    let auth_method_id = auth_methods.first()
        .ok_or_else(|| "No OIDC auth methods available".to_string())?
        .id.clone();

    // Trigger re-authentication
    let refreshed_token = trigger_reauthentication(&cli_path, &server.url, &auth_method_id, &server_id).await?;

    // Store the refreshed token
    store_auth_token(&refreshed_token).await?;

    Ok(refreshed_token)
}

// Validate a token using Boundary CLI
#[command]
#[instrument(skip(app))]
async fn validate_token_command(
    app: AppHandle,
    server_id: String,
    user_id: String
) -> Result<bool, String> {
    info!("Frontend requested token validation for user {} on server {}", user_id, server_id);

    let state = app.state::<AppState>();

    // Retrieve the current stored token
    let current_token = retrieve_auth_token(&server_id, &user_id).await?;

    // Find the server configuration
    let servers = load_servers(app.clone()).await?;
    let server = servers
        .iter()
        .find(|s| s.id == server_id)
        .ok_or_else(|| format!("Server with id '{}' not found", server_id))?;

    let cli_path = get_boundary_cli_path(server, &state.config);

    // Validate the token
    validate_token_with_cli(&cli_path, &server.url, &current_token).await
}

// Check and refresh token if needed (main orchestration function)
#[command]
#[instrument(skip(app))]
async fn check_and_refresh_token_command(
    app: AppHandle,
    server_id: String,
    user_id: String,
    threshold_minutes: Option<u32>
) -> Result<StoredToken, String> {
    info!("Frontend requested token check and refresh for user {} on server {}", user_id, server_id);

    let state = app.state::<AppState>();
    let threshold = threshold_minutes.unwrap_or(5); // Default 5 minutes

    // Retrieve the current stored token
    let current_token = retrieve_auth_token(&server_id, &user_id).await?;

    // Find the server configuration
    let servers = load_servers(app.clone()).await?;
    let server = servers
        .iter()
        .find(|s| s.id == server_id)
        .ok_or_else(|| format!("Server with id '{}' not found", server_id))?;

    let cli_path = get_boundary_cli_path(server, &state.config);

    // Get auth method for re-authentication if needed
    let auth_methods = discover_oidc_auth_methods(&cli_path, &server.url).await?;
    let auth_method_id = auth_methods.first()
        .ok_or_else(|| "No OIDC auth methods available".to_string())?
        .id.clone();

    // Check and refresh token
    let result = check_and_refresh_token(
        &cli_path,
        &server.url,
        &current_token.access_token,
        &auth_method_id,
        &server_id,
        threshold
    ).await?;

    match result {
        Some(token) => Ok(token),
        None => Ok(current_token) // Return existing token if no refresh was needed
    }
}

// Start background token auto-refresh monitoring service
#[command]
#[instrument(skip(app))]
async fn start_token_auto_refresh_command(
    app: AppHandle,
    check_interval_seconds: Option<u64>,
    refresh_threshold_minutes: Option<u32>
) -> Result<(), String> {
    info!("Frontend requested to start token auto-refresh monitoring service");

    let state = app.state::<AppState>();
    let interval = check_interval_seconds.unwrap_or(300); // Default 5 minutes
    let threshold = refresh_threshold_minutes.unwrap_or(5); // Default 5 minutes

    // Start the background monitoring service
    let app_state = Arc::new(AppState {
        config: state.config.clone(),
        active_connections: state.active_connections.clone(),
        session_health: state.session_health.clone(),
        monitoring_enabled: state.monitoring_enabled.clone(),
    });
    start_token_monitoring(app_state, interval, threshold).await;

    Ok(())
}

// Auto-logout expired tokens
#[command]
#[instrument]
async fn auto_logout_expired_tokens_command() -> Result<Vec<String>, String> {
    info!("Frontend requested auto-logout of expired tokens");
    auto_logout_expired_tokens(60).await // Default 60 minutes max age
}

#[command]
#[instrument(skip(app))]
async fn start_session_monitoring_command(app: AppHandle) -> Result<(), String> {
    info!("Frontend requested to start session monitoring");

    let state = app.state::<AppState>();
    let app_state = Arc::new(AppState {
        config: state.config.clone(),
        active_connections: state.active_connections.clone(),
        session_health: state.session_health.clone(),
        monitoring_enabled: state.monitoring_enabled.clone(),
    });

    start_session_monitoring(app_state).await;
    Ok(())
}

#[command]
#[instrument(skip(app))]
async fn stop_session_monitoring_command(app: AppHandle) -> Result<(), String> {
    info!("Frontend requested to stop session monitoring");
    let state = app.state::<AppState>();
    stop_session_monitoring(&*state).await
}

#[command]
#[instrument(skip(app))]
async fn get_session_health_command(app: AppHandle, session_id: String) -> Result<SessionHealth, String> {
    debug!("Frontend requested health for session: {}", session_id);
    let state = app.state::<AppState>();
    get_session_health(&*state, &session_id).await
}

#[command]
#[instrument(skip(app))]
async fn get_monitoring_stats_command(app: AppHandle) -> Result<SessionMonitoringStats, String> {
    debug!("Frontend requested monitoring statistics");
    let state = app.state::<AppState>();
    get_monitoring_stats(&*state).await
}

#[command]
#[instrument(skip(app))]
async fn monitor_sessions_once_command(app: AppHandle) -> Result<SessionMonitoringStats, String> {
    info!("Frontend requested one-time session monitoring check");
    let state = app.state::<AppState>();
    monitor_active_sessions(&*state).await
}

// === OIDC AUTHENTICATION COMMANDS ===

// Initiate OIDC authentication flow
#[command]
#[instrument(skip(app))]
async fn initiate_oidc_auth_command(
    app: AppHandle,
    auth_request: OIDCAuthRequest
) -> Result<OIDCAuthProgress, String> {
    info!("Frontend requested OIDC authentication for server {} with auth method {}",
          auth_request.server_id, auth_request.auth_method_id);

    let state = app.state::<AppState>();

    // Find the server configuration
    let servers = load_servers(app.clone()).await?;
    let server = servers
        .iter()
        .find(|s| s.id == auth_request.server_id)
        .ok_or_else(|| format!("Server with id '{}' not found", auth_request.server_id))?;

    let cli_path = get_boundary_cli_path(server, &state.config);

    // Start OIDC authentication in background
    let server_id = auth_request.server_id.clone();
    let auth_method_id = auth_request.auth_method_id.clone();
    let server_url = server.url.clone();

    tokio::spawn(async move {
        info!("Starting background OIDC authentication for server {}", server_id);

        match trigger_reauthentication(&cli_path, &server_url, &auth_method_id, &server_id).await {
            Ok(token) => {
                info!("OIDC authentication completed successfully for server {}", server_id);
                // Store the token
                if let Err(e) = store_auth_token(&token).await {
                    error!("Failed to store authentication token: {}", e);
                }
            }
            Err(e) => {
                error!("OIDC authentication failed for server {}: {}", server_id, e);
            }
        }
    });

    // Return initial progress
    Ok(OIDCAuthProgress {
        status: "started".to_string(),
        message: "Initiating OIDC authentication...".to_string(),
        auth_url: None,
        progress_percent: 10,
    })
}

// Check OIDC authentication status
#[command]
#[instrument(skip(app))]
async fn check_oidc_auth_status_command(
    app: AppHandle,
    server_id: String,
    user_id: String
) -> Result<OIDCAuthResult, String> {
    info!("Frontend checking OIDC authentication status for user {} on server {}", user_id, server_id);

    // Check if we have a stored token
    match retrieve_auth_token(&server_id, &user_id).await {
        Ok(token) => {
            info!("Found stored authentication token for user {} on server {}", user_id, server_id);

            // Check if we need to discover scopes
            let state = app.state::<AppState>();
            let servers = load_servers(app.clone()).await?;
            let server = servers
                .iter()
                .find(|s| s.id == server_id)
                .ok_or_else(|| format!("Server with id '{}' not found", server_id))?;

            let cli_path = get_boundary_cli_path(server, &state.config);

            // Discover available scopes
            match discover_scopes(&cli_path, &server.url).await {
                Ok(scopes) => {
                    Ok(OIDCAuthResult {
                        success: true,
                        token: Some(token),
                        error: None,
                        scopes: Some(scopes),
                    })
                }
                Err(e) => {
                    warn!("Failed to discover scopes but token exists: {}", e);
                    Ok(OIDCAuthResult {
                        success: true,
                        token: Some(token),
                        error: None,
                        scopes: None,
                    })
                }
            }
        }
        Err(_) => {
            Ok(OIDCAuthResult {
                success: false,
                token: None,
                error: Some("No authentication token found".to_string()),
                scopes: None,
            })
        }
    }
}

// Complete OIDC authentication workflow with scope selection
#[command]
#[instrument(skip(app))]
async fn complete_oidc_auth_workflow_command(
    app: AppHandle,
    server_id: String,
    user_id: String,
    scope_id: Option<String>
) -> Result<OIDCAuthResult, String> {
    info!("Frontend completing OIDC workflow for user {} on server {} with scope {:?}",
          user_id, server_id, scope_id);

    // Retrieve the current token
    let mut token = retrieve_auth_token(&server_id, &user_id).await?;

    // Update the scope if provided
    if let Some(scope) = scope_id {
        token.scope_id = scope;
        store_auth_token(&token).await?;
    }

    Ok(OIDCAuthResult {
        success: true,
        token: Some(token),
        error: None,
        scopes: None,
    })
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

            // Store configuration and initialize connection tracking and monitoring in app state
            app.manage(AppState {
                config,
                active_connections: Arc::new(Mutex::new(Vec::new())),
                session_health: Arc::new(Mutex::new(HashMap::new())),
                monitoring_enabled: Arc::new(Mutex::new(false)),
            });

            info!("Application initialization completed successfully");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            load_servers,
            log_from_frontend,
            get_config,
            verify_boundary_cli_command,
            discover_auth_methods_command,
            discover_scopes_command,
            discover_oidc_auth_methods_command,
            verify_oidc_support_command,
            discover_targets_command,
            discover_all_targets_command,
            authorize_session_command,
            establish_connection_command,
            get_active_connections_command,
            terminate_connection_command,
            detect_rdp_clients_command,
            launch_rdp_client_command,
            store_auth_token_command,
            retrieve_auth_token_command,
            delete_auth_token_command,
            token_exists_command,
            list_stored_tokens_command,
            refresh_auth_token_command,
            validate_token_command,
            check_and_refresh_token_command,
            start_token_auto_refresh_command,
            auto_logout_expired_tokens_command,
            initiate_oidc_auth_command,
            check_oidc_auth_status_command,
            complete_oidc_auth_workflow_command,
            start_session_monitoring_command,
            stop_session_monitoring_command,
            get_session_health_command,
            get_monitoring_stats_command,
            monitor_sessions_once_command
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}