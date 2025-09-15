// Global setup for E2E tests
import fs from 'fs/promises';
import path from 'path';
import { fileURLToPath } from 'url';
import { dirname } from 'path';

// ES module equivalent of __dirname
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

async function globalSetup() {
  console.log('Setting up E2E test environment...');

  // Ensure test config directory exists
  const testConfigDir = path.join(__dirname, '../../test-config');
  await fs.mkdir(testConfigDir, { recursive: true });

  // Create test configuration file
  const testConfig = {
    version: "0.1.0",
    application: {
      name: "Regis Test",
      window: {
        width: 1000,
        height: 700,
        min_width: 600,
        min_height: 400,
        resizable: true,
        center: true,
        title: "Regis - E2E Test"
      },
      system_tray: {
        enabled: false, // Disable for testing
        minimize_to_tray: false,
        show_notifications: false
      },
      auto_connect: {
        single_target: false, // Disable auto-connect for testing
        remember_last_server: false
      }
    },
    boundary: {
      cli_path: "boundary",
      cli_timeout_seconds: 30,
      connection_timeout_seconds: 60,
      token_refresh_threshold_minutes: 5
    },
    servers: [
      {
        id: "test-server-1",
        name: "Test Server 1",
        description: "First test server for E2E testing",
        url: "https://boundary-test1.example.com",
        enabled: true,
        oidc: {
          auto_discover: true,
          discovery_url: "",
          client_id: "test-client-1",
          scopes: ["openid", "profile"],
          provider_hints: {
            name: "Test Provider 1",
            type: "oidc",
            logo_url: null
          }
        },
        advanced: {
          verify_ssl: true,
          custom_ca_path: null,
          proxy_url: null,
          headers: {}
        }
      },
      {
        id: "test-server-2",
        name: "Test Server 2",
        description: "Second test server for E2E testing",
        url: "https://boundary-test2.example.com",
        enabled: true,
        oidc: {
          auto_discover: true,
          discovery_url: "",
          client_id: "test-client-2",
          scopes: ["openid"],
          provider_hints: {
            name: "Test Provider 2",
            type: "ping",
            logo_url: null
          }
        },
        advanced: {
          verify_ssl: false,
          custom_ca_path: null,
          proxy_url: null,
          headers: {}
        }
      }
    ],
    logging: {
      level: "debug",
      file_path: null,
      console: true
    },
    rdp: {
      clients: {
        windows: {
          executable: "mstsc",
          args: ["/v:{host}:{port}"],
          auto_detect: true,
          preferred_apps: []
        },
        macos: {
          executable: "open",
          args: ["rdp://{host}:{port}"],
          auto_detect: true,
          preferred_apps: ["Microsoft Remote Desktop"]
        }
      },
      connection: {
        fullscreen: false,
        resolution: "1920x1080",
        color_depth: 32
      }
    },
    ui: {
      theme: "light", // Use light theme for consistent testing
      show_connection_details: true,
      show_server_descriptions: true,
      compact_mode: false
    },
    security: {
      store_tokens_in_keychain: false, // Disable for testing
      auto_logout_minutes: 60,
      require_confirmation_for_connections: false
    }
  };

  // Write test configuration
  const configPath = path.join(testConfigDir, 'default.json');
  await fs.writeFile(configPath, JSON.stringify(testConfig, null, 2));

  console.log(`Test configuration written to: ${configPath}`);

  // Set environment variable for test config path
  process.env.REGIS_TEST_CONFIG_DIR = testConfigDir;

  console.log('E2E test environment setup complete');
}

export default globalSetup;