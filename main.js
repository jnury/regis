// Regis - Boundary GUI Client

// Logging system
class Logger {
    constructor() {
        this.component = 'frontend';
        this.isInitialized = false;
    }

    async init() {
        try {
            // Import Tauri API
            const { invoke } = await import('@tauri-apps/api/core');
            this.invoke = invoke;
            this.isInitialized = true;
            this.info('Frontend logging system initialized');
        } catch (error) {
            console.error('Failed to initialize frontend logging:', error);
            this.isInitialized = false;
        }
    }

    async log(level, message, component = null, data = null) {
        const comp = component || this.component;
        const timestamp = new Date().toISOString();

        // Always log to console
        const consoleMsg = `[${timestamp}] [${level.toUpperCase()}] [${comp}] ${message}`;
        if (data) {
            console.log(consoleMsg, data);
        } else {
            console.log(consoleMsg);
        }

        // Forward to backend if initialized
        if (this.isInitialized && this.invoke) {
            try {
                await this.invoke('log_from_frontend', {
                    level: level.toLowerCase(),
                    component: comp,
                    message: message,
                    data: data ? JSON.stringify(data) : null
                });
            } catch (error) {
                console.error('Failed to forward log to backend:', error);
            }
        }
    }

    async debug(message, component = null, data = null) {
        await this.log('debug', message, component, data);
    }

    async info(message, component = null, data = null) {
        await this.log('info', message, component, data);
    }

    async warn(message, component = null, data = null) {
        await this.log('warn', message, component, data);
    }

    async error(message, component = null, data = null) {
        await this.log('error', message, component, data);
    }
}

// Global logger instance
const logger = new Logger();

// Application state
let servers = [];
let selectedServer = null;
let appConfig = null;

// DOM elements
let serverListElement;
let connectButton;
let errorMessage;

// Load application configuration
async function loadAppConfig() {
    try {
        await logger.info('Loading application configuration', 'config');

        if (!window.__TAURI__) {
            await logger.warn('Tauri environment not available, using default config', 'config');
            return;
        }

        const { invoke } = await import('@tauri-apps/api/core');
        appConfig = await invoke('get_config');

        await logger.info('Application configuration loaded successfully', 'config', {
            logLevel: appConfig.logging.level,
            debugMode: appConfig.advanced.debug_mode
        });

        // Update logger behavior based on config
        if (appConfig.advanced.debug_mode) {
            await logger.info('Debug mode enabled', 'config');
        }

    } catch (error) {
        await logger.error('Failed to load application configuration', 'config', { error: error.message });
        // Continue with default behavior
    }
}

// Initialize application
document.addEventListener('DOMContentLoaded', async () => {
    try {
        // Initialize logging system first
        await logger.init();
        await logger.info('DOM loaded, initializing app', 'app');

        // Get DOM elements
        serverListElement = document.getElementById('server-list');
        connectButton = document.getElementById('connect-btn');
        errorMessage = document.getElementById('error-message');

        if (!serverListElement || !connectButton || !errorMessage) {
            await logger.error('Failed to find required DOM elements', 'app');
            throw new Error('Required DOM elements not found');
        }

        await logger.debug('DOM elements found successfully', 'app');

        // Load application configuration
        await loadAppConfig();

        // Set up event listeners
        connectButton.addEventListener('click', handleConnect);
        await logger.debug('Event listeners attached', 'app');

        // Load and display servers
        await loadServers();

        await logger.info('Application initialization completed successfully', 'app');
    } catch (error) {
        await logger.error('Application initialization failed', 'app', { error: error.message });
        showError(`Application initialization failed: ${error.message}`);
    }
});

// Load servers using Tauri command
async function loadServers() {
    try {
        console.log('Loading servers via Tauri backend...');

        // Check if running in Tauri environment
        if (!window.__TAURI__) {
            throw new Error('Tauri environment not available. Please run the app via Tauri.');
        }

        // Call the Rust backend command using global Tauri API
        servers = await window.__TAURI__.core.invoke('load_servers');

        console.log(`Loaded ${servers.length} servers:`, servers);

        if (servers.length === 0) {
            throw new Error('No servers found in configuration');
        }

        renderServerList();
        hideError();

    } catch (error) {
        console.error('Error loading servers:', error);
        showError(`Failed to load server configuration: ${error.message || error}`);
        renderEmptyServerList();
    }
}

// Render the server list
function renderServerList() {
    if (!servers || servers.length === 0) {
        renderEmptyServerList();
        return;
    }

    const serverItems = servers.map(server => createServerElement(server)).join('');
    serverListElement.innerHTML = serverItems;

    // Add click event listeners to server items
    serverListElement.querySelectorAll('.server-item').forEach((element, index) => {
        element.addEventListener('click', () => selectServer(servers[index]));
    });
}

// Create HTML element for a server
function createServerElement(server) {
    return `
        <div class="server-item" data-server-id="${server.id}">
            <div class="server-name">${escapeHtml(server.name)}</div>
            <div class="server-url">${escapeHtml(server.url)}</div>
            <div class="server-description">${escapeHtml(server.description)}</div>
            <div class="server-meta">
                <span class="server-environment ${server.environment}">${escapeHtml(server.environment)}</span>
                <span class="server-region">${escapeHtml(server.region)}</span>
            </div>
        </div>
    `;
}

// Render empty server list
function renderEmptyServerList() {
    serverListElement.innerHTML = '<div class="loading">No servers available</div>';
}

// Select a server
function selectServer(server) {
    console.log('Server selected:', server);

    // Update selected server
    selectedServer = server;

    // Update UI
    updateServerSelection(server.id);
    updateConnectButton();
}

// Update visual selection in server list
function updateServerSelection(serverId) {
    // Remove existing selection
    serverListElement.querySelectorAll('.server-item').forEach(element => {
        element.classList.remove('selected');
    });

    // Add selection to clicked server
    const selectedElement = serverListElement.querySelector(`[data-server-id="${serverId}"]`);
    if (selectedElement) {
        selectedElement.classList.add('selected');
    }
}

// Update connect button state
function updateConnectButton() {
    connectButton.disabled = !selectedServer;
    connectButton.textContent = selectedServer
        ? `Connect to ${selectedServer.name}`
        : 'Connect to Server';
}

// Handle connect button click
function handleConnect() {
    if (!selectedServer) {
        showError('Please select a server first');
        return;
    }

    console.log('Connecting to server:', selectedServer);

    // For now, just show an alert - this will be replaced with actual connection logic
    alert(`Connecting to ${selectedServer.name} at ${selectedServer.url}...`);
}

// Show error message
function showError(message) {
    errorMessage.textContent = message;
    errorMessage.style.display = 'block';
}

// Hide error message
function hideError() {
    errorMessage.style.display = 'none';
}

// Utility function to escape HTML
function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}