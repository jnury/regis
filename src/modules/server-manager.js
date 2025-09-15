// Server management module - extracted for testability
import { invoke } from '@tauri-apps/api/core';
import { Utils } from './utils.js';

// Application state (simplified for testing)
export const AppState = {
    currentServer: null,
    isAuthenticated: false,
    currentToken: null,
    targets: [],
    connectionStatus: 'disconnected'
};

export const ServerManager = {
    async loadServers() {
        try {
            Utils.showLoading('Loading server configuration...');
            const servers = await invoke('load_server_config');
            this.renderServerList(servers);
            Utils.hideLoading();
        } catch (error) {
            console.error('Failed to load servers:', error);
            Utils.showError('Failed to load server configuration', error);
            Utils.hideLoading();
        }
    },

    renderServerList(servers) {
        const serverList = document.getElementById('server-list');
        if (!serverList) return;

        serverList.innerHTML = '';

        if (!servers || servers.length === 0) {
            serverList.innerHTML = `
                <div class="text-center text-muted">
                    <p>No Boundary servers configured.</p>
                    <p>Please add server configurations to the config file.</p>
                </div>
            `;
            return;
        }

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

            serverList.appendChild(serverItem);
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
            const serverSelection = document.getElementById('server-selection');
            const authSection = document.getElementById('auth-section');

            if (serverSelection) Utils.hideElement(serverSelection);
            if (authSection) Utils.showElement(authSection);

            // Start authentication process
            const { AuthManager } = await import('./auth-manager.js');
            await AuthManager.startAuthentication(server);

        } catch (error) {
            console.error('Connection failed:', error);
            Utils.showError('Failed to connect to server', error);
            Utils.hideLoading();
        }
    }
};