# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2026-04-02

### Added

- Initial release of ah CLI tool
- Session management (init, update, restore, list, remove)
- Provider system (dev-templates, devenv)
- Implicit `use` command support (`ah rust go nodejs`)
- Provider management commands (`ah provider list`, `ah provider show`)
- Table-formatted output for better readability
- Auto-enter dev shell after flake update

### Features

- Nix-based development environment management
- Language validation and alias mapping
- Session persistence with JSON storage
- Multi-language support (via providers)

### Commands

- `ah use <lang>...` / `ah <lang>...` - Create and enter dev environment
- `ah init <lang>...` - Initialize session and generate flake.nix
- `ah session update [session]` - Update current session dependencies
- `ah session list` - List all sessions
- `ah session restore <key>` - Restore session by index or ID
- `ah session remove <id>` - Remove specific session
- `ah session clear` - Clear all sessions
- `ah provider list` - List available providers
- `ah provider show <name>` - Show provider details
