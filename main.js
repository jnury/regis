// Regis - Boundary GUI Client
// Build timestamp: 2025-09-19T18:09:00Z

// Logging system
class Logger {
    constructor() {
        this.component = 'frontend';
        this.isInitialized = false;
    }

    async init() {
        try {
            // Check if Tauri is available
            if (window.__TAURI__ && window.__TAURI__.core) {
                this.invoke = window.__TAURI__.core.invoke;
                this.isInitialized = true;
                this.info('Frontend logging system initialized');
            } else {
                console.warn('Tauri environment not available for logging');
                this.isInitialized = false;
            }
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

        appConfig = await window.__TAURI__.core.invoke('get_config');

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
    console.log('handleConnect called!', { selectedServer });
    await logger.info('Connect button clicked', 'auth');

    if (!selectedServer) {
        console.log('No server selected');
        showError('Please select a server first');
        return;
    }

    console.log('Starting authentication for server:', selectedServer.name);
    try {
        await logger.info('Starting authentication for server', 'auth', { server: selectedServer.name });
    } catch (e) {
        console.warn('Logger failed:', e);
    }

    try {
        console.log('Starting try block...');

        // Update UI to show authentication in progress
        console.log('Calling updateConnectButtonForAuth...');
        updateConnectButtonForAuth(true);

        console.log('Calling hideError...');
        hideError();

        // First, discover OIDC auth methods for this server
        console.log('About to discover OIDC auth methods...');
        try {
            await logger.info('Discovering OIDC auth methods', 'auth');
        } catch (e) {
            console.warn('Logger failed:', e);
        }

        console.log('Calling discover_oidc_auth_methods_command with serverId:', selectedServer.id);
        let authMethods;
        try {
            authMethods = await window.__TAURI__.core.invoke('discover_oidc_auth_methods_command', {
                serverId: selectedServer.id
            });
            console.log('OIDC discovery result:', authMethods);
        } catch (discoveryError) {
            console.error('OIDC discovery failed with error:', discoveryError);
            throw new Error(`OIDC discovery failed: ${discoveryError.message || discoveryError}`);
        }

        if (!authMethods || authMethods.length === 0) {
            throw new Error('No OIDC authentication methods available for this server');
        }

        // Use the first available auth method
        const authMethod = authMethods[0];
        await logger.info('Using auth method', 'auth', { methodId: authMethod.id });

        // Start OIDC authentication
        const authRequest = {
            serverId: selectedServer.id,
            authMethodId: authMethod.id,
            scopeId: null
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

// Show authentication success and transition to target selection
async function showAuthenticationSuccess() {
    await logger.info('Authentication successful, transitioning to target selection', 'app');

    // Hide the connect button and show target discovery
    connectButton.style.display = 'none';

    // Start target discovery immediately
    await showTargetSelection();
}

// Show target selection UI with real-time fetching
async function showTargetSelection() {
    await logger.info('Loading target selection UI', 'targets');

    const targetSelectionHTML = `
        <div class="target-selection">
            <div class="target-header">
                <h2>Select Target</h2>
                <p>Connected to <strong>${selectedServer.name}</strong></p>
                <div class="target-actions">
                    <input type="text" id="target-search" placeholder="Search targets..." class="target-search">
                    <button id="refresh-targets" class="refresh-btn">↻ Refresh</button>
                    <button id="back-to-servers" class="back-btn">← Back to Servers</button>
                </div>
            </div>
            <div class="target-list-container">
                <div id="target-loading" class="loading-state">
                    <div class="spinner"></div>
                    <p>Discovering available targets...</p>
                </div>
                <div id="target-list" class="target-list" style="display: none;"></div>
                <div id="target-error" class="error-state" style="display: none;"></div>
            </div>
        </div>
    `;

    serverListElement.innerHTML = targetSelectionHTML;

    // Add event listeners
    document.getElementById('target-search').addEventListener('input', handleTargetSearch);
    document.getElementById('refresh-targets').addEventListener('click', refreshTargets);
    document.getElementById('back-to-servers').addEventListener('click', backToServerSelection);

    // Start target discovery
    await discoverAndDisplayTargets();
}

// Discover and display targets
async function discoverAndDisplayTargets() {
    const loadingElement = document.getElementById('target-loading');
    const targetListElement = document.getElementById('target-list');
    const errorElement = document.getElementById('target-error');

    try {
        await logger.info('Discovering targets from Boundary server', 'targets');

        // Show loading state
        loadingElement.style.display = 'block';
        targetListElement.style.display = 'none';
        errorElement.style.display = 'none';

        // Discover all targets for the authenticated user
        const targets = await window.__TAURI__.core.invoke('discover_all_targets_command', {
            serverId: selectedServer.id
        });

        await logger.info('Targets discovered successfully', 'targets', { count: targets.length });

        if (targets.length === 0) {
            showNoTargetsMessage();
        } else if (targets.length === 1) {
            // Auto-connect for single target
            await handleSingleTargetAutoConnect(targets[0]);
        } else {
            // Show target selection UI
            displayTargetList(targets);
        }

    } catch (error) {
        await logger.error('Failed to discover targets', 'targets', { error: error.message });
        showTargetError(error.message || error);
    }
}

// Display the list of targets
function displayTargetList(targets) {
    const loadingElement = document.getElementById('target-loading');
    const targetListElement = document.getElementById('target-list');

    // Hide loading, show target list
    loadingElement.style.display = 'none';
    targetListElement.style.display = 'block';

    // Store targets globally for search functionality
    window.availableTargets = targets;

    // Group targets by type for better organization
    const groupedTargets = groupTargetsByType(targets);

    let targetsHTML = '';

    Object.keys(groupedTargets).forEach(type => {
        const typeTargets = groupedTargets[type];
        const typeIcon = getTargetTypeIcon(type);

        targetsHTML += `
            <div class="target-group">
                <h3 class="target-group-header">
                    <span class="target-type-icon">${typeIcon}</span>
                    ${escapeHtml(type)} (${typeTargets.length})
                </h3>
                <div class="target-group-list">
                    ${typeTargets.map(target => `
                        <div class="target-item" data-target-id="${target.id}" data-target-type="${target.type}">
                            <div class="target-info">
                                <div class="target-name">${escapeHtml(target.name)}</div>
                                <div class="target-description">${escapeHtml(target.description || 'No description')}</div>
                                <div class="target-details">
                                    <span class="target-id">ID: ${escapeHtml(target.id)}</span>
                                    <span class="target-address">${escapeHtml(target.address || 'Dynamic')}</span>
                                </div>
                            </div>
                            <button class="target-connect-btn" data-target-id="${target.id}">Connect</button>
                        </div>
                    `).join('')}
                </div>
            </div>
        `;
    });

    targetListElement.innerHTML = targetsHTML;

    // Add click handlers for target connection
    targetListElement.querySelectorAll('.target-connect-btn').forEach(button => {
        button.addEventListener('click', async (e) => {
            e.stopPropagation();
            const targetId = button.getAttribute('data-target-id');
            const target = targets.find(t => t.id === targetId);
            if (target) {
                await handleTargetConnection(target);
            }
        });
    });

    // Add click handlers for target items (select on click)
    targetListElement.querySelectorAll('.target-item').forEach(item => {
        item.addEventListener('click', () => {
            // Remove previous selection
            targetListElement.querySelectorAll('.target-item').forEach(i => i.classList.remove('selected'));
            // Add selection to clicked item
            item.classList.add('selected');
        });
    });
}

// Group targets by type for better organization
function groupTargetsByType(targets) {
    const grouped = {};

    targets.forEach(target => {
        const type = target.type || 'Unknown';
        if (!grouped[type]) {
            grouped[type] = [];
        }
        grouped[type].push(target);
    });

    return grouped;
}

// Get icon for target type
function getTargetTypeIcon(type) {
    const icons = {
        'tcp': '🔗',
        'ssh': '💻',
        'rdp': '🖥️',
        'http': '🌐',
        'https': '🔒',
        'Unknown': '❓'
    };
    return icons[type] || icons['Unknown'];
}

// Handle target search
function handleTargetSearch(event) {
    const searchTerm = event.target.value.toLowerCase();
    const targets = window.availableTargets || [];

    if (!searchTerm) {
        displayTargetList(targets);
        return;
    }

    const filteredTargets = targets.filter(target =>
        target.name.toLowerCase().includes(searchTerm) ||
        target.description?.toLowerCase().includes(searchTerm) ||
        target.id.toLowerCase().includes(searchTerm) ||
        target.address?.toLowerCase().includes(searchTerm)
    );

    displayTargetList(filteredTargets);
}

// Refresh targets
async function refreshTargets() {
    await logger.info('Refreshing target list', 'targets');
    await discoverAndDisplayTargets();
}

// Handle single target auto-connect
async function handleSingleTargetAutoConnect(target) {
    await logger.info('Single target found, auto-connecting', 'targets', { targetName: target.name });

    const targetListElement = document.getElementById('target-list');
    const loadingElement = document.getElementById('target-loading');

    // Show auto-connect message
    loadingElement.innerHTML = `
        <div class="auto-connect-state">
            <div class="spinner"></div>
            <h3>Auto-connecting to ${escapeHtml(target.name)}</h3>
            <p>Only one target available, connecting automatically...</p>
        </div>
    `;

    // Connect to the single target
    await handleTargetConnection(target);
}

// Show no targets message
function showNoTargetsMessage() {
    const loadingElement = document.getElementById('target-loading');
    const targetListElement = document.getElementById('target-list');

    loadingElement.style.display = 'none';
    targetListElement.style.display = 'block';
    targetListElement.innerHTML = `
        <div class="no-targets-state">
            <h3>No Targets Available</h3>
            <p>No targets are currently available in your scope.</p>
            <p>Contact your administrator if you expect to see targets here.</p>
            <button id="retry-targets" class="retry-btn">Try Again</button>
        </div>
    `;

    document.getElementById('retry-targets').addEventListener('click', refreshTargets);
}

// Show target error
function showTargetError(errorMessage) {
    const loadingElement = document.getElementById('target-loading');
    const errorElement = document.getElementById('target-error');

    loadingElement.style.display = 'none';
    errorElement.style.display = 'block';
    errorElement.innerHTML = `
        <div class="error-content">
            <h3>Failed to Load Targets</h3>
            <p>${escapeHtml(errorMessage)}</p>
            <button id="retry-targets" class="retry-btn">Retry</button>
        </div>
    `;

    document.getElementById('retry-targets').addEventListener('click', refreshTargets);
}

// Handle target connection
async function handleTargetConnection(target) {
    await logger.info('Initiating connection to target', 'connection', {
        targetName: target.name,
        targetId: target.id
    });

    try {
        // Update UI to show connection in progress
        const connectBtn = document.querySelector(`[data-target-id="${target.id}"]`);
        if (connectBtn) {
            connectBtn.disabled = true;
            connectBtn.textContent = 'Connecting...';
        }

        // Authorize session for this target
        await logger.info('Authorizing session for target', 'connection');
        const authorization = await window.__TAURI__.core.invoke('authorize_session_command', {
            serverId: selectedServer.id,
            targetId: target.id
        });

        // Establish connection
        await logger.info('Establishing connection', 'connection');
        const connection = await window.__TAURI__.core.invoke('establish_connection_command', {
            serverId: selectedServer.id,
            authorization: authorization,
            connection_type: "tcp", // Default connection type
            target_name: target.name
        });

        await logger.info('Connection established successfully', 'connection', {
            localAddress: connection.local_address,
            localPort: connection.local_port
        });

        // Show connection success and handle RDP launch if applicable
        await showConnectionSuccess(target, connection);

    } catch (error) {
        await logger.error('Connection failed', 'connection', { error: error.message });

        // Reset button state
        const connectBtn = document.querySelector(`[data-target-id="${target.id}"]`);
        if (connectBtn) {
            connectBtn.disabled = false;
            connectBtn.textContent = 'Connect';
        }

        showError(`Connection failed: ${error.message || error}`);
    }
}

// Show connection success and handle post-connection actions
async function showConnectionSuccess(target, connection) {
    await logger.info('Connection successful, handling post-connection actions', 'connection');

    // Check if this is an RDP target and launch client if available
    if (target.type === 'rdp' || target.name.toLowerCase().includes('rdp')) {
        await handleRDPClientLaunch(target, connection);
    }

    // Show connection status
    showConnectionStatus(target, connection);
}

// Handle RDP client launch
async function handleRDPClientLaunch(target, connection) {
    try {
        await logger.info('Detecting RDP clients for auto-launch', 'rdp');

        const rdpClients = await window.__TAURI__.core.invoke('detect_rdp_clients_command');

        if (rdpClients.length > 0) {
            const client = rdpClients[0]; // Use first available client

            await logger.info('Launching RDP client', 'rdp', { client: client.name });

            await window.__TAURI__.core.invoke('launch_rdp_client_command', {
                clientPath: client.path,
                targetAddress: connection.local_address,
                targetPort: connection.local_port,
                targetName: target.name
            });

            await logger.info('RDP client launched successfully', 'rdp');
        } else {
            await logger.warn('No RDP clients detected', 'rdp');
            showManualConnectionInfo(target, connection);
        }

    } catch (error) {
        await logger.error('RDP client launch failed', 'rdp', { error: error.message });
        showManualConnectionInfo(target, connection);
    }
}

// Show manual connection information
function showManualConnectionInfo(target, connection) {
    const info = `
        <div class="manual-connection-info">
            <h4>Manual Connection Required</h4>
            <p>Connect manually using these details:</p>
            <div class="connection-details">
                <div><strong>Address:</strong> ${connection.local_address}</div>
                <div><strong>Port:</strong> ${connection.local_port}</div>
                <div><strong>Target:</strong> ${target.name}</div>
            </div>
        </div>
    `;

    // Add to the target list as a notification
    const targetListElement = document.getElementById('target-list');
    const infoDiv = document.createElement('div');
    infoDiv.className = 'connection-info-notification';
    infoDiv.innerHTML = info;
    targetListElement.insertBefore(infoDiv, targetListElement.firstChild);
}

// Show connection status
function showConnectionStatus(target, connection) {
    // Replace target selection with connection monitoring view
    const connectionStatusHTML = `
        <div class="connection-status">
            <div class="connection-header">
                <h2>Connected to ${escapeHtml(target.name)}</h2>
                <p>Server: <strong>${selectedServer.name}</strong></p>
            </div>
            <div class="connection-details">
                <div class="connection-info">
                    <h3>Connection Details</h3>
                    <div class="detail-item">
                        <span class="label">Local Address:</span>
                        <span class="value">${connection.local_address}:${connection.local_port}</span>
                    </div>
                    <div class="detail-item">
                        <span class="label">Target ID:</span>
                        <span class="value">${target.id}</span>
                    </div>
                    <div class="detail-item">
                        <span class="label">Session ID:</span>
                        <span class="value">${connection.session_id}</span>
                    </div>
                    <div class="detail-item">
                        <span class="label">Status:</span>
                        <span class="value status-active">Active</span>
                    </div>
                </div>
                <div class="connection-actions">
                    <button id="terminate-connection" class="danger-btn">Terminate Connection</button>
                    <button id="back-to-targets" class="secondary-btn">Back to Targets</button>
                    <button id="monitor-session" class="primary-btn">Monitor Session</button>
                </div>
            </div>
        </div>
    `;

    serverListElement.innerHTML = connectionStatusHTML;

    // Add event listeners
    document.getElementById('terminate-connection').addEventListener('click', () => terminateConnection(connection));
    document.getElementById('back-to-targets').addEventListener('click', showTargetSelection);
    document.getElementById('monitor-session').addEventListener('click', () => showSessionMonitoring(connection));
}

// Terminate connection
async function terminateConnection(connection) {
    try {
        await logger.info('Terminating connection', 'connection', { sessionId: connection.session_id });

        await window.__TAURI__.core.invoke('terminate_connection_command', {
            session_id: connection.session_id
        });

        await logger.info('Connection terminated successfully', 'connection');

        // Return to target selection
        await showTargetSelection();

    } catch (error) {
        await logger.error('Failed to terminate connection', 'connection', { error: error.message });
        showError(`Failed to terminate connection: ${error.message || error}`);
    }
}

// Show session monitoring (placeholder for future implementation)
function showSessionMonitoring(connection) {
    alert(`Session monitoring for ${connection.session_id} - Coming soon!`);
}

// Back to server selection
function backToServerSelection() {
    // Reset state
    selectedServer = null;

    // Show connect button again
    connectButton.style.display = 'block';
    updateConnectButton();

    // Reload servers
    loadServers();
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