// Frontend unit tests for modular utilities
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';

// Import the modules we want to test from separate testable modules
import { Utils } from '../../src/modules/utils.js';
import { ServerManager, AppState } from '../../src/modules/server-manager.js';
import { AuthManager } from '../../src/modules/auth-manager.js';
import { TargetManager } from '../../src/modules/target-manager.js';

// Get reference to the mocked invoke function (mocked in setup.js)
const mockInvoke = vi.mocked(invoke);

describe('ServerManager', () => {
    beforeEach(() => {
        vi.clearAllMocks();
        // DOM is already setup in setup.js, just ensure it's clean
        const serverList = document.getElementById('server-list');
        if (serverList) serverList.innerHTML = '';
    });

    it('should load servers successfully', async () => {
        const mockServers = [
            { id: 'server1', name: 'Test Server', url: 'https://test.com', enabled: true }
        ];

        mockInvoke.mockResolvedValue(mockServers);

        await ServerManager.loadServers();

        expect(mockInvoke).toHaveBeenCalledWith('load_server_config');
    });

    it('should handle empty server list', async () => {
        mockInvoke.mockResolvedValue([]);

        await ServerManager.loadServers();

        const serverList = document.getElementById('server-list');
        expect(serverList.textContent).toContain('No Boundary servers configured');
    });

    it('should handle load server error', async () => {
        mockInvoke.mockRejectedValue(new Error('Failed to load'));

        await ServerManager.loadServers();

        // Should show error message
        expect(document.querySelector('.error-message')).toBeTruthy();
    });

    it('should render server list correctly', () => {
        const servers = [
            { id: 'server1', name: 'Server 1', url: 'https://server1.com', description: 'First server' },
            { id: 'server2', name: 'Server 2', url: 'https://server2.com', description: 'Second server' }
        ];

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
        const authContent = document.getElementById('auth-content');
        if (authContent) authContent.innerHTML = '';
    });

    it('should start OIDC authentication flow', async () => {
        const mockServer = { name: 'Test Server', url: 'https://test.com' };
        const mockOidcConfig = { provider_name: 'Test Provider', issuer: 'https://test.com' };

        mockInvoke.mockResolvedValue(mockOidcConfig);

        await AuthManager.startAuthentication(mockServer);

        expect(mockInvoke).toHaveBeenCalledWith('discover_oidc_config', { serverUrl: mockServer.url });
    });

    it('should handle OIDC discovery failure', async () => {
        const mockServer = { name: 'Test Server', url: 'https://test.com' };

        mockInvoke.mockRejectedValue(new Error('OIDC discovery failed'));

        await AuthManager.startAuthentication(mockServer);

        // Should show error message
        expect(document.querySelector('.error-message')).toBeTruthy();
    });
});

describe('TargetManager', () => {
    beforeEach(() => {
        vi.clearAllMocks();
        const targetsList = document.getElementById('targets-list');
        if (targetsList) targetsList.innerHTML = '';

        // Set up mock state
        AppState.currentServer = { id: 'test-server', name: 'Test Server' };
        AppState.currentToken = 'mock-token';
    });

    it('should load and render multiple targets', async () => {
        const mockServer = { name: 'Test Server', url: 'https://test.com' };
        const mockTargets = [
            { id: 'target1', name: 'Target 1', type: 'tcp' },
            { id: 'target2', name: 'Target 2', type: 'rdp' }
        ];

        mockInvoke.mockResolvedValue(mockTargets);

        await TargetManager.loadTargets(mockServer);

        expect(mockInvoke).toHaveBeenCalledWith('list_targets', {
            serverUrl: mockServer.url,
            token: AppState.currentToken
        });

        const targetsList = document.getElementById('targets-list');
        const targetItems = targetsList.querySelectorAll('.target-item');
        expect(targetItems.length).toBe(2);
    });

    it('should auto-connect to single target', async () => {
        const mockServer = { name: 'Test Server', url: 'https://test.com' };
        const mockTargets = [{ id: 'target1', name: 'Single Target', type: 'tcp' }];

        mockInvoke
            .mockResolvedValueOnce(mockTargets) // list_targets
            .mockResolvedValueOnce({ success: true, session_id: 'session123', connection: {} }); // connect_to_target

        await TargetManager.loadTargets(mockServer);

        // Should auto-connect to single target
        expect(mockInvoke).toHaveBeenCalledWith('connect_to_target', {
            serverId: AppState.currentServer.id,
            targetId: 'target1',
            token: AppState.currentToken
        });
    });

    it('should handle no targets scenario', async () => {
        const mockServer = { name: 'Test Server', url: 'https://test.com' };
        mockInvoke.mockResolvedValue([]);

        await TargetManager.loadTargets(mockServer);

        // Should show error message
        expect(document.querySelector('.error-message')).toBeTruthy();
    });
});

describe('Utils', () => {
    it('should show and hide elements', () => {
        const element = document.createElement('div');
        element.id = 'test-element';
        element.classList.add('hidden');
        document.body.appendChild(element);

        Utils.showElement(element);
        expect(element.classList.contains('hidden')).toBe(false);

        Utils.hideElement(element);
        expect(element.classList.contains('hidden')).toBe(true);
    });

    it('should show and hide loading spinner', () => {
        Utils.showLoading('Test message');

        const spinner = document.getElementById('loading-spinner');
        const statusText = document.getElementById('status-text');

        expect(spinner.classList.contains('hidden')).toBe(false);
        expect(statusText.textContent).toBe('Test message');

        Utils.hideLoading();

        expect(spinner.classList.contains('hidden')).toBe(true);
        expect(statusText.textContent).toBe('Ready');
    });

    it('should display error messages', () => {
        Utils.showError('Test error message', 'Additional details');

        const errorMessage = document.querySelector('.error-message');
        expect(errorMessage).toBeTruthy();
        expect(errorMessage.textContent).toContain('Test error message');
        expect(errorMessage.textContent).toContain('Additional details');
    });

    it('should display success messages', () => {
        Utils.showSuccess('Test success message');

        const successMessage = document.querySelector('.success-message');
        expect(successMessage).toBeTruthy();
        expect(successMessage.textContent).toBe('Test success message');
    });
});