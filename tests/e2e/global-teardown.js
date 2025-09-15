// Global teardown for E2E tests
import fs from 'fs/promises';
import path from 'path';
import { fileURLToPath } from 'url';
import { dirname } from 'path';

// ES module equivalent of __dirname
const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

async function globalTeardown() {
  console.log('Cleaning up E2E test environment...');

  try {
    // Clean up test configuration directory if it exists
    const testConfigDir = path.join(__dirname, '../../test-config');

    try {
      await fs.access(testConfigDir);
      await fs.rm(testConfigDir, { recursive: true, force: true });
      console.log(`Removed test config directory: ${testConfigDir}`);
    } catch (error) {
      // Directory doesn't exist, which is fine
      console.log('Test config directory already cleaned up or did not exist');
    }

    // Clean up any temporary files created during tests
    const tempDirs = [
      path.join(__dirname, '../../temp'),
      path.join(__dirname, '../../test-results'),
    ];

    for (const tempDir of tempDirs) {
      try {
        await fs.access(tempDir);
        await fs.rm(tempDir, { recursive: true, force: true });
        console.log(`Cleaned up temporary directory: ${tempDir}`);
      } catch (error) {
        // Directory doesn't exist, which is fine
      }
    }

    // Clean up environment variables
    delete process.env.REGIS_TEST_CONFIG_DIR;

    console.log('E2E test environment cleanup complete');
  } catch (error) {
    console.error('Error during E2E test cleanup:', error);
    // Don't throw - we don't want cleanup errors to fail the test run
  }
}

export default globalTeardown;