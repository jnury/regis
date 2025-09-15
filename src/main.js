// Main application entry point
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { appWindow } from '@tauri-apps/api/window';
import logger from './logger.js';

// Application state
const AppState = {
    currentServer: null,
    isAuthenticated: false,
    currentToken: null,
    targets: [],
    connectionStatus: 'disconnected'
};

// DOM elements
const elements = {
    serverSelection: document.getElementById('server-selection'),
    serverList: document.getElementById('server-list'),
    authSection: document.getElementById('auth-section'),
    authContent: document.getElementById('auth-content'),
    targetsSection: document.getElementById('targets-section'),
    targetsList: document.getElementById('targets-list'),
    statusText: document.getElementById('status-text'),
    loadingSpinner: document.getElementById('loading-spinner')
};

// Utility functions
const Utils = {
    showElement(element) {
        element.classList.remove('hidden');
    },

    hideElement(element) {
        element.classList.add('hidden');
    },

    showLoading(message = 'Loading...') {
        elements.statusText.textContent = message;
        elements.loadingSpinner.classList.remove('hidden');
    },

    hideLoading() {
        elements.loadingSpinner.classList.add('hidden');
        elements.statusText.textContent = 'Ready';
    },

    showError(message, details = null) {
        const errorDiv = document.createElement('div');
        errorDiv.className = 'error-message';
        errorDiv.innerHTML = `
            <div>${message}</div>
            ${details ? `<div class="error-details">${details}</div>` : ''}
        `;

        // Insert at the beginning of main
        const main = document.querySelector('.main');
        main.insertBefore(errorDiv, main.firstChild);

        // Remove after 5 seconds
        setTimeout(() => {
            if (errorDiv.parentNode) {
                errorDiv.parentNode.removeChild(errorDiv);
            }
        }, 5000);
    },

    showSuccess(message) {
        const successDiv = document.createElement('div');
        successDiv.className = 'success-message';
        successDiv.textContent = message;

        const main = document.querySelector('.main');
        main.insertBefore(successDiv, main.firstChild);

        setTimeout(() => {
            if (successDiv.parentNode) {
                successDiv.parentNode.removeChild(successDiv);
            }
        }, 3000);
    }
};

// Server management
const ServerManager = {
    async loadServers() {
        try {
            await logger.info('Starting to load servers...');
            Utils.showLoading('Loading server configuration...');

            await logger.debug('Invoking load_server_config...');
            const servers = await invoke('load_server_config');
            await logger.debug('Received servers:', servers);

            this.renderServerList(servers);
            Utils.hideLoading();
        } catch (error) {
            await logger.logError(error, 'Failed to load servers');
            Utils.showError('Failed to load server configuration', error);
            Utils.hideLoading();
        }
    },

    renderServerList(servers) {
        await logger.debug('Rendering server list, element found:', !!elements.serverList);
        await logger.debug('Servers to render:', servers);

        if (!elements.serverList) {
            await logger.error('server-list element not found!');
            return;
        }

        elements.serverList.innerHTML = '';

        if (!servers || servers.length === 0) {
            await logger.info('No servers found, showing empty message');
            elements.serverList.innerHTML = `
                <div class="text-center text-muted">
                    <p>No Boundary servers configured.</p>
                    <p>Please add server configurations to the config file.</p>
                </div>
            `;
            return;
        }

        await logger.info(`Rendering ${servers.length} servers`);

        servers.forEach(server => {
            const serverItem = document.createElement('div');
            serverItem.className = 'server-item';
            serverItem.innerHTML = `
                <div class="server-item__name">${server.name}</div>
                <div class="server-item__url">${server.url}</div>
                <div class="server-item__description">${server.description || 'No description'}</div>
            `;

            serverItem.addEventListener('click', () => {
                this.selectServer(server, serverItem);
            });

            elements.serverList.appendChild(serverItem);
        });
    },

    selectServer(server, serverElement) {
        // Clear previous selection
        document.querySelectorAll('.server-item').forEach(item => {
            item.classList.remove('selected');
        });

        // Select current server
        serverElement.classList.add('selected');
        AppState.currentServer = server;

        // Add connect button if not exists
        let connectBtn = serverElement.querySelector('.connect-btn');
        if (!connectBtn) {
            connectBtn = document.createElement('button');
            connectBtn.className = 'btn btn--primary btn--small mt-2';
            connectBtn.textContent = 'Connect';
            connectBtn.addEventListener('click', (e) => {
                e.stopPropagation();
                this.connectToServer(server);
            });
            serverElement.appendChild(connectBtn);
        }
    },

    async connectToServer(server) {
        try {
            Utils.showLoading('Connecting to Boundary server...');

            // Hide server selection and show auth section
            Utils.hideElement(elements.serverSelection);
            Utils.showElement(elements.authSection);

            // Start authentication process
            await AuthManager.startAuthentication(server);

        } catch (error) {
            await logger.logError(error, 'ServerManager connectToServer failed');
            Utils.showError('Failed to connect to server', error);
            Utils.hideLoading();
        }
    }
};

// Authentication management
const AuthManager = {
    async startAuthentication(server) {
        try {
            elements.authContent.innerHTML = `
                <div class="auth-message">
                    <p>Starting OIDC authentication with ${server.name}...</p>
                    <p class="text-muted">Please wait while we discover the OIDC configuration.</p>
                </div>
            `;

            // Discover OIDC configuration
            const oidcConfig = await invoke('discover_oidc_config', { serverUrl: server.url });

            // Start OIDC flow
            await this.handleOidcFlow(server, oidcConfig);

        } catch (error) {
            await logger.logError(error, 'AuthManager startAuthentication failed');
            Utils.showError('Authentication failed', error);
            Utils.hideLoading();
        }
    },

    async handleOidcFlow(server, oidcConfig) {
        elements.authContent.innerHTML = `
            <div class="auth-message">
                <p>Authenticating with ${oidcConfig.provider_name || 'OIDC Provider'}...</p>
                <div class="auth-progress">
                    <div class="loading-spinner"></div>
                </div>
                <p class="text-muted">Complete the authentication in the embedded browser.</p>
            </div>
        `;

        try {
            // Start OIDC authentication
            const authResult = await invoke('start_oidc_auth', {
                serverUrl: server.url,
                oidcConfig: oidcConfig
            });

            if (authResult.success) {
                AppState.isAuthenticated = true;
                AppState.currentToken = authResult.token;

                Utils.showSuccess('Authentication successful!');
                Utils.hideElement(elements.authSection);

                // Load targets
                await TargetManager.loadTargets(server);
            } else {
                throw new Error(authResult.error || 'Authentication failed');
            }
        } catch (error) {
            await logger.logError(error, 'AuthManager handleOidcFlow failed');
            Utils.showError('Authentication failed', error);
            Utils.hideLoading();
        }
    }
};

// Target management
const TargetManager = {
    async loadTargets(server) {
        try {
            Utils.showLoading('Loading available targets...');

            const targets = await invoke('list_targets', {
                serverUrl: server.url,
                token: AppState.currentToken
            });

            AppState.targets = targets;

            if (targets.length === 1) {
                // Auto-connect to single target
                await this.connectToTarget(targets[0]);
            } else if (targets.length > 1) {
                // Show target selection
                this.renderTargetList(targets);
                Utils.showElement(elements.targetsSection);
            } else {
                Utils.showError('No targets available for your user');
            }

            Utils.hideLoading();
        } catch (error) {
            await logger.logError(error, 'TargetManager loadTargets failed');
            Utils.showError('Failed to load targets', error);
            Utils.hideLoading();
        }
    },

    renderTargetList(targets) {
        elements.targetsList.innerHTML = '';

        targets.forEach(target => {
            const targetItem = document.createElement('div');
            targetItem.className = 'target-item';
            targetItem.innerHTML = `
                <div class="target-item__header">
                    <div>
                        <div class="target-item__name">${target.name}</div>
                        <div class="target-item__address">${target.address || 'Address not specified'}</div>
                    </div>
                    <span class="target-item__type">${target.type}</span>
                </div>
                <div class="target-item__description">${target.description || 'No description'}</div>
            `;

            targetItem.addEventListener('click', () => {
                this.connectToTarget(target);
            });

            elements.targetsList.appendChild(targetItem);
        });
    },

    async connectToTarget(target) {
        try {
            Utils.showLoading(`Connecting to ${target.name}...`);

            const connectionResult = await invoke('connect_to_target', {
                serverId: AppState.currentServer.id,
                targetId: target.id,
                token: AppState.currentToken
            });

            if (connectionResult.success) {
                Utils.showSuccess(`Connected to ${target.name}`);
                AppState.connectionStatus = 'connected';

                // If RDP target, launch RDP client
                if (target.type.toLowerCase() === 'rdp') {
                    await invoke('launch_rdp_client', {
                        connectionDetails: connectionResult.connection
                    });
                }

                // Minimize window on Windows
                if (await invoke('get_platform') === 'windows') {
                    await appWindow.minimize();
                }

            } else {
                throw new Error(connectionResult.error || 'Connection failed');
            }

        } catch (error) {
            await logger.logError(error, 'TargetManager connectToTarget failed');
            Utils.showError('Failed to connect to target', error);
        } finally {
            Utils.hideLoading();
        }
    }
};

// Application initialization
async function initializeApp() {
    try {
        await logger.info('Initializing Regis application...');

        // Set up event listeners
        await setupEventListeners();

        // Test: Load some hardcoded servers first to verify UI
        await logger.debug('Testing with hardcoded servers...');
        alert('JavaScript is running!'); // Visual confirmation
        await logger.info('Alert displayed for JavaScript execution test');
        const testServers = [
            {
                id: 'test1',
                name: 'Test Server 1',
                url: 'https://test1.example.com',
                description: 'Test server for UI verification'
            },
            {
                id: 'test2',
                name: 'Test Server 2',
                url: 'https://test2.example.com',
                description: 'Another test server'
            }
        ];

        await logger.debug('Rendering test servers directly...');
        ServerManager.renderServerList(testServers);

        // Load initial server configuration
        await ServerManager.loadServers();

        await logger.info('Application initialized successfully');
    } catch (error) {
        await logger.logError(error, 'Failed to initialize application');
        Utils.showError('Application initialization failed', error);
    }
}

// Event listeners setup
async function setupEventListeners() {
    // Listen for backend events
    await listen('auth-status-changed', (event) => {
        await logger.debug('Auth status changed:', event.payload);
        AppState.isAuthenticated = event.payload.authenticated;
    });

    await listen('connection-status-changed', (event) => {
        await logger.debug('Connection status changed:', event.payload);
        AppState.connectionStatus = event.payload.status;
        elements.statusText.textContent = `Status: ${event.payload.status}`;
    });

    await listen('error-occurred', (event) => {
        await logger.error('Backend error:', event.payload);
        Utils.showError(event.payload.message, event.payload.details);
    });

    // Handle window events
    appWindow.onCloseRequested(async (event) => {
        // Clean up connections before closing
        if (AppState.connectionStatus === 'connected') {
            try {
                await invoke('cleanup_connections');
            } catch (error) {
                await logger.logError(error, 'Failed to cleanup connections');
            }
        }
    });
}

// Start the application when DOM is loaded
document.addEventListener('DOMContentLoaded', async () => {
    try {
        await logger.info('DOMContentLoaded event fired - starting application initialization');
        await initializeApp();
    } catch (error) {
        await logger.logError(error, 'Critical error during DOMContentLoaded initialization');
        console.error('Critical initialization error:', error);
    }
});

// Log that this script is being executed
logger.info('main.js script loaded and executing').catch(console.error);

// Handle unhandled promise rejections
window.addEventListener('unhandledrejection', (event) => {
    logger.logError(event.reason, 'Unhandled promise rejection').catch(console.error);
    Utils.showError('An unexpected error occurred', event.reason);
    event.preventDefault();
});

// Export modules for testing\nexport {\n    AppState,\n    ServerManager,\n    AuthManager,\n    TargetManager,\n    Utils\n};\n\n// Export for debugging
window.RegisApp = {
    AppState,
    ServerManager,
    AuthManager,
    TargetManager,
    Utils
};