# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Regis** is a lightweight Boundary GUI client built with Tauri and vanilla JavaScript, designed as an alternative to the official Boundary Desktop application. It focuses on essential functionality with minimal resource usage.

## Architecture

### Tech Stack
- **Frontend**: Vanilla JavaScript with modern ES modules
- **Backend**: Rust with Tauri framework
- **Build System**: npm + Cargo
- **Configuration**: JSON-based with user overrides

### Directory Structure
```
regis/
├── src/                    # Frontend (Vanilla JS)
│   ├── index.html         # Main HTML file
│   ├── main.js            # Application entry point
│   ├── styles/            # CSS stylesheets
│   ├── components/        # JS components
│   └── utils/             # Utility functions
├── src-tauri/             # Rust backend
│   ├── src/
│   │   ├── main.rs        # Entry point
│   │   ├── lib.rs         # Main application logic
│   │   ├── config.rs      # Configuration management
│   │   ├── boundary.rs    # Boundary CLI integration
│   │   ├── oidc.rs        # OIDC authentication
│   │   └── tray.rs        # System tray integration
│   ├── Cargo.toml         # Rust dependencies
│   └── tauri.conf.json    # Tauri configuration
├── config/                # Configuration files
│   ├── default.json       # Default server configurations
│   └── user.json.template # User override template
└── icons/                 # Application icons
```

## Development Commands

### Prerequisites
- Node.js 18+ and npm 9+
- Rust and Cargo
- Tauri CLI: `npm install -g @tauri-apps/cli`

### Common Commands
```bash
# Development
npm run dev                # Start development server
npm run build             # Build production release
npm run check             # Lint and format code
npm run clean             # Clean build artifacts
npm run setup             # Install all dependencies

# Tauri specific
tauri dev                 # Development mode (alternative)
tauri build               # Production build (alternative)
tauri info                # Environment information
```

## Core Functionality

### 1. Configuration System
- **default.json**: Ships with application, contains default Boundary servers
- **user.json**: User-specific overrides (optional)
- **Merge Logic**: User config overrides default on a per-key basis
- **Schema**: Comprehensive configuration covering UI, security, RDP, OIDC settings

### 2. Authentication Flow
1. User selects Boundary server from configured list
2. Application discovers OIDC configuration from server
3. In-app OIDC flow (no external browser required)
4. Token storage in system keychain with automatic refresh

### 3. Connection Logic
- **Single Target**: Auto-connect immediately after authentication
- **Multiple Targets**: Present selection interface
- **RDP Integration**: Automatic detection and launch of RDP clients
- **Session Management**: Token refresh and connection monitoring

### 4. System Integration
- **Windows**: Minimize to system tray on connection
- **macOS**: Menu bar integration with status indicators
- **Cross-platform**: Consistent core functionality with platform-specific UX

## Key Design Decisions

### Minimal Dependencies
- Vanilla JS instead of React/Vue for smallest footprint
- Direct Boundary CLI integration (no custom API client)
- JSON configuration (no complex config formats)

### Security-First Approach
- System keychain for token storage
- No token logging or persistence in plain text
- SSL certificate validation for all HTTPS requests
- Configurable auto-logout timeouts

### User Experience Focus
- Single-window application with clear workflow
- Auto-connect for simple use cases
- Comprehensive error messages for troubleshooting
- Platform-native system integration

## Configuration Reference

### Server Configuration
```json
{
  "id": "unique-server-id",
  "name": "Display Name",
  "url": "https://boundary.example.com:9200",
  "oidc": {
    "auto_discover": true,
    "provider_hints": {
      "name": "PING Identity",
      "type": "ping"
    }
  }
}
```

### Key Settings
- `boundary.cli_path`: Path to Boundary CLI executable
- `application.auto_connect.single_target`: Auto-connect when only one target
- `security.store_tokens_in_keychain`: Use system keychain for token storage
- `rdp.clients.{platform}`: Platform-specific RDP client configuration

## Development Notes

### Module Architecture
- **config.rs**: Handles all configuration loading and merging
- **boundary.rs**: Wraps Boundary CLI commands and parses output
- **oidc.rs**: Implements OIDC discovery and authentication flows
- **tray.rs**: Manages system tray and menu bar integration

### Frontend State Management
- Centralized `AppState` object in main.js
- Event-driven updates via Tauri's event system
- Component-based UI without framework overhead

### Error Handling Strategy
- Structured error types with help-desk friendly messages
- Comprehensive logging with configurable levels
- Graceful degradation when optional features fail

### Debug Mode
Set `RUST_LOG=debug` environment variable for detailed logging during development.


This documentation should be updated as implementation progresses and new features are added.