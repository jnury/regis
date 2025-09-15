// Authentication management module - extracted for testability
import { invoke } from '@tauri-apps/api/core';
import { Utils } from './utils.js';
import { AppState } from './server-manager.js';

export const AuthManager = {
    async startAuthentication(server) {
        try {
            const authContent = document.getElementById('auth-content');
            if (authContent) {
                authContent.innerHTML = `
                    <div class="auth-message">
                        <p>Starting OIDC authentication with ${server.name}...</p>
                        <p class="text-muted">Please wait while we discover the OIDC configuration.</p>
                    </div>
                `;
            }

            // Discover OIDC configuration
            const oidcConfig = await invoke('discover_oidc_config', { serverUrl: server.url });

            // Start OIDC flow
            await this.handleOidcFlow(server, oidcConfig);

        } catch (error) {
            console.error('Authentication failed:', error);
            Utils.showError('Authentication failed', error);
            Utils.hideLoading();
        }
    },

    async handleOidcFlow(server, oidcConfig) {
        const authContent = document.getElementById('auth-content');
        if (authContent) {
            authContent.innerHTML = `
                <div class="auth-message">
                    <p>Authenticating with ${oidcConfig.provider_name || 'OIDC Provider'}...</p>
                    <div class="auth-progress">
                        <div class="loading-spinner"></div>
                    </div>
                    <p class="text-muted">Complete the authentication in the embedded browser.</p>
                </div>
            `;
        }

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

                const authSection = document.getElementById('auth-section');
                if (authSection) Utils.hideElement(authSection);

                // Load targets
                const { TargetManager } = await import('./target-manager.js');
                await TargetManager.loadTargets(server);
            } else {
                throw new Error(authResult.error || 'Authentication failed');
            }
        } catch (error) {
            console.error('OIDC authentication failed:', error);
            Utils.showError('Authentication failed', error);
            Utils.hideLoading();
        }
    }
};