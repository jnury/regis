// Target management module - extracted for testability
import { invoke } from '@tauri-apps/api/core';
import { appWindow } from '@tauri-apps/api/window';
import { Utils } from './utils.js';
import { AppState } from './server-manager.js';

export const TargetManager = {
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
                const targetsSection = document.getElementById('targets-section');
                if (targetsSection) Utils.showElement(targetsSection);
            } else {
                Utils.showError('No targets available for your user');
            }

            Utils.hideLoading();
        } catch (error) {
            console.error('Failed to load targets:', error);
            Utils.showError('Failed to load targets', error);
            Utils.hideLoading();
        }
    },

    renderTargetList(targets) {
        const targetsList = document.getElementById('targets-list');
        if (!targetsList) return;

        targetsList.innerHTML = '';

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

            targetsList.appendChild(targetItem);
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
            console.error('Connection failed:', error);
            Utils.showError('Failed to connect to target', error);
        } finally {
            Utils.hideLoading();
        }
    }
};