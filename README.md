# Regis - Lightweight Boundary GUI Client

A minimal, fast GUI alternative to Boundary Desktop built with Tauri and vanilla JavaScript. Designed for organizations that need a lightweight solution covering essential Boundary use cases without the overhead of the official client.

## Overview

Regis provides a streamlined interface for HashiCorp Boundary connections with focus on:
- **Minimal resource footprint** - Built with Tauri for native performance
- **Essential features only** - No bloat, just what you need to connect
- **OIDC integration** - In-app authentication without external browser dependencies
- **Automatic connection handling** - Smart connection logic based on available targets

## Features

### Core Functionality
- **Server Selection**: Choose from pre-configured Boundary server endpoints
- **Embedded OIDC Flow**: Complete authentication process within the application
- **Automatic Connection**:
  - Single target: Auto-connect immediately after authentication
  - Multiple targets: Present target list for user selection
- **RDP Integration**: Automatic Remote Desktop launch for RDP targets
- **Multi-Target Support**: Open multiple connections to the same Boundary server

### User Experience
- **System Integration**:
  - Taskbar/dock icon status indicators
  - Windows: Minimize to system tray on connection
  - macOS: Menu bar integration
- **Session Management**: Automatic token refresh while possible
- **Error Diagnostics**: Detailed error messages with help-desk information
- **Manual Retry**: User-initiated connection retry on failures

### Technical Specifications
- **Frontend**: Vanilla JavaScript (KISS principle)
- **Framework**: Tauri for cross-platform native performance
- **Platforms**: macOS and Windows (Linux-compatible architecture)
- **Boundary CLI**: Uses system-installed CLI (version agnostic)
- **Authentication**: OIDC with primary support for PING Identity
- **Configuration**: File-based (bundled defaults + user customizations)

## Architecture

### Application Structure
```
regis/
├── src-tauri/           # Rust backend (Tauri)
│   ├── src/
│   │   ├── main.rs      # Main application entry
│   │   ├── boundary.rs  # Boundary CLI integration
│   │   ├── config.rs    # Configuration management
│   │   └── oidc.rs      # OIDC authentication flow
│   └── Cargo.toml
├── src/                 # Frontend (Vanilla JS)
│   ├── index.html       # Main application window
│   ├── main.js          # Application logic
│   ├── components/      # UI components
│   └── styles/          # CSS styling
├── config/              # Configuration files
│   ├── default.json     # Bundled server configurations
│   └── user.json        # User preferences (optional)
└── icons/               # Application icons
```

### Configuration System
- **default.json**: Bundled Boundary server endpoints
- **user.json**: User preferences and custom settings
- No UI for server management (config file only)

### Connection Flow
1. **Startup**: Load server list from configuration
2. **Server Selection**: User chooses Boundary server endpoint
3. **Authentication**: In-app OIDC flow with selected server
4. **Target Discovery**: Retrieve available targets for user
5. **Connection Logic**:
   - Single target: Auto-connect and launch appropriate client
   - Multiple targets: Display selection interface
6. **Session Management**: Monitor connection and refresh tokens

### Error Handling
Comprehensive error diagnostics including:
- Server unreachable (network/DNS issues)
- TCP connection available but no response
- OIDC authentication failures
- Boundary CLI errors
- Target connection failures

Each error includes actionable help-desk information for troubleshooting.

## Development

### Prerequisites
- Node.js and npm
- Rust and Cargo
- Tauri CLI: `npm install -g @tauri-apps/cli`
- System-installed Boundary CLI

### Commands
```bash
# Development server
npm run tauri dev

# Build for production
npm run tauri build

# Run tests
npm run test
```

### Platform Considerations
- **Cross-platform compatibility**: Architecture designed to avoid Linux incompatibilities
- **System tray**: Windows-specific minimize behavior
- **Menu bar**: macOS-specific integration patterns

## License

MIT License - see [LICENSE](LICENSE) file for details.