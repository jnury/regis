// Test setup file for Vitest
import { vi } from 'vitest';

// Mock Tauri APIs globally
global.window = global.window || {};

// Mock Tauri core API
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn()
}));

// Mock Tauri event API
vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => Promise.resolve(() => {}))
}));

// Mock Tauri window API
vi.mock('@tauri-apps/api/window', () => ({
  appWindow: {
    minimize: vi.fn(() => Promise.resolve()),
    onCloseRequested: vi.fn((callback) => {
      // Store callback for testing
      global.__mockCloseCallback = callback;
      return Promise.resolve(() => {});
    })
  }
}));

// Add basic CSS styles for testing
const style = document.createElement('style');
style.textContent = `
  .hidden { display: none !important; }
  .server-item { padding: 10px; margin: 5px; border: 1px solid #ccc; }
  .server-item.selected { background-color: #e3f2fd; }
  .error-message { color: red; background: #ffe6e6; padding: 10px; }
  .success-message { color: green; background: #e6ffe6; padding: 10px; }
  .loading-spinner { display: inline-block; }
  .btn { padding: 5px 10px; margin: 2px; }
  .target-item { padding: 8px; border: 1px solid #ddd; margin: 4px 0; }
`;
document.head.appendChild(style);

// Setup initial DOM structure before any imports happen
document.body.innerHTML = `
  <div class="main">
    <div id="server-selection">
      <div id="server-list"></div>
    </div>
    <div id="auth-section" class="hidden">
      <div id="auth-content"></div>
    </div>
    <div id="targets-section" class="hidden">
      <div id="targets-list"></div>
    </div>
    <div id="status-text">Ready</div>
    <div id="loading-spinner" class="hidden"></div>
  </div>
`;

// Mock DOM APIs that might not be available in test environment
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: vi.fn().mockImplementation(query => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(), // Deprecated
    removeListener: vi.fn(), // Deprecated
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
});

// Mock localStorage
const localStorageMock = {
  getItem: vi.fn(),
  setItem: vi.fn(),
  removeItem: vi.fn(),
  clear: vi.fn(),
};
global.localStorage = localStorageMock;

// Mock console methods to reduce noise in tests
global.console = {
  ...console,
  log: vi.fn(),
  error: vi.fn(),
  warn: vi.fn(),
  info: vi.fn(),
  debug: vi.fn(),
};

// Initialize global test state
beforeEach(() => {
  // Clear all mocks before each test
  vi.clearAllMocks();

  // Setup DOM with required elements for main.js
  document.body.innerHTML = `
    <div class="main">
      <div id="server-selection">
        <div id="server-list"></div>
      </div>
      <div id="auth-section" class="hidden">
        <div id="auth-content"></div>
      </div>
      <div id="targets-section" class="hidden">
        <div id="targets-list"></div>
      </div>
      <div id="status-text">Ready</div>
      <div id="loading-spinner" class="hidden"></div>
    </div>
  `;

  // Reset global state
  if (global.window.RegisApp) {
    global.window.RegisApp.AppState = {
      currentServer: null,
      isAuthenticated: false,
      currentToken: null,
      targets: [],
      connectionStatus: 'disconnected'
    };
  }
});

// Cleanup after each test
afterEach(() => {
  vi.restoreAllMocks();
});