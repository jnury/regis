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

// Handle connect button click - starts OIDC authentication
async function handleConnect() {
    if (!selectedServer) {
        showError('Please select a server first');
        return;
    }

    await logger.info('Starting authentication for server', 'auth', { server: selectedServer.name });

    try {
        // Update UI to show authentication in progress
        updateConnectButtonForAuth(true);
        hideError();

        // First, discover OIDC auth methods for this server
        await logger.info('Discovering OIDC auth methods', 'auth');
        const authMethods = await window.__TAURI__.core.invoke('discover_oidc_auth_methods_command', {
            serverAddr: selectedServer.url
        });

        if (!authMethods || authMethods.length === 0) {
            throw new Error('No OIDC authentication methods available for this server');
        }

        // Use the first available auth method
        const authMethod = authMethods[0];
        await logger.info('Using auth method', 'auth', { methodId: authMethod.id });

        // Start OIDC authentication
        const authRequest = {
            server_id: selectedServer.id,
            auth_method_id: authMethod.id,
            scope_id: null
        };

        const authProgress = await window.__TAURI__.core.invoke('initiate_oidc_auth_command', {
            authRequest: authRequest
        });

        await logger.info('OIDC authentication initiated', 'auth', { status: authProgress.status });

        // Monitor authentication progress
        await monitorAuthenticationProgress(selectedServer.id, 'user'); // TODO: Get actual user ID

    } catch (error) {
        await logger.error('Authentication failed', 'auth', { error: error.message });
        showError(`Authentication failed: ${error.message || error}`);
        updateConnectButtonForAuth(false);
    }
}

// Monitor authentication progress and handle completion
async function monitorAuthenticationProgress(serverId, userId) {
    const maxAttempts = 30; // 30 seconds timeout
    const checkInterval = 1000; // 1 second

    for (let attempt = 0; attempt < maxAttempts; attempt++) {
        try {
            const authResult = await window.__TAURI__.core.invoke('check_oidc_auth_status_command', {
                serverId: serverId,
                userId: userId
            });

            if (authResult.success && authResult.token) {
                await logger.info('Authentication completed successfully', 'auth');

                // Check if scope selection is needed
                if (authResult.scopes && authResult.scopes.length > 1) {
                    await handleScopeSelection(serverId, userId, authResult.scopes);
                } else {
                    // Single scope or no scopes - complete authentication
                    const scopeId = authResult.scopes && authResult.scopes.length === 1
                        ? authResult.scopes[0].id
                        : null;

                    await completeAuthentication(serverId, userId, scopeId);
                }
                return;
            }

            // Wait before next check
            await new Promise(resolve => setTimeout(resolve, checkInterval));

        } catch (error) {
            await logger.error('Error checking authentication status', 'auth', { error: error.message });
            // Continue checking - might be temporary error
        }
    }

    // Authentication timed out
    await logger.warn('Authentication timed out', 'auth');
    showError('Authentication timed out. Please try again.');
    updateConnectButtonForAuth(false);
}

// Handle scope selection when multiple scopes are available
async function handleScopeSelection(serverId, userId, scopes) {
    await logger.info('Multiple scopes available, showing selection UI', 'auth', { scopeCount: scopes.length });

    // Create scope selection UI
    const scopeSelectionHTML = `
        <div class="scope-selection">
            <h3>Select Scope</h3>
            <p>Multiple scopes are available. Please select one:</p>
            <div class="scope-list">
                ${scopes.map(scope => `
                    <div class="scope-item" data-scope-id="${scope.id}">
                        <div class="scope-name">${escapeHtml(scope.name)}</div>
                        <div class="scope-description">${escapeHtml(scope.description || '')}</div>
                    </div>
                `).join('')}
            </div>
            <div class="scope-actions">
                <button id="cancel-scope-selection">Cancel</button>
            </div>
        </div>
    `;

    // Replace server list with scope selection
    serverListElement.innerHTML = scopeSelectionHTML;

    // Add event listeners for scope selection
    serverListElement.querySelectorAll('.scope-item').forEach(element => {
        element.addEventListener('click', async () => {
            const scopeId = element.getAttribute('data-scope-id');
            await completeAuthentication(serverId, userId, scopeId);
        });
    });

    // Add cancel handler
    document.getElementById('cancel-scope-selection').addEventListener('click', () => {
        loadServers(); // Reload server list
        updateConnectButtonForAuth(false);
    });
}

// Complete authentication workflow
async function completeAuthentication(serverId, userId, scopeId) {
    try {
        await logger.info('Completing authentication workflow', 'auth', { serverId, scopeId });

        const authResult = await window.__TAURI__.core.invoke('complete_oidc_auth_workflow_command', {
            serverId: serverId,
            userId: userId,
            scopeId: scopeId
        });

        if (authResult.success) {
            await logger.info('Authentication workflow completed successfully', 'auth');

            // Show success and transition to main application
            showAuthenticationSuccess();
        } else {
            throw new Error(authResult.error || 'Authentication completion failed');
        }

    } catch (error) {
        await logger.error('Failed to complete authentication', 'auth', { error: error.message });
        showError(`Failed to complete authentication: ${error.message || error}`);
        updateConnectButtonForAuth(false);
    }
}

// Update connect button for authentication state
function updateConnectButtonForAuth(isAuthenticating) {
    if (isAuthenticating) {
        connectButton.disabled = true;
        connectButton.textContent = 'Authenticating...';
    } else {
        connectButton.disabled = !selectedServer;
        connectButton.textContent = selectedServer
            ? `Connect to ${selectedServer.name}`
            : 'Connect to Server';
    }
}

// Show authentication success
function showAuthenticationSuccess() {
    const successHTML = `
        <div class="auth-success">
            <h2>Authentication Successful!</h2>
            <p>You have successfully authenticated to ${selectedServer.name}.</p>
            <button id="continue-to-app">Continue to Application</button>
        </div>
    `;

    serverListElement.innerHTML = successHTML;
    updateConnectButtonForAuth(false);

    // Add continue handler
    document.getElementById('continue-to-app').addEventListener('click', () => {
        // TODO: Transition to main application view
        alert('Transitioning to main application...');
    });
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