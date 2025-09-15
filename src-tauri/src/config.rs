use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use anyhow::{Result, Context};
use log::{info, warn, debug, error};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub version: String,
    pub application: ApplicationConfig,
    pub boundary: BoundaryConfig,
    pub servers: Vec<BoundaryServer>,
    pub logging: LoggingConfig,
    pub rdp: RdpConfig,
    pub ui: UiConfig,
    pub security: SecurityConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApplicationConfig {
    pub name: String,
    pub window: WindowConfig,
    pub system_tray: SystemTrayConfig,
    pub auto_connect: AutoConnectConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    pub width: u32,
    pub height: u32,
    pub min_width: u32,
    pub min_height: u32,
    pub resizable: bool,
    pub center: bool,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemTrayConfig {
    pub enabled: bool,
    pub minimize_to_tray: bool,
    pub show_notifications: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoConnectConfig {
    pub single_target: bool,
    pub remember_last_server: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundaryConfig {
    pub cli_path: String,
    pub cli_timeout_seconds: u64,
    pub connection_timeout_seconds: u64,
    pub token_refresh_threshold_minutes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundaryServer {
    pub id: String,
    pub name: String,
    pub description: String,
    pub url: String,
    pub enabled: bool,
    pub oidc: OidcConfig,
    pub advanced: AdvancedConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OidcConfig {
    pub auto_discover: bool,
    pub discovery_url: String,
    pub client_id: String,
    pub scopes: Vec<String>,
    pub provider_hints: ProviderHints,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderHints {
    pub name: String,
    pub r#type: String,
    pub logo_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdvancedConfig {
    pub verify_ssl: bool,
    pub custom_ca_path: Option<String>,
    pub proxy_url: Option<String>,
    pub headers: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file_path: Option<String>,
    pub console: bool,
    #[serde(default = "default_log_dir")]
    pub log_dir: String,
    #[serde(default = "default_max_log_size_mb")]
    pub max_log_size_mb: u64,
    #[serde(default = "default_file_rotation")]
    pub file_rotation: bool,
}

fn default_log_dir() -> String {
    "logs".to_string()
}

fn default_max_log_size_mb() -> u64 {
    10
}

fn default_file_rotation() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RdpConfig {
    pub clients: RdpClients,
    pub connection: RdpConnection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RdpClients {
    pub windows: RdpClient,
    pub macos: RdpClient,
    #[serde(default)]
    pub linux: Option<RdpClient>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RdpClient {
    pub executable: String,
    pub args: Vec<String>,
    pub auto_detect: bool,
    #[serde(default)]
    pub preferred_apps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RdpConnection {
    pub fullscreen: bool,
    pub resolution: String,
    pub color_depth: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub theme: String,
    pub show_connection_details: bool,
    #[serde(default)]
    pub show_server_descriptions: Option<bool>,
    pub compact_mode: bool,
    #[serde(default)]
    pub auto_refresh_targets: Option<bool>,
    #[serde(default)]
    pub refresh_interval_seconds: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    #[serde(alias = "token_storage", deserialize_with = "deserialize_token_storage")]
    pub store_tokens_in_keychain: bool,
    pub auto_logout_minutes: u64,
    #[serde(default)]
    pub require_confirmation_for_connections: Option<bool>,
    #[serde(default)]
    pub verify_certificates: Option<bool>,
    #[serde(default)]
    pub allowed_redirect_hosts: Option<Vec<String>>,
}

fn deserialize_token_storage<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de::Error;
    use serde_json::Value;

    let value: Value = serde::Deserialize::deserialize(deserializer)?;
    match value {
        Value::Bool(b) => Ok(b),
        Value::String(s) => match s.as_str() {
            "keychain" | "true" => Ok(true),
            "none" | "false" => Ok(false),
            _ => Err(D::Error::custom(format!("Invalid token storage value: {}", s)))
        },
        _ => Err(D::Error::custom("Token storage must be a boolean or string"))
    }
}

#[derive(Debug)]
pub struct ConfigManager {
    config: AppConfig,
    config_dir: PathBuf,
    default_config_path: PathBuf,
    user_config_path: PathBuf,
}

impl ConfigManager {
    pub fn new() -> Result<Self> {
        let config_dir = Self::get_config_directory()?;
        let default_config_path = config_dir.join("default.json");
        let user_config_path = config_dir.join("user.json");

        debug!("Config directory: {:?}", config_dir);
        debug!("Default config path: {:?}", default_config_path);
        debug!("User config path: {:?}", user_config_path);

        let config = Self::load_merged_config(&default_config_path, &user_config_path)?;

        Ok(ConfigManager {
            config,
            config_dir,
            default_config_path,
            user_config_path,
        })
    }

    pub fn new_with_paths(config_dir: PathBuf, default_config_path: PathBuf, user_config_path: PathBuf) -> Result<Self> {
        debug!("Config directory: {:?}", config_dir);
        debug!("Default config path: {:?}", default_config_path);
        debug!("User config path: {:?}", user_config_path);

        let config = Self::load_merged_config(&default_config_path, &user_config_path)?;

        Ok(ConfigManager {
            config,
            config_dir,
            default_config_path,
            user_config_path,
        })
    }

    fn get_config_directory() -> Result<PathBuf> {
        // In development, use the config/ directory relative to project root
        // In production, this would be in the app's data directory
        let exe_dir = std::env::current_exe()?
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Cannot determine executable directory"))?
            .to_path_buf();

        // Try to find config directory in several locations
        let possible_paths = vec![
            exe_dir.join("config"),
            exe_dir.join("../config"), // For development
            exe_dir.join("../../config"), // For Tauri dev mode
            exe_dir.join("../../../config"), // For deeper nesting
        ];

        for path in possible_paths {
            if path.exists() && path.is_dir() {
                debug!("Found config directory: {:?}", path);
                return Ok(path);
            }
        }

        // If no config directory found, create one in the app's data directory
        let app_data_dir = dirs::data_dir()
            .ok_or_else(|| anyhow::anyhow!("Cannot determine app data directory"))?
            .join("Regis");

        std::fs::create_dir_all(&app_data_dir)?;
        let config_dir = app_data_dir.join("config");
        std::fs::create_dir_all(&config_dir)?;

        info!("Created config directory: {:?}", config_dir);
        Ok(config_dir)
    }

    fn load_merged_config(default_path: &PathBuf, user_path: &PathBuf) -> Result<AppConfig> {
        // Try to load default configuration, fall back to built-in defaults if parsing fails
        let mut config: AppConfig = if default_path.exists() {
            match std::fs::read_to_string(default_path) {
                Ok(default_content) => {
                    match serde_json::from_str(&default_content) {
                        Ok(parsed_config) => {
                            info!("Loaded default configuration from {:?}", default_path);
                            parsed_config
                        }
                        Err(e) => {
                            error!("Failed to parse default configuration: {}", e);
                            return Err(anyhow::anyhow!("Failed to parse default configuration: {}", e));
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to read default config from {:?}: {}. Using built-in defaults.", default_path, e);
                    AppConfig::default()
                }
            }
        } else {
            info!("No default config found at {:?}, using built-in defaults", default_path);
            AppConfig::default()
        };

        // Load user configuration if it exists
        if user_path.exists() {
            let user_content = std::fs::read_to_string(user_path)
                .with_context(|| format!("Failed to read user config from {:?}", user_path))?;

            let user_config: serde_json::Value = serde_json::from_str(&user_content)
                .with_context(|| "Failed to parse user configuration")?;

            // Merge user config into default config
            let default_json = serde_json::to_value(&config)?;
            let merged_json = Self::merge_json_values(default_json, user_config);
            config = serde_json::from_value(merged_json)
                .with_context(|| "Failed to deserialize merged configuration")?;

            info!("Merged user configuration from {:?}", user_path);
        } else {
            info!("No user configuration found at {:?}", user_path);
        }

        // Validate configuration
        Self::validate_config(&config)?;

        Ok(config)
    }

    fn merge_json_values(default: serde_json::Value, user: serde_json::Value) -> serde_json::Value {
        use serde_json::Value;

        match (default, user) {
            (Value::Object(mut default_map), Value::Object(user_map)) => {
                for (key, user_value) in user_map {
                    let merged_value = match default_map.get(&key) {
                        Some(default_value) => {
                            Self::merge_json_values(default_value.clone(), user_value)
                        }
                        None => user_value,
                    };
                    default_map.insert(key, merged_value);
                }
                Value::Object(default_map)
            }
            (Value::Array(_default_array), Value::Array(user_array)) => {
                // For arrays, user config completely replaces default
                Value::Array(user_array)
            }
            (_, user_value) => {
                // For primitive values, user config completely replaces default
                user_value
            }
        }
    }

    fn validate_config(config: &AppConfig) -> Result<()> {
        // Validate servers
        if config.servers.is_empty() {
            warn!("No Boundary servers configured");
        }

        for server in &config.servers {
            if server.id.is_empty() {
                return Err(anyhow::anyhow!("Server ID cannot be empty"));
            }
            if server.url.is_empty() {
                return Err(anyhow::anyhow!("Server URL cannot be empty for server '{}'", server.id));
            }

            // Validate URL format
            url::Url::parse(&server.url)
                .with_context(|| format!("Invalid URL for server '{}': {}", server.id, server.url))?;
        }

        // Validate logging level
        match config.logging.level.to_lowercase().as_str() {
            "trace" | "debug" | "info" | "warn" | "error" => {},
            _ => return Err(anyhow::anyhow!("Invalid logging level: {}", config.logging.level)),
        }

        // Validate UI theme
        match config.ui.theme.to_lowercase().as_str() {
            "auto" | "light" | "dark" => {},
            _ => return Err(anyhow::anyhow!("Invalid UI theme: {}", config.ui.theme)),
        }

        debug!("Configuration validation passed");
        Ok(())
    }

    pub fn get_config(&self) -> &AppConfig {
        &self.config
    }

    pub fn get_enabled_servers(&self) -> Vec<&BoundaryServer> {
        self.config.servers.iter().filter(|s| s.enabled).collect()
    }

    pub fn get_server_by_id(&self, id: &str) -> Option<&BoundaryServer> {
        self.config.servers.iter().find(|s| s.id == id)
    }

    pub fn reload(&mut self) -> Result<()> {
        info!("Reloading configuration");
        self.config = Self::load_merged_config(&self.default_config_path, &self.user_config_path)?;
        Ok(())
    }

    pub fn get_config_dir(&self) -> &PathBuf {
        &self.config_dir
    }

    // Helper methods for commonly accessed config values
    pub fn get_boundary_cli_path(&self) -> &str {
        &self.config.boundary.cli_path
    }

    pub fn get_cli_timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.config.boundary.cli_timeout_seconds)
    }

    pub fn should_minimize_to_tray(&self) -> bool {
        self.config.application.system_tray.enabled &&
        self.config.application.system_tray.minimize_to_tray
    }

    pub fn should_auto_connect_single_target(&self) -> bool {
        self.config.application.auto_connect.single_target
    }

    pub fn get_rdp_client_config(&self) -> &RdpClient {
        #[cfg(target_os = "windows")]
        return &self.config.rdp.clients.windows;

        #[cfg(target_os = "macos")]
        return &self.config.rdp.clients.macos;

        #[cfg(not(any(target_os = "windows", target_os = "macos")))]
        &self.config.rdp.clients.windows // Fallback
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        // Provide sensible defaults in case config loading fails
        AppConfig {
            version: "0.1.0".to_string(),
            application: ApplicationConfig {
                name: "Regis".to_string(),
                window: WindowConfig {
                    width: 800,
                    height: 600,
                    min_width: 600,
                    min_height: 400,
                    resizable: true,
                    center: true,
                    title: "Regis - Boundary Client".to_string(),
                },
                system_tray: SystemTrayConfig {
                    enabled: true,
                    minimize_to_tray: true,
                    show_notifications: true,
                },
                auto_connect: AutoConnectConfig {
                    single_target: true,
                    remember_last_server: true,
                },
            },
            boundary: BoundaryConfig {
                cli_path: "boundary".to_string(),
                cli_timeout_seconds: 30,
                connection_timeout_seconds: 60,
                token_refresh_threshold_minutes: 5,
            },
            servers: vec![
                BoundaryServer {
                    id: "demo-boundary".to_string(),
                    name: "Demo Boundary Server".to_string(),
                    description: "Sample Boundary server for testing".to_string(),
                    url: "https://demo.boundary.io".to_string(),
                    enabled: true,
                    oidc: OidcConfig {
                        auto_discover: true,
                        discovery_url: "https://demo.boundary.io/.well-known/openid_configuration".to_string(),
                        client_id: "demo-client".to_string(),
                        scopes: vec!["openid".to_string(), "profile".to_string()],
                        provider_hints: ProviderHints {
                            name: "Demo Provider".to_string(),
                            r#type: "oidc".to_string(),
                            logo_url: None,
                        },
                    },
                    advanced: AdvancedConfig {
                        verify_ssl: true,
                        custom_ca_path: None,
                        proxy_url: None,
                        headers: HashMap::new(),
                    },
                },
                BoundaryServer {
                    id: "local-boundary".to_string(),
                    name: "Local Development".to_string(),
                    description: "Local Boundary server for development".to_string(),
                    url: "http://localhost:9200".to_string(),
                    enabled: true,
                    oidc: OidcConfig {
                        auto_discover: true,
                        discovery_url: "http://localhost:9200/.well-known/openid_configuration".to_string(),
                        client_id: "local-client".to_string(),
                        scopes: vec!["openid".to_string(), "profile".to_string()],
                        provider_hints: ProviderHints {
                            name: "Local Auth".to_string(),
                            r#type: "oidc".to_string(),
                            logo_url: None,
                        },
                    },
                    advanced: AdvancedConfig {
                        verify_ssl: false,
                        custom_ca_path: None,
                        proxy_url: None,
                        headers: HashMap::new(),
                    },
                },
            ],
            logging: LoggingConfig {
                level: "debug".to_string(),
                file_path: None,
                console: true,
                log_dir: "logs".to_string(),
                max_log_size_mb: 10,
                file_rotation: true,
            },
            rdp: RdpConfig {
                clients: RdpClients {
                    windows: RdpClient {
                        executable: "mstsc".to_string(),
                        args: vec!["/v:{host}:{port}".to_string()],
                        auto_detect: true,
                        preferred_apps: vec![],
                    },
                    macos: RdpClient {
                        executable: "open".to_string(),
                        args: vec!["rdp://{host}:{port}".to_string()],
                        auto_detect: true,
                        preferred_apps: vec![
                            "Microsoft Remote Desktop".to_string(),
                            "Royal TSX".to_string(),
                        ],
                    },
                    linux: Some(RdpClient {
                        executable: "rdesktop".to_string(),
                        args: vec!["-a".to_string(), "16".to_string(), "{host}:{port}".to_string()],
                        auto_detect: true,
                        preferred_apps: vec![
                            "rdesktop".to_string(),
                            "freerdp".to_string(),
                            "vinagre".to_string(),
                        ],
                    }),
                },
                connection: RdpConnection {
                    fullscreen: false,
                    resolution: "1920x1080".to_string(),
                    color_depth: 32,
                },
            },
            ui: UiConfig {
                theme: "auto".to_string(),
                show_connection_details: true,
                show_server_descriptions: Some(true),
                compact_mode: false,
                auto_refresh_targets: Some(true),
                refresh_interval_seconds: Some(30),
            },
            security: SecurityConfig {
                store_tokens_in_keychain: true,
                auto_logout_minutes: 480,
                require_confirmation_for_connections: Some(false),
                verify_certificates: Some(true),
                allowed_redirect_hosts: Some(vec![]),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    fn create_test_config_manager(default_json: &str, user_json: Option<&str>) -> Result<ConfigManager> {
        let temp_dir = TempDir::new()?;
        let config_dir = temp_dir.path().to_path_buf();
        let default_path = config_dir.join("default.json");
        let user_path = config_dir.join("user.json");

        fs::write(&default_path, default_json)?;

        if let Some(user_content) = user_json {
            fs::write(&user_path, user_content)?;
        }

        ConfigManager::new_with_paths(config_dir, default_path, user_path)
    }

    #[test]
    fn test_config_loading_with_valid_default() {
        let default_json = r#"{
            "version": "0.1.0",
            "application": {
                "name": "Test",
                "window": {
                    "width": 800,
                    "height": 600,
                    "min_width": 600,
                    "min_height": 400,
                    "resizable": true,
                    "center": true,
                    "title": "Test App"
                },
                "system_tray": {
                    "enabled": true,
                    "minimize_to_tray": false,
                    "show_notifications": true
                },
                "auto_connect": {
                    "single_target": true,
                    "remember_last_server": true
                }
            },
            "boundary": {
                "cli_path": "boundary",
                "cli_timeout_seconds": 30,
                "connection_timeout_seconds": 60,
                "token_refresh_threshold_minutes": 5
            },
            "servers": [{
                "id": "test-server",
                "name": "Test Server",
                "description": "Test description",
                "url": "https://boundary.example.com",
                "enabled": true,
                "oidc": {
                    "auto_discover": true,
                    "discovery_url": "",
                    "client_id": "test-client",
                    "scopes": ["openid"],
                    "provider_hints": {
                        "name": "Test Provider",
                        "type": "oidc",
                        "logo_url": null
                    }
                },
                "advanced": {
                    "verify_ssl": true,
                    "custom_ca_path": null,
                    "proxy_url": null,
                    "headers": {}
                }
            }],
            "logging": {
                "level": "info",
                "file_path": null,
                "console": true
            },
            "rdp": {
                "clients": {
                    "windows": {
                        "executable": "mstsc",
                        "args": ["/v:{host}:{port}"],
                        "auto_detect": true,
                        "preferred_apps": []
                    },
                    "macos": {
                        "executable": "open",
                        "args": ["rdp://{host}:{port}"],
                        "auto_detect": true,
                        "preferred_apps": []
                    }
                },
                "connection": {
                    "fullscreen": false,
                    "resolution": "1920x1080",
                    "color_depth": 32
                }
            },
            "ui": {
                "theme": "auto",
                "show_connection_details": true,
                "show_server_descriptions": true,
                "compact_mode": false
            },
            "security": {
                "store_tokens_in_keychain": true,
                "auto_logout_minutes": 480,
                "require_confirmation_for_connections": false
            }
        }"#;

        let config_manager = create_test_config_manager(default_json, None).unwrap();
        let config = config_manager.get_config();

        assert_eq!(config.version, "0.1.0");
        assert_eq!(config.servers.len(), 1);
        assert_eq!(config.servers[0].id, "test-server");
        assert_eq!(config.servers[0].name, "Test Server");
        assert_eq!(config.servers[0].enabled, true);
    }

    #[test]
    fn test_config_merging_user_overrides() {
        let default_json = r#"{
            "version": "0.1.0",
            "application": {
                "name": "Default App",
                "window": {
                    "width": 800,
                    "height": 600,
                    "min_width": 600,
                    "min_height": 400,
                    "resizable": true,
                    "center": true,
                    "title": "Default Title"
                },
                "system_tray": {
                    "enabled": true,
                    "minimize_to_tray": false,
                    "show_notifications": true
                },
                "auto_connect": {
                    "single_target": true,
                    "remember_last_server": true
                }
            },
            "boundary": {
                "cli_path": "boundary",
                "cli_timeout_seconds": 30,
                "connection_timeout_seconds": 60,
                "token_refresh_threshold_minutes": 5
            },
            "servers": [],
            "logging": {
                "level": "info",
                "file_path": null,
                "console": true
            },
            "rdp": {
                "clients": {
                    "windows": {
                        "executable": "mstsc",
                        "args": ["/v:{host}:{port}"],
                        "auto_detect": true,
                        "preferred_apps": []
                    },
                    "macos": {
                        "executable": "open",
                        "args": ["rdp://{host}:{port}"],
                        "auto_detect": true,
                        "preferred_apps": []
                    }
                },
                "connection": {
                    "fullscreen": false,
                    "resolution": "1920x1080",
                    "color_depth": 32
                }
            },
            "ui": {
                "theme": "auto",
                "show_connection_details": true,
                "show_server_descriptions": true,
                "compact_mode": false
            },
            "security": {
                "store_tokens_in_keychain": true,
                "auto_logout_minutes": 480,
                "require_confirmation_for_connections": false
            }
        }"#;

        let user_json = r#"{
            "application": {
                "window": {
                    "width": 1200,
                    "title": "Custom Title"
                }
            },
            "logging": {
                "level": "debug"
            }
        }"#;

        let config_manager = create_test_config_manager(default_json, Some(user_json)).unwrap();
        let config = config_manager.get_config();

        // User overrides should take effect
        assert_eq!(config.application.window.width, 1200);
        assert_eq!(config.application.window.title, "Custom Title");
        assert_eq!(config.logging.level, "debug");

        // Default values should remain for non-overridden fields
        assert_eq!(config.application.window.height, 600);
        assert_eq!(config.application.name, "Default App");
    }

    #[test]
    fn test_get_enabled_servers() {
        let default_json = r#"{
            "version": "0.1.0",
            "application": {
                "name": "Test",
                "window": {
                    "width": 800,
                    "height": 600,
                    "min_width": 600,
                    "min_height": 400,
                    "resizable": true,
                    "center": true,
                    "title": "Test"
                },
                "system_tray": {
                    "enabled": true,
                    "minimize_to_tray": false,
                    "show_notifications": true
                },
                "auto_connect": {
                    "single_target": true,
                    "remember_last_server": true
                }
            },
            "boundary": {
                "cli_path": "boundary",
                "cli_timeout_seconds": 30,
                "connection_timeout_seconds": 60,
                "token_refresh_threshold_minutes": 5
            },
            "servers": [
                {
                    "id": "enabled-server",
                    "name": "Enabled Server",
                    "description": "Test",
                    "url": "https://enabled.example.com",
                    "enabled": true,
                    "oidc": {
                        "auto_discover": true,
                        "discovery_url": "",
                        "client_id": "test",
                        "scopes": ["openid"],
                        "provider_hints": {
                            "name": "Test",
                            "type": "oidc",
                            "logo_url": null
                        }
                    },
                    "advanced": {
                        "verify_ssl": true,
                        "custom_ca_path": null,
                        "proxy_url": null,
                        "headers": {}
                    }
                },
                {
                    "id": "disabled-server",
                    "name": "Disabled Server",
                    "description": "Test",
                    "url": "https://disabled.example.com",
                    "enabled": false,
                    "oidc": {
                        "auto_discover": true,
                        "discovery_url": "",
                        "client_id": "test",
                        "scopes": ["openid"],
                        "provider_hints": {
                            "name": "Test",
                            "type": "oidc",
                            "logo_url": null
                        }
                    },
                    "advanced": {
                        "verify_ssl": true,
                        "custom_ca_path": null,
                        "proxy_url": null,
                        "headers": {}
                    }
                }
            ],
            "logging": {
                "level": "info",
                "file_path": null,
                "console": true
            },
            "rdp": {
                "clients": {
                    "windows": {
                        "executable": "mstsc",
                        "args": ["/v:{host}:{port}"],
                        "auto_detect": true,
                        "preferred_apps": []
                    },
                    "macos": {
                        "executable": "open",
                        "args": ["rdp://{host}:{port}"],
                        "auto_detect": true,
                        "preferred_apps": []
                    }
                },
                "connection": {
                    "fullscreen": false,
                    "resolution": "1920x1080",
                    "color_depth": 32
                }
            },
            "ui": {
                "theme": "auto",
                "show_connection_details": true,
                "show_server_descriptions": true,
                "compact_mode": false
            },
            "security": {
                "store_tokens_in_keychain": true,
                "auto_logout_minutes": 480,
                "require_confirmation_for_connections": false
            }
        }"#;

        let config_manager = create_test_config_manager(default_json, None).unwrap();
        let enabled_servers = config_manager.get_enabled_servers();

        assert_eq!(enabled_servers.len(), 1);
        assert_eq!(enabled_servers[0].id, "enabled-server");
        assert_eq!(enabled_servers[0].enabled, true);
    }

    #[test]
    fn test_get_server_by_id() {
        let default_json = r#"{
            "version": "0.1.0",
            "application": {
                "name": "Test",
                "window": {
                    "width": 800,
                    "height": 600,
                    "min_width": 600,
                    "min_height": 400,
                    "resizable": true,
                    "center": true,
                    "title": "Test"
                },
                "system_tray": {
                    "enabled": true,
                    "minimize_to_tray": false,
                    "show_notifications": true
                },
                "auto_connect": {
                    "single_target": true,
                    "remember_last_server": true
                }
            },
            "boundary": {
                "cli_path": "boundary",
                "cli_timeout_seconds": 30,
                "connection_timeout_seconds": 60,
                "token_refresh_threshold_minutes": 5
            },
            "servers": [{
                "id": "test-server",
                "name": "Test Server",
                "description": "Test",
                "url": "https://test.example.com",
                "enabled": true,
                "oidc": {
                    "auto_discover": true,
                    "discovery_url": "",
                    "client_id": "test",
                    "scopes": ["openid"],
                    "provider_hints": {
                        "name": "Test",
                        "type": "oidc",
                        "logo_url": null
                    }
                },
                "advanced": {
                    "verify_ssl": true,
                    "custom_ca_path": null,
                    "proxy_url": null,
                    "headers": {}
                }
            }],
            "logging": {
                "level": "info",
                "file_path": null,
                "console": true
            },
            "rdp": {
                "clients": {
                    "windows": {
                        "executable": "mstsc",
                        "args": ["/v:{host}:{port}"],
                        "auto_detect": true,
                        "preferred_apps": []
                    },
                    "macos": {
                        "executable": "open",
                        "args": ["rdp://{host}:{port}"],
                        "auto_detect": true,
                        "preferred_apps": []
                    }
                },
                "connection": {
                    "fullscreen": false,
                    "resolution": "1920x1080",
                    "color_depth": 32
                }
            },
            "ui": {
                "theme": "auto",
                "show_connection_details": true,
                "show_server_descriptions": true,
                "compact_mode": false
            },
            "security": {
                "store_tokens_in_keychain": true,
                "auto_logout_minutes": 480,
                "require_confirmation_for_connections": false
            }
        }"#;

        let config_manager = create_test_config_manager(default_json, None).unwrap();

        let server = config_manager.get_server_by_id("test-server");
        assert!(server.is_some());
        assert_eq!(server.unwrap().name, "Test Server");

        let non_existent = config_manager.get_server_by_id("non-existent");
        assert!(non_existent.is_none());
    }

    #[test]
    fn test_config_validation_invalid_url() {
        let invalid_config = r#"{
            "version": "0.1.0",
            "application": {
                "name": "Test",
                "window": {
                    "width": 800,
                    "height": 600,
                    "min_width": 600,
                    "min_height": 400,
                    "resizable": true,
                    "center": true,
                    "title": "Test"
                },
                "system_tray": {
                    "enabled": true,
                    "minimize_to_tray": false,
                    "show_notifications": true
                },
                "auto_connect": {
                    "single_target": true,
                    "remember_last_server": true
                }
            },
            "boundary": {
                "cli_path": "boundary",
                "cli_timeout_seconds": 30,
                "connection_timeout_seconds": 60,
                "token_refresh_threshold_minutes": 5
            },
            "servers": [{
                "id": "test-server",
                "name": "Test Server",
                "description": "Test",
                "url": "invalid-url",
                "enabled": true,
                "oidc": {
                    "auto_discover": true,
                    "discovery_url": "",
                    "client_id": "test",
                    "scopes": ["openid"],
                    "provider_hints": {
                        "name": "Test",
                        "type": "oidc",
                        "logo_url": null
                    }
                },
                "advanced": {
                    "verify_ssl": true,
                    "custom_ca_path": null,
                    "proxy_url": null,
                    "headers": {}
                }
            }],
            "logging": {
                "level": "info",
                "file_path": null,
                "console": true
            },
            "rdp": {
                "clients": {
                    "windows": {
                        "executable": "mstsc",
                        "args": ["/v:{host}:{port}"],
                        "auto_detect": true,
                        "preferred_apps": []
                    },
                    "macos": {
                        "executable": "open",
                        "args": ["rdp://{host}:{port}"],
                        "auto_detect": true,
                        "preferred_apps": []
                    }
                },
                "connection": {
                    "fullscreen": false,
                    "resolution": "1920x1080",
                    "color_depth": 32
                }
            },
            "ui": {
                "theme": "auto",
                "show_connection_details": true,
                "show_server_descriptions": true,
                "compact_mode": false
            },
            "security": {
                "store_tokens_in_keychain": true,
                "auto_logout_minutes": 480,
                "require_confirmation_for_connections": false
            }
        }"#;

        let result = create_test_config_manager(invalid_config, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid URL"));
    }

    #[test]
    fn test_config_validation_invalid_log_level() {
        let invalid_config = r#"{
            "version": "0.1.0",
            "application": {
                "name": "Test",
                "window": {
                    "width": 800,
                    "height": 600,
                    "min_width": 600,
                    "min_height": 400,
                    "resizable": true,
                    "center": true,
                    "title": "Test"
                },
                "system_tray": {
                    "enabled": true,
                    "minimize_to_tray": false,
                    "show_notifications": true
                },
                "auto_connect": {
                    "single_target": true,
                    "remember_last_server": true
                }
            },
            "boundary": {
                "cli_path": "boundary",
                "cli_timeout_seconds": 30,
                "connection_timeout_seconds": 60,
                "token_refresh_threshold_minutes": 5
            },
            "servers": [],
            "logging": {
                "level": "invalid",
                "file_path": null,
                "console": true
            },
            "rdp": {
                "clients": {
                    "windows": {
                        "executable": "mstsc",
                        "args": ["/v:{host}:{port}"],
                        "auto_detect": true,
                        "preferred_apps": []
                    },
                    "macos": {
                        "executable": "open",
                        "args": ["rdp://{host}:{port}"],
                        "auto_detect": true,
                        "preferred_apps": []
                    }
                },
                "connection": {
                    "fullscreen": false,
                    "resolution": "1920x1080",
                    "color_depth": 32
                }
            },
            "ui": {
                "theme": "auto",
                "show_connection_details": true,
                "show_server_descriptions": true,
                "compact_mode": false
            },
            "security": {
                "store_tokens_in_keychain": true,
                "auto_logout_minutes": 480,
                "require_confirmation_for_connections": false
            }
        }"#;

        let result = create_test_config_manager(invalid_config, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid logging level"));
    }

    #[test]
    fn test_helper_methods() {
        let default_json = r#"{
            "version": "0.1.0",
            "application": {
                "name": "Test",
                "window": {
                    "width": 800,
                    "height": 600,
                    "min_width": 600,
                    "min_height": 400,
                    "resizable": true,
                    "center": true,
                    "title": "Test"
                },
                "system_tray": {
                    "enabled": true,
                    "minimize_to_tray": true,
                    "show_notifications": true
                },
                "auto_connect": {
                    "single_target": false,
                    "remember_last_server": true
                }
            },
            "boundary": {
                "cli_path": "/usr/local/bin/boundary",
                "cli_timeout_seconds": 45,
                "connection_timeout_seconds": 60,
                "token_refresh_threshold_minutes": 5
            },
            "servers": [],
            "logging": {
                "level": "info",
                "file_path": null,
                "console": true
            },
            "rdp": {
                "clients": {
                    "windows": {
                        "executable": "mstsc",
                        "args": ["/v:{host}:{port}"],
                        "auto_detect": true,
                        "preferred_apps": []
                    },
                    "macos": {
                        "executable": "open",
                        "args": ["rdp://{host}:{port}"],
                        "auto_detect": true,
                        "preferred_apps": []
                    }
                },
                "connection": {
                    "fullscreen": false,
                    "resolution": "1920x1080",
                    "color_depth": 32
                }
            },
            "ui": {
                "theme": "auto",
                "show_connection_details": true,
                "show_server_descriptions": true,
                "compact_mode": false
            },
            "security": {
                "store_tokens_in_keychain": true,
                "auto_logout_minutes": 480,
                "require_confirmation_for_connections": false
            }
        }"#;

        let config_manager = create_test_config_manager(default_json, None).unwrap();

        assert_eq!(config_manager.get_boundary_cli_path(), "/usr/local/bin/boundary");
        assert_eq!(config_manager.get_cli_timeout(), std::time::Duration::from_secs(45));
        assert_eq!(config_manager.should_minimize_to_tray(), true);
        assert_eq!(config_manager.should_auto_connect_single_target(), false);
    }
}