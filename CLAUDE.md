# CLAUDE.md

## Project Overview

**Regis** is a lightweight Boundary GUI client built with Tauri and vanilla JavaScript, designed as an alternative to the official Boundary Desktop application. It focuses on essential functionality with minimal resource usage.

## Critical Rules (Never Violate)
- Keep evything simple so a beginer/medium experimented developer can understand and maintain the code.
- Each time you touch the code, update version in package.json with the following rule: if you just implemented a new important feature, increment the minor version digit; else increment the patch version digit.
- Commit the repository only when I ask. When you create a commit, tag it with the current app verion. Always push after you commited.
- If you learn something interesting and usefull for the rest of the project, update this CLAUDE.md file in section "Today I learned". But before, ask me if your new knowledge is correct.
- If you made a mistake in your interpretation of the specs, architecture, features etc. update this CLAUDE.md file in section "Never again". But before, ask me if your new knowledge is correct.
- Always ask questions when you need clarification or if you have the choice between multiple solutions.
- Always use context7 for library and tools documentation

## Always Think Step by Step
- Read specification → Check dependencies → Validate data flow → Implement incrementally → Test immediately

## Today I learned
- When using playwright, always add ' --reporter=line' so you don't have to wait for results

## Never again
- Never add features that weren't explicitly requested (like the Auto-save toggle I added to Settings). Always implement exactly what was asked for, but DO propose good ideas as suggestions for the user to accept or decline. Frame additional features as questions: "Would you also like me to add [feature], or should we keep it as-is for now?"
- Never rely on dynamic feather icon updates - they break when innerHTML is replaced. Use CSS visibility or dual-button patterns instead.

## Architecture

### Tech Stack
- **Frontend**: Vanilla JavaScript with modern ES modules
- **Backend**: Rust with Tauri framework
- **Build System**: npm + Cargo
- **Configuration**: JSON-based with user overrides

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
