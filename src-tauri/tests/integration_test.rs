// Integration tests for individual modules and their functionality
// Note: Tauri 2.x doesn't provide the same test mocking utilities as v1
// These tests focus on unit testing individual modules directly

use serde_json::json;
use tempfile::TempDir;

// Import what's available from the main lib crate
use regis_lib;

#[tokio::test]
async fn test_config_manager_integration() {
    // Test ConfigManager with a complete test configuration
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().to_path_buf();
    let default_config_path = config_dir.join("default.json");

    let config_json = json!({
        "version": "0.1.0",
        "application": {
            "name": "Test App",
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
                "id": "test-server-1",
                "name": "Test Server 1",
                "description": "First test server",
                "url": "https://boundary1.example.com",
                "enabled": true,
                "oidc": {
                    "auto_discover": true,
                    "discovery_url": "",
                    "client_id": "test-client-1",
                    "scopes": ["openid"],
                    "provider_hints": {
                        "name": "Test Provider 1",
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
                "id": "test-server-2",
                "name": "Test Server 2",
                "description": "Second test server (disabled)",
                "url": "https://boundary2.example.com",
                "enabled": false,
                "oidc": {
                    "auto_discover": true,
                    "discovery_url": "",
                    "client_id": "test-client-2",
                    "scopes": ["openid", "profile"],
                    "provider_hints": {
                        "name": "Test Provider 2",
                        "type": "ping",
                        "logo_url": null
                    }
                },
                "advanced": {
                    "verify_ssl": false,
                    "custom_ca_path": null,
                    "proxy_url": null,
                    "headers": {}
                }
            }
        ],
        "logging": {
            "level": "debug",
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
            "theme": "light",
            "show_connection_details": true,
            "show_server_descriptions": true,
            "compact_mode": false
        },
        "security": {
            "store_tokens_in_keychain": false,
            "auto_logout_minutes": 60,
            "require_confirmation_for_connections": false
        }
    });

    std::fs::write(&default_config_path, serde_json::to_string_pretty(&config_json).unwrap()).unwrap();

    // Create ConfigManager with test paths
    let config_manager = regis_lib::config::ConfigManager::new_with_paths(
        config_dir.clone(),
        default_config_path,
        config_dir.join("user.json")
    ).unwrap();

    // Test the get_enabled_servers method
    let enabled_servers = config_manager.get_enabled_servers();
    assert_eq!(enabled_servers.len(), 1, "Should have exactly 1 enabled server");
    assert_eq!(enabled_servers[0].id, "test-server-1");
    assert_eq!(enabled_servers[0].name, "Test Server 1");
    assert!(enabled_servers[0].enabled);

    // Test that we can access the config and check total server count
    let config = config_manager.get_config();
    assert_eq!(config.servers.len(), 2, "Should have 2 servers total");

    // Test server filtering logic by checking the disabled server exists
    let server_2 = config_manager.get_server_by_id("test-server-2");
    assert!(server_2.is_some(), "Should find test-server-2");
    assert!(!server_2.unwrap().enabled, "test-server-2 should be disabled");
}

#[tokio::test]
async fn test_platform_detection() {
    // Test platform detection utility function
    let platform = std::env::consts::OS;

    // Verify we get expected platform strings
    assert!(matches!(platform, "macos" | "windows" | "linux"), "Platform should be one of the supported OS types");

    // Test that we can detect the current platform
    #[cfg(target_os = "macos")]
    assert_eq!(platform, "macos");

    #[cfg(target_os = "windows")]
    assert_eq!(platform, "windows");

    #[cfg(target_os = "linux")]
    assert_eq!(platform, "linux");
}

#[cfg(test)]
mod boundary_module_tests {
    use super::*;

    #[tokio::test]
    async fn test_boundary_mock_functions() {
        // Test that boundary module functions return expected mock data
        // These tests validate the mock implementations work correctly

        // Test list_targets mock function
        let targets = regis_lib::boundary::list_targets("https://boundary.example.com", "mock-token")
            .await
            .expect("list_targets should return mock data successfully");

        assert!(targets.len() >= 3, "Should have at least 3 mock targets");

        // Check target types are present and valid
        let target_types: Vec<&str> = targets
            .iter()
            .filter_map(|t| t.get("type")?.as_str())
            .collect();

        assert!(target_types.contains(&"tcp"), "Should include TCP targets");
        assert!(target_types.contains(&"ssh"), "Should include SSH targets");
        assert!(target_types.contains(&"rdp"), "Should include RDP targets");

        // Verify target structure
        for target in &targets {
            assert!(target.get("id").is_some(), "Each target should have an id");
            assert!(target.get("name").is_some(), "Each target should have a name");
            assert!(target.get("type").is_some(), "Each target should have a type");
        }

        // Test connect_to_target mock function
        let connection = regis_lib::boundary::connect_to_target("test-server", "test-target", "mock-token")
            .await
            .expect("connect_to_target should return mock data successfully");

        assert_eq!(connection.get("success").unwrap(), &json!(true), "Connection should be successful");
        assert!(connection.get("session_id").is_some(), "Connection should have session_id");

        let connection_details = connection.get("connection").unwrap();
        assert!(connection_details.get("host").is_some(), "Connection should have host");
        assert!(connection_details.get("port").is_some(), "Connection should have port");
        assert!(connection_details.get("protocol").is_some(), "Connection should have protocol");
    }

    #[tokio::test]
    async fn test_boundary_error_handling() {
        // Test error handling in boundary functions

        // Test with invalid server URL
        let _result = regis_lib::boundary::list_targets("invalid-url", "token").await;
        // Note: Since these are mock functions, they might still succeed
        // In a real implementation, this would test actual error conditions

        // Test with empty token
        let _result = regis_lib::boundary::list_targets("https://boundary.example.com", "").await;
        // Again, mock functions may not reflect real error conditions

        // These tests validate that the mock functions can be called with various inputs
        // In a real implementation, we would test actual error scenarios
    }
}

#[cfg(test)]
mod oidc_module_tests {
    use super::*;

    #[tokio::test]
    async fn test_oidc_discovery_mock() {
        // Test OIDC discovery returns proper mock configuration
        let oidc_config = regis_lib::oidc::discover_oidc_configuration("https://boundary.example.com")
            .await
            .expect("OIDC discovery should return mock configuration");

        // Verify required OIDC fields are present
        assert!(oidc_config.get("provider_name").is_some(), "Should have provider_name");
        assert!(oidc_config.get("issuer").is_some(), "Should have issuer");
        assert!(oidc_config.get("authorization_endpoint").is_some(), "Should have authorization_endpoint");
        assert!(oidc_config.get("token_endpoint").is_some(), "Should have token_endpoint");
        assert!(oidc_config.get("userinfo_endpoint").is_some(), "Should have userinfo_endpoint");
        assert!(oidc_config.get("jwks_uri").is_some(), "Should have jwks_uri");

        // Verify scopes are properly structured
        if let Some(scopes) = oidc_config.get("scopes_supported") {
            let scopes_array = scopes.as_array().expect("scopes_supported should be an array");
            assert!(scopes_array.contains(&json!("openid")), "Should support openid scope");
            assert!(scopes_array.contains(&json!("profile")), "Should support profile scope");
            assert!(scopes_array.contains(&json!("email")), "Should support email scope");
        }
    }

    #[tokio::test]
    async fn test_oidc_authentication_mock() {
        // Test OIDC authentication returns proper mock result
        let mock_config = json!({
            "provider_name": "Test Provider",
            "issuer": "https://identity.example.com",
            "authorization_endpoint": "https://identity.example.com/auth",
            "token_endpoint": "https://identity.example.com/token"
        });

        let auth_result = regis_lib::oidc::start_oidc_authentication("https://boundary.example.com", &mock_config)
            .await
            .expect("OIDC authentication should return mock result");

        // Verify authentication result structure
        assert_eq!(auth_result.get("success").unwrap(), &json!(true), "Authentication should be successful");

        // Verify token format and presence
        let token = auth_result.get("token").unwrap().as_str().unwrap();
        assert!(token.starts_with("bt_"), "Token should start with 'bt_' prefix");
        assert!(token.len() > 10, "Token should have reasonable length");

        let refresh_token = auth_result.get("refresh_token").unwrap().as_str().unwrap();
        assert!(refresh_token.starts_with("rt_"), "Refresh token should start with 'rt_' prefix");

        assert!(auth_result.get("expires_at").is_some(), "Should have expiration time");

        // Verify user info structure
        let user_info = auth_result.get("user_info").unwrap();
        assert!(user_info.get("sub").is_some(), "User info should have subject");
        assert!(user_info.get("email").is_some(), "User info should have email");
        assert!(user_info.get("name").is_some(), "User info should have name");
    }

    #[tokio::test]
    async fn test_oidc_error_scenarios() {
        // Test OIDC functions with various input scenarios

        // Test discovery with different server URLs
        let configs = vec![
            "https://boundary1.example.com",
            "https://boundary2.example.com",
            "https://ping-identity.example.com"
        ];

        for server_url in configs {
            let result = regis_lib::oidc::discover_oidc_configuration(server_url).await;
            // Mock functions should handle various inputs gracefully
            assert!(result.is_ok(), "Discovery should work for various server URLs");
        }

        // Test authentication with minimal config
        let minimal_config = json!({
            "provider_name": "Minimal Provider"
        });

        let _result = regis_lib::oidc::start_oidc_authentication(
            "https://boundary.example.com",
            &minimal_config
        ).await;

        // Mock implementation should handle minimal configurations
        // In a real implementation, this might fail validation
    }
}