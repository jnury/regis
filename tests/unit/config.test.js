// Frontend unit tests for configuration utilities
// Run with: npm test

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';

// Import the modules we want to test
import { Utils } from '../../src/modules/utils.js';
import { ServerManager, AppState } from '../../src/modules/server-manager.js';
import { AuthManager } from '../../src/modules/auth-manager.js';
import { TargetManager } from '../../src/modules/target-manager.js';

// Get reference to the mocked invoke function (mocked in setup.js)
const mockInvoke = vi.mocked(invoke);

describe('ServerManager', () => {
    beforeEach(() => {
        vi.clearAllMocks();
        // Clear DOM
        document.body.innerHTML = '';
    });

    it('should load servers successfully', async () => {
        const mockServers = [
            {
                id: 'test-server',
                name: 'Test Server',
                description: 'Test Description',
                url: 'https://boundary.example.com',
                enabled: true
            }
        ];

        mockInvoke.mockResolvedValue(mockServers);

        // Create required DOM elements
        document.body.innerHTML = `
            <div id="server-list"></div>
            <div id="loading-spinner" class="hidden"></div>
            <div id="status-text">Ready</div>
        `;

        await ServerManager.loadServers();

        expect(mockInvoke).toHaveBeenCalledWith('load_server_config');
        expect(document.getElementById('server-list').children.length).toBe(1);
        expect(document.getElementById('server-list').textContent).toContain('Test Server');
    });

    it('should handle empty server list', async () => {
        mockInvoke.mockResolvedValue([]);

        document.body.innerHTML = `
            <div id="server-list"></div>
            <div id="loading-spinner" class="hidden"></div>
            <div id="status-text">Ready</div>
        `;

        await ServerManager.loadServers();

        expect(document.getElementById('server-list').textContent).toContain('No Boundary servers configured');
    });

    it('should handle load server error', async () => {
        const error = new Error('Failed to load servers');
        mockInvoke.mockRejectedValue(error);

        document.body.innerHTML = `
            <div id="server-list"></div>
            <div id="loading-spinner" class="hidden"></div>
            <div id="status-text">Ready</div>
            <div class="main"></div>
        `;

        await ServerManager.loadServers();

        // Should show error message
        const errorMessage = document.querySelector('.error-message');
        expect(errorMessage).toBeTruthy();
        expect(errorMessage.textContent).toContain('Failed to load server configuration');
    });

    it('should render server list correctly', () => {
        const servers = [
            {
                id: 'server1',
                name: 'Server 1',
                description: 'Description 1',
                url: 'https://server1.example.com'
            },
            {
                id: 'server2',
                name: 'Server 2',
                description: 'Description 2',
                url: 'https://server2.example.com'
            }
        ];

        document.body.innerHTML = `<div id="server-list"></div>`;

        ServerManager.renderServerList(servers);

        const serverList = document.getElementById('server-list');
        const serverItems = serverList.querySelectorAll('.server-item');

        expect(serverItems.length).toBe(2);
        expect(serverItems[0].textContent).toContain('Server 1');
        expect(serverItems[1].textContent).toContain('Server 2');
    });
});

describe('AuthManager', () => {
    beforeEach(() => {
        vi.clearAllMocks();
        document.body.innerHTML = '';
    });

    it('should start OIDC authentication flow', async () => {
        const mockServer = {
            id: 'test-server',
            name: 'Test Server',
            url: 'https://boundary.example.com'
        };

        const mockOidcConfig = {
            provider_name: 'Test Provider',
            issuer: 'https://identity.example.com'
        };

        const mockAuthResult = {
            success: true,
            token: 'mock-token',
            expires_at: '2024-12-31T23:59:59Z'
        };

        mockInvoke
            .mockResolvedValueOnce(mockOidcConfig) // discover_oidc_config
            .mockResolvedValueOnce(mockAuthResult); // start_oidc_auth

        document.body.innerHTML = `
            <div id="auth-content"></div>
            <div id="auth-section"></div>
            <div id="targets-section" class="hidden"></div>
            <div id="loading-spinner" class="hidden"></div>
            <div id="status-text">Ready</div>
        `;

        await AuthManager.startAuthentication(mockServer);

        expect(mockInvoke).toHaveBeenCalledWith('discover_oidc_config', { serverUrl: mockServer.url });
        expect(mockInvoke).toHaveBeenCalledWith('start_oidc_auth', {
            serverUrl: mockServer.url,
            oidcConfig: mockOidcConfig
        });
    });

    it('should handle OIDC discovery failure', async () => {
        const mockServer = {
            id: 'test-server',
            name: 'Test Server',
            url: 'https://boundary.example.com'
        };

        const error = new Error('OIDC discovery failed');
        mockInvoke.mockRejectedValue(error);

        document.body.innerHTML = `
            <div id="auth-content"></div>
            <div id="loading-spinner" class="hidden"></div>
            <div id="status-text">Ready</div>
            <div class="main"></div>
        `;

        await AuthManager.startAuthentication(mockServer);

        // Should show error message
        const errorMessage = document.querySelector('.error-message');
        expect(errorMessage).toBeTruthy();
        expect(errorMessage.textContent).toContain('Authentication failed');
    });
});

describe('TargetManager', () => {
    beforeEach(() => {
        vi.clearAllMocks();
        document.body.innerHTML = '';
    });

    it('should load and render multiple targets', async () => {
        const mockTargets = [
            {
                id: 'target1',
                name: 'Database Server',
                type: 'tcp',
                address: 'db.example.com:5432',
                description: 'PostgreSQL Database'
            },
            {
                id: 'target2',
                name: 'Web Server',
                type: 'rdp',
                address: 'web.example.com:3389',
                description: 'Windows Web Server'
            }
        ];

        mockInvoke.mockResolvedValue(mockTargets);

        const mockServer = { id: 'test-server', url: 'https://boundary.example.com' };
        AppState.currentToken = 'mock-token';

        document.body.innerHTML = `
            <div id="targets-section" class="hidden"></div>
            <div id="targets-list"></div>
            <div id="loading-spinner" class="hidden"></div>
            <div id="status-text">Ready</div>
        `;

        await TargetManager.loadTargets(mockServer);

        expect(mockInvoke).toHaveBeenCalledWith('list_targets', {
            serverUrl: mockServer.url,
            token: 'mock-token'
        });

        // Should show targets section and render targets
        const targetsSection = document.getElementById('targets-section');
        expect(targetsSection.classList.contains('hidden')).toBe(false);

        const targetItems = document.querySelectorAll('.target-item');
        expect(targetItems.length).toBe(2);
        expect(targetItems[0].textContent).toContain('Database Server');
        expect(targetItems[1].textContent).toContain('Web Server');
    });

    it('should auto-connect to single target', async () => {
        const mockTargets = [
            {
                id: 'single-target',
                name: 'Only Target',
                type: 'rdp',
                address: 'server.example.com:3389'
            }
        ];

        const mockConnectionResult = {
            success: true,
            session_id: 'mock-session',
            connection: {
                host: '127.0.0.1',
                port: 52100,
                protocol: 'rdp'
            }
        };

        mockInvoke
            .mockResolvedValueOnce(mockTargets) // list_targets
            .mockResolvedValueOnce(mockConnectionResult) // connect_to_target
            .mockResolvedValueOnce(undefined) // launch_rdp_client
            .mockResolvedValueOnce('macos'); // get_platform

        const mockServer = { id: 'test-server', url: 'https://boundary.example.com' };
        AppState.currentServer = mockServer;
        AppState.currentToken = 'mock-token';

        document.body.innerHTML = `
            <div id="targets-section" class="hidden"></div>
            <div id="targets-list"></div>
            <div id="loading-spinner" class="hidden"></div>
            <div id="status-text">Ready</div>
            <div class="main"></div>
        `;

        await TargetManager.loadTargets(mockServer);

        // Should auto-connect to single target
        expect(mockInvoke).toHaveBeenCalledWith('connect_to_target', {
            serverId: mockServer.id,
            targetId: 'single-target',
            token: 'mock-token'
        });

        // Should launch RDP client for RDP target
        expect(mockInvoke).toHaveBeenCalledWith('launch_rdp_client', {
            connectionDetails: mockConnectionResult.connection
        });
    });

    it('should handle no targets scenario', async () => {
        mockInvoke.mockResolvedValue([]);

        const mockServer = { id: 'test-server', url: 'https://boundary.example.com' };
        AppState.currentToken = 'mock-token';

        document.body.innerHTML = `
            <div id="loading-spinner" class="hidden"></div>
            <div id="status-text">Ready</div>
            <div class="main"></div>
        `;

        await TargetManager.loadTargets(mockServer);

        // Should show error message
        const errorMessage = document.querySelector('.error-message');
        expect(errorMessage).toBeTruthy();
        expect(errorMessage.textContent).toContain('No targets available');
    });
});

describe('Utils', () => {
    beforeEach(() => {
        document.body.innerHTML = '';
    });

    it('should show and hide elements', () => {
        document.body.innerHTML = '<div id="test-element" class="hidden"></div>';
        const element = document.getElementById('test-element');

        Utils.showElement(element);
        expect(element.classList.contains('hidden')).toBe(false);

        Utils.hideElement(element);
        expect(element.classList.contains('hidden')).toBe(true);
    });

    it('should show and hide loading spinner', () => {
        document.body.innerHTML = `
            <div id="loading-spinner" class="hidden"></div>
            <div id="status-text">Ready</div>
        `;

        Utils.showLoading('Test message');

        const spinner = document.getElementById('loading-spinner');
        const status = document.getElementById('status-text');

        expect(spinner.classList.contains('hidden')).toBe(false);
        expect(status.textContent).toBe('Test message');

        Utils.hideLoading();

        expect(spinner.classList.contains('hidden')).toBe(true);
        expect(status.textContent).toBe('Ready');
    });

    it('should display error messages', () => {
        document.body.innerHTML = '<div class="main"></div>';

        Utils.showError('Test error message', 'Additional details');

        const errorMessage = document.querySelector('.error-message');
        expect(errorMessage).toBeTruthy();
        expect(errorMessage.textContent).toContain('Test error message');
        expect(errorMessage.textContent).toContain('Additional details');

        // Should auto-remove after timeout
        setTimeout(() => {
            expect(document.querySelector('.error-message')).toBeFalsy();
        }, 5100);
    });

    it('should display success messages', () => {
        document.body.innerHTML = '<div class="main"></div>';

        Utils.showSuccess('Test success message');

        const successMessage = document.querySelector('.success-message');
        expect(successMessage).toBeTruthy();
        expect(successMessage.textContent).toBe('Test success message');

        // Should auto-remove after timeout
        setTimeout(() => {
            expect(document.querySelector('.success-message')).toBeFalsy();
        }, 3100);
    });
});