use tauri::{Manager, State};
use log::{info, error, debug};
use std::sync::Mutex;

pub mod config;
pub mod boundary;
pub mod oidc;
mod tray;
mod logger;

use config::{ConfigManager, BoundaryServer};

// Application state
pub struct AppState {
    config_manager: Mutex<ConfigManager>,
}

// Tauri commands
#[tauri::command]
async fn load_server_config(state: State<'_, AppState>) -> Result<Vec<BoundaryServer>, String> {
    info!("[BACKEND] load_server_config called - JavaScript is executing!");

    let config_manager = state.config_manager.lock().map_err(|e| {
        error!("[BACKEND] Failed to acquire config manager lock: {}", e);
        "Internal error: Failed to access configuration".to_string()
    })?;

    let servers: Vec<BoundaryServer> = config_manager.get_enabled_servers()
        .into_iter()
        .cloned()
        .collect();

    info!("[BACKEND] Loaded {} enabled servers", servers.len());
    for server in &servers {
        debug!("[BACKEND] Server: {} - {} ({})", server.id, server.name, server.url);
    }

    Ok(servers)
}

#[tauri::command]
async fn get_platform() -> Result<String, String> {
    Ok(std::env::consts::OS.to_string())
}

#[tauri::command]
async fn discover_oidc_config(server_url: String) -> Result<serde_json::Value, String> {
    debug!("Discovering OIDC configuration for: {}", server_url);

    // This will be implemented in the OIDC module
    oidc::discover_oidc_configuration(&server_url).await
        .map_err(|e| {
            error!("OIDC discovery failed for {}: {}", server_url, e);
            format!("Failed to discover OIDC configuration: {}", e)
        })
}

#[tauri::command]
async fn start_oidc_auth(server_url: String, oidc_config: serde_json::Value) -> Result<serde_json::Value, String> {
    debug!("Starting OIDC authentication for: {}", server_url);

    // This will be implemented in the OIDC module
    oidc::start_oidc_authentication(&server_url, &oidc_config).await
        .map_err(|e| {
            error!("OIDC authentication failed for {}: {}", server_url, e);
            format!("Authentication failed: {}", e)
        })
}

#[tauri::command]
async fn list_targets(server_url: String, token: String) -> Result<Vec<serde_json::Value>, String> {
    debug!("Listing targets for server: {}", server_url);

    // This will be implemented in the boundary module
    boundary::list_targets(&server_url, &token).await
        .map_err(|e| {
            error!("Failed to list targets for {}: {}", server_url, e);
            format!("Failed to retrieve targets: {}", e)
        })
}

#[tauri::command]
async fn connect_to_target(server_id: String, target_id: String, token: String) -> Result<serde_json::Value, String> {
    debug!("Connecting to target: {} on server: {}", target_id, server_id);

    // This will be implemented in the boundary module
    boundary::connect_to_target(&server_id, &target_id, &token).await
        .map_err(|e| {
            error!("Failed to connect to target {} on {}: {}", target_id, server_id, e);
            format!("Connection failed: {}", e)
        })
}

#[tauri::command]
async fn launch_rdp_client(connection_details: serde_json::Value) -> Result<(), String> {
    debug!("Launching RDP client with connection details");

    // This will be implemented in the boundary module
    boundary::launch_rdp_client(&connection_details).await
        .map_err(|e| {
            error!("Failed to launch RDP client: {}", e);
            format!("Failed to launch RDP client: {}", e)
        })
}

#[tauri::command]
async fn cleanup_connections() -> Result<(), String> {
    debug!("Cleaning up connections");

    // This will be implemented in the boundary module
    boundary::cleanup_connections().await
        .map_err(|e| {
            error!("Failed to cleanup connections: {}", e);
            format!("Failed to cleanup connections: {}", e)
        })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize configuration first
    let config_manager = match ConfigManager::new() {
        Ok(manager) => manager,
        Err(e) => {
            eprintln!("[BACKEND] Failed to initialize configuration: {}", e);
            panic!("Cannot start application without valid configuration: {}", e);
        }
    };

    // Initialize comprehensive file logging
    let logging_config = config_manager.get_config().logging.clone();
    if let Err(e) = logger::Logger::setup_logging(&logging_config) {
        eprintln!("[BACKEND] Failed to setup file logging: {}", e);
        // Fallback to basic logging
        env_logger::init();
        error!("[BACKEND] Falling back to basic logging due to setup error: {}", e);
    } else {
        info!("[BACKEND] File logging initialized successfully");
    }

    info!("[BACKEND] Starting Regis application");

    info!("[BACKEND] Configuration loaded successfully");

    let app_state = AppState {
        config_manager: Mutex::new(config_manager),
    };

    info!("[BACKEND] About to create Tauri builder...");
    let builder = tauri::Builder::default()
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            load_server_config,
            get_platform,
            discover_oidc_config,
            start_oidc_auth,
            list_targets,
            connect_to_target,
            launch_rdp_client,
            cleanup_connections,
            logger::log_frontend_debug,
            logger::log_frontend_info,
            logger::log_frontend_warn,
            logger::log_frontend_error,
            logger::get_log_files_list,
        ])
        .setup(|app| {
            info!("[BACKEND] Setting up Tauri application");
            info!("[BACKEND] Tauri application window should be visible now");

            // System tray will be set up later when we implement it
            info!("[BACKEND] Application setup completed");
            Ok(())
        });

    info!("[BACKEND] About to run Tauri application...");
    builder.run(tauri::generate_context!())
        .expect("error while running tauri application");
}
