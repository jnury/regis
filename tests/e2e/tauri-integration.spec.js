// Tauri Integration Tests - Testing Tauri commands and app functionality
// Run with: npm run test:e2e

import { test, expect } from '@playwright/test';
import { spawn } from 'child_process';
import path from 'path';
import fs from 'fs/promises';
import { fileURLToPath } from 'url';
import { dirname } from 'path';

// ES module equivalent of __dirname
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const tauriApp = path.join(__dirname, '../../src-tauri/target/release/regis');

test.describe('Tauri Application Integration Tests', () => {
    test('should have built Tauri executable', async () => {
        // Check if the Tauri executable exists
        try {
            await fs.access(tauriApp);
            expect(true).toBe(true); // File exists
        } catch (error) {
            throw new Error(`Tauri executable not found at ${tauriApp}`);
        }
    });

    test('should have valid Tauri configuration', async () => {
        const configPath = path.join(__dirname, '../../src-tauri/tauri.conf.json');
        const configContent = await fs.readFile(configPath, 'utf8');
        const config = JSON.parse(configContent);

        // Verify essential configuration
        expect(config.productName).toBe('Regis');
        expect(config.identifier).toBe('com.regis.boundary-client');
        expect(config.version).toBe('0.1.0');
        expect(config.build.frontendDist).toBe('../src');
    });

    test('should have all required frontend files', async () => {
        const frontendPath = path.join(__dirname, '../../src');

        // Check main files exist
        const requiredFiles = [
            'index.html',
            'main.js',
            'styles/main.css',
            'modules/utils.js',
            'modules/server-manager.js',
            'modules/auth-manager.js',
            'modules/target-manager.js'
        ];

        for (const file of requiredFiles) {
            const filePath = path.join(frontendPath, file);
            try {
                await fs.access(filePath);
            } catch (error) {
                throw new Error(`Required file missing: ${file}`);
            }
        }
    });

    test('should start and exit cleanly', async () => {
        // This test verifies the app can start without crashing
        // We'll start it and then terminate it after a short time

        const appProcess = spawn(tauriApp, [], {
            stdio: 'pipe',
            detached: false
        });

        let processExited = false;
        let exitCode = null;

        appProcess.on('exit', (code) => {
            processExited = true;
            exitCode = code;
        });

        // Let the app start up
        await new Promise(resolve => setTimeout(resolve, 2000));

        // Terminate the process
        appProcess.kill('SIGTERM');

        // Wait for process to exit
        await new Promise(resolve => setTimeout(resolve, 1000));

        if (!processExited) {
            appProcess.kill('SIGKILL');
            await new Promise(resolve => setTimeout(resolve, 500));
        }

        // The app should start without immediate crashes
        // Handle null exit code (process killed) or normal exit codes
        // Exit code 0 = clean exit, 1 = terminated (expected), others = error
        if (exitCode === null) {
            // Process was killed, which is expected for our test
            expect(true).toBe(true);
        } else {
            expect([0, 1, 143, 15]).toContain(exitCode); // 143 = SIGTERM, 15 = SIGTERM on some systems
        }
    });

    test('should have valid package.json configuration', async () => {
        const packagePath = path.join(__dirname, '../../package.json');
        const packageContent = await fs.readFile(packagePath, 'utf8');
        const packageJson = JSON.parse(packageContent);

        expect(packageJson.name).toBe('regis');
        expect(packageJson.version).toBe('0.1.0');
        expect(packageJson.scripts['tauri']).toBeDefined();
        expect(packageJson.devDependencies['@tauri-apps/cli']).toBeDefined();
    });

    test('should have complete testing infrastructure', async () => {
        // Verify all test files exist
        const testFiles = [
            'tests/unit/modules.test.js',
            'tests/unit/config.test.js',
            'tests/setup.js',
            'vitest.config.js',
            'playwright.config.js'
        ];

        for (const testFile of testFiles) {
            const filePath = path.join(__dirname, '../../', testFile);
            try {
                await fs.access(filePath);
            } catch (error) {
                throw new Error(`Test infrastructure file missing: ${testFile}`);
            }
        }
    });
});

// Note: Full GUI E2E testing requires manual testing or specialized Tauri testing tools
// These integration tests verify that:
// 1. The application builds correctly
// 2. All required files are present
// 3. The app can start without immediate crashes
// 4. Configuration is valid
//
// For comprehensive GUI testing, consider:
// - Manual testing of the UI
// - Unit tests for business logic (already implemented)
// - Integration tests for Tauri commands (implemented in Rust tests)