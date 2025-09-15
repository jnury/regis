# Testing Guide for Regis

This document describes the comprehensive testing strategy for the Regis Boundary GUI client, covering unit tests, integration tests, and end-to-end tests.

## Testing Stack

### **Backend (Rust)**
- **Framework**: Built-in Rust testing framework with `#[test]` macros
- **Mocking**: Custom test utilities with `tempfile` for filesystem mocking
- **Coverage**: `cargo test` with coverage reporting

### **Frontend (JavaScript)**
- **Framework**: Vitest (fast Vite-native test runner)
- **Environment**: jsdom for DOM simulation
- **Mocking**: Vitest's built-in mocking for Tauri APIs
- **Coverage**: V8 coverage provider

### **Integration & E2E**
- **Framework**: Playwright for cross-platform Electron/Tauri testing
- **Environment**: Real Tauri application instances
- **Coverage**: Full application workflow testing

## Test Types and Structure

```
tests/
├── unit/              # Frontend unit tests
│   ├── config.test.js  # Configuration management tests
│   └── setup.js       # Test environment setup
├── e2e/               # End-to-end tests
│   ├── app.spec.js    # Main application E2E tests
│   ├── global-setup.js
│   └── global-teardown.js
└── fixtures/          # Test data and configurations

src-tauri/
├── src/
│   └── config.rs      # Contains unit tests in #[cfg(test)] modules
└── tests/
    └── integration_test.rs  # Tauri command integration tests
```

## Running Tests

### **Quick Commands**

```bash
# Run all tests
npm run test:all

# Backend only
npm run test:rust

# Frontend only
npm run test:run

# E2E only
npm run test:e2e

# With coverage
npm run test:coverage

# Interactive testing
npm run test:ui          # Frontend tests UI
npm run test:e2e:ui      # E2E tests UI
```

### **Detailed Commands**

```bash
# Backend Rust tests
cd src-tauri
cargo test                    # Unit tests
cargo test --test integration_test  # Integration tests
cargo test -- --nocapture   # With output

# Frontend JavaScript tests
npm run test                 # Watch mode
npm run test:run            # Single run
npm run test:coverage       # With coverage report
npm run test:ui             # Visual test UI

# E2E tests
npm run test:e2e            # All E2E tests
npm run test:e2e:ui         # Interactive mode
npm run test:e2e:debug      # Debug mode
playwright test --headed    # With browser UI
```

## Test Coverage Areas

### **1. Rust Backend Unit Tests** (`src-tauri/src/config.rs`)

**Configuration Management:**
- ✅ Loading valid default configuration
- ✅ Merging user overrides with defaults
- ✅ Configuration validation (URLs, log levels, themes)
- ✅ Server filtering (enabled/disabled)
- ✅ Helper method functionality
- ✅ Error handling for invalid configurations

**Key Test Functions:**
```rust
#[test]
fn test_config_loading_with_valid_default()
#[test]
fn test_config_merging_user_overrides()
#[test]
fn test_get_enabled_servers()
#[test]
fn test_config_validation_invalid_url()
```

### **2. Frontend Unit Tests** (`tests/unit/config.test.js`)

**Server Management:**
- ✅ Server loading and rendering
- ✅ Empty server list handling
- ✅ Server selection interactions
- ✅ Error state management

**Authentication Flow:**
- ✅ OIDC discovery process
- ✅ Authentication state management
- ✅ Error handling for auth failures

**Target Management:**
- ✅ Target listing and filtering
- ✅ Single target auto-connection
- ✅ Multiple target selection
- ✅ RDP client launching

**Utility Functions:**
- ✅ DOM manipulation helpers
- ✅ Loading state management
- ✅ Error/success message display

### **3. Integration Tests** (`src-tauri/tests/integration_test.rs`)

**Tauri Commands:**
- ✅ `load_server_config` command
- ✅ `get_platform` command
- ✅ `discover_oidc_config` command
- ✅ `start_oidc_auth` command
- ✅ `list_targets` command
- ✅ `connect_to_target` command

**Module Integration:**
- ✅ Boundary CLI integration (mock)
- ✅ OIDC flow integration (mock)
- ✅ Configuration system integration

### **4. End-to-End Tests** (`tests/e2e/app.spec.js`)

**Application Lifecycle:**
- ✅ Application startup and window creation
- ✅ Configuration loading and server display
- ✅ User interface responsiveness
- ✅ Window resize handling

**User Workflows:**
- ✅ Server selection and connection initiation
- ✅ Authentication flow (mock)
- ✅ Target selection and connection
- ✅ Error handling and recovery

**Accessibility:**
- ✅ Keyboard navigation
- ✅ ARIA attributes (planned)
- ✅ Color contrast compliance (planned)

**Performance:**
- ✅ Application load time
- ✅ UI responsiveness with multiple servers

## Test Configuration

### **Vitest Configuration** (`vitest.config.js`)
```javascript
export default defineConfig({
  test: {
    environment: 'jsdom',
    globals: true,
    setupFiles: ['./tests/setup.js'],
    coverage: {
      reporter: ['text', 'json', 'html'],
      exclude: ['node_modules/', 'tests/', 'src-tauri/']
    }
  }
});
```

### **Playwright Configuration** (`playwright.config.js`)
```javascript
export default defineConfig({
  testDir: './tests/e2e',
  fullyParallel: false,  // Electron apps need sequential testing
  use: {
    trace: 'on-first-retry',
    screenshot: 'only-on-failure',
    video: 'retain-on-failure'
  }
});
```

## Mocking Strategy

### **Frontend Mocks**
```javascript
// Tauri API mocking
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}));

// Window API mocking
vi.mock('@tauri-apps/api/window', () => ({
  appWindow: {
    minimize: vi.fn(() => Promise.resolve())
  }
}));
```

### **Backend Test Data**
```rust
// Temporary configuration for testing
fn create_test_config_manager(default_json: &str, user_json: Option<&str>) -> Result<ConfigManager> {
    let temp_dir = TempDir::new()?;
    let config_dir = temp_dir.path().to_path_buf();
    // ... create test configs
    ConfigManager::new_with_paths(config_dir, default_path, user_path)
}
```

## Test Data and Fixtures

### **Mock Server Configurations**
- Test servers with various OIDC providers (PING, generic)
- Invalid configurations for error testing
- Mixed enabled/disabled server states

### **Mock API Responses**
- OIDC discovery responses
- Authentication results
- Target listings (TCP, SSH, RDP)
- Connection establishment data

### **Test Environment Setup**
- Isolated configuration directories
- Predictable test data
- Cleanup after test runs

## Continuous Integration

### **GitHub Actions Pipeline** (planned)
```yaml
- name: Run Rust Tests
  run: cd src-tauri && cargo test

- name: Run Frontend Tests
  run: npm run test:run

- name: Run E2E Tests
  run: npm run test:e2e
```

### **Pre-commit Hooks** (planned)
```bash
#!/bin/sh
npm run test:rust
npm run test:run
npm run lint
```

## Test Development Guidelines

### **Writing Good Tests**

**1. Arrange-Act-Assert Pattern:**
```javascript
test('should load servers successfully', async () => {
  // Arrange
  const mockServers = [{ id: 'test', name: 'Test Server' }];
  mockInvoke.mockResolvedValue(mockServers);

  // Act
  await ServerManager.loadServers();

  // Assert
  expect(mockInvoke).toHaveBeenCalledWith('load_server_config');
});
```

**2. Test Isolation:**
```javascript
beforeEach(() => {
  vi.clearAllMocks();
  document.body.innerHTML = '';
});
```

**3. Descriptive Test Names:**
```rust
#[test]
fn test_config_validation_rejects_invalid_urls() { /* ... */ }
```

### **Testing Best Practices**

1. **Test Behavior, Not Implementation**
   - Focus on what the function does, not how
   - Test public interfaces, not internal details

2. **Mock External Dependencies**
   - Mock Tauri APIs, file system, network calls
   - Use dependency injection where possible

3. **Test Error Conditions**
   - Network failures, invalid inputs, edge cases
   - Ensure graceful degradation

4. **Keep Tests Fast**
   - Unit tests should run in milliseconds
   - Use mocks to avoid I/O operations

5. **Test Coverage Goals**
   - Aim for >80% code coverage
   - 100% coverage for critical paths (auth, config)

## Debugging Tests

### **Frontend Test Debugging**
```bash
# Run specific test file
npm run test config.test.js

# Debug mode with browser
npm run test:ui

# Run with verbose output
npm run test -- --reporter=verbose
```

### **Backend Test Debugging**
```bash
# Run with output
cd src-tauri && cargo test -- --nocapture

# Run specific test
cargo test test_config_loading_with_valid_default

# Show ignored tests
cargo test -- --ignored
```

### **E2E Test Debugging**
```bash
# Run with browser visible
npm run test:e2e:debug

# Run specific test
playwright test app.spec.js

# Generate trace files
playwright test --trace on
```

## Test Maintenance

### **Regular Tasks**
- Update test data when schemas change
- Refresh mock responses with real API changes
- Review and update E2E test scenarios
- Monitor test execution time and optimize slow tests

### **Dependency Updates**
- Keep testing frameworks updated
- Update Playwright browsers regularly
- Sync Tauri API mocks with actual API changes

This comprehensive testing setup ensures reliability, maintainability, and confidence in the Regis application across all platforms and use cases.