# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.2] - 2026-02-07

### Security

- Updated `bytes` from 1.11.0 to 1.11.1 — fixes integer overflow in `BytesMut::reserve`
- Updated `time` from 0.3.45 to 0.3.47 — fixes stack exhaustion Denial of Service
- Deprecated v0.3.1 release due to vulnerable transitive dependencies

### Changed

- Added personality, commit rules, and security filters to CLAUDE.md

## [0.3.1] - 2026-01-20

### Added

- Optional `?pretty=true` query parameter for API endpoints to format JSON output with indentation
- Improved API readability for browser and debugging use cases

### Security

- Fixed RUSTSEC-2026-0002 / GHSA-rhfx-m35p-ff5j: Updated `ratatui` from 0.26 to 0.30, which resolves low severity vulnerability in `lru` crate (IterMut Stacked Borrows violation)
- Updated `lru` transitive dependency from 0.12.5 to 0.16.3 (patched version)

### Changed

- Updated deprecated `Frame::size()` call to `Frame::area()` for ratatui 0.30 compatibility
- API responses default to compact JSON for optimal performance

## [0.3.0] - 2026-01-20 [YANKED]

**This version has been yanked due to a security vulnerability in a transitive dependency. Please use 0.3.1 or later.**

### Added

- Interactive Terminal User Interface (TUI) mode with dual-mode operation (optional `tui` feature)
  - Calculate mode for real-time subnet information display
  - Split mode for interactive subnet generation with scrollable results
  - TAB key to switch between Calculate and Split modes
  - Support for MAX mode to generate all possible subnets
  - Arrow key navigation for scrolling through generated subnet lists
  - Color-coded input fields with active field highlighting
  - Real-time validation and error messages
  - Automatic IPv4/IPv6 detection
- `--tui` command-line flag to launch TUI mode (only available when built with `tui` feature)
- Optional dependencies: `ratatui`, `crossterm`, and `ipnet` for TUI functionality

### Changed

- TUI feature is opt-in and not included in default builds to maintain smaller binary size
- Module structure reorganized: `tui` module now part of `lib.rs` instead of `main.rs`

## [0.2.1] - 2026-01-16

### Fixed

- Swagger endpoints now only appear in help text when swagger feature is enabled

## [0.2.0] - 2026-01-16

### Added

- OpenAPI 3.0 documentation for all API endpoints via optional `swagger` feature (enabled by default)
- New `/api-docs/openapi.json` endpoint to retrieve OpenAPI specification
- Comprehensive schema documentation for all request/response types
- Support for importing API spec into Swagger Editor, Postman, Insomnia, and other tools

### Changed

- API documentation is now machine-readable and can be consumed by API tooling
- Binary can be built without swagger support using `--no-default-features` for smaller size

## [0.1.8] - 2026-01-16

### Changed

- CLI now displays help message when run without arguments instead of showing an error
- Improved user experience with exit code 0 (success) when showing help

## [0.1.7] - 2026-01-16

### Added

- Direct CIDR notation support: use `ipcalc <cidr>` instead of subcommands
- Auto-detection of IPv4 vs IPv6 based on input format
- Integration tests for direct CIDR input and deprecation warnings

### Changed

- Simplified CLI interface - CIDR can now be passed directly as a positional argument
- Users should now use `ipcalc 192.168.1.0/24` instead of `ipcalc v4 192.168.1.0/24`

### Deprecated

- `v4` subcommand - use `ipcalc <cidr>` instead
- `v6` subcommand - use `ipcalc <cidr>` instead

## [0.1.6] - 2026-01-16

### Fixed

- Broken pipe panic when output is piped to commands like `head`

## [0.1.5] - 2025-01-16

### Changed

- Updated axum from 0.7 to 0.8
- Updated thiserror from 1 to 2
- Updated tower-http from 0.5 to 0.6

## [0.1.4] - 2025-01-16

### Added

- `--max` (`-m`) option for split command to generate maximum number of subnets possible
- API support for `max=true` query parameter on `/v4/split` and `/v6/split` endpoints

### Changed

- IPv6 help text now uses "prefix" terminology instead of "CIDR" for consistency with IPv6 conventions
- IPv6 example in help changed from `/32` to `/48` (more typical enterprise allocation)
- Split command now requires either `--count` or `--max` (mutually exclusive)

## [0.1.3] - 2025-01-16

### Added

- CI and license status badges to README

### Fixed

- Code formatting to comply with rustfmt standards

## [0.1.2] - 2025-01-16

### Added

- Pre-commit git hook for automated linting and format checks
- `make setup` command to install git hooks for development

## [0.1.1] - 2025-01-16

### Added

- CI workflow with automated testing, linting, and format checks
- CodeQL security scanning (on push, PR, and weekly schedule)
- cargo-audit for dependency vulnerability scanning

## [0.1.0] - 2025-01-16

### Added

- IPv4 subnet calculation with network address, broadcast, subnet mask, wildcard mask, host ranges
- IPv4 network class detection (A, B, C, D, E) and private address identification
- IPv6 prefix calculation with full hextet expansion
- IPv6 address type detection (global unicast, link-local, ULA, multicast, loopback)
- Subnet generator to split supernets into smaller subnets
- CLI interface with `v4`, `v6`, `split`, and `serve` commands
- JSON output format (default)
- Plain text output format (`--format text`)
- File output option (`-o, --output`)
- HTTP API server with REST endpoints
- API endpoints: `/health`, `/v4`, `/v6`, `/v4/split`, `/v6/split`
- Structured logging with tracing (stdout, file output, JSON format)
- Configurable log levels (trace, debug, info, warn, error)
- HTTP request tracing via tower-http
- Unit tests for IPv4, IPv6, and subnet generation
- Integration tests for CLI
- Dockerfile for containerized deployment
- Makefile for common development tasks

[Unreleased]: https://github.com/wingnut128/ipcalc/compare/v0.3.2...HEAD
[0.3.2]: https://github.com/wingnut128/ipcalc/compare/v0.3.1...v0.3.2
[0.3.1]: https://github.com/wingnut128/ipcalc/compare/v0.3.0...v0.3.1
[0.3.0]: https://github.com/wingnut128/ipcalc/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/wingnut128/ipcalc/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/wingnut128/ipcalc/compare/v0.1.8...v0.2.0
[0.1.8]: https://github.com/wingnut128/ipcalc/compare/v0.1.7...v0.1.8
[0.1.7]: https://github.com/wingnut128/ipcalc/compare/v0.1.6...v0.1.7
[0.1.6]: https://github.com/wingnut128/ipcalc/compare/v0.1.5...v0.1.6
[0.1.5]: https://github.com/wingnut128/ipcalc/compare/v0.1.4...v0.1.5
[0.1.4]: https://github.com/wingnut128/ipcalc/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/wingnut128/ipcalc/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/wingnut128/ipcalc/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/wingnut128/ipcalc/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/wingnut128/ipcalc/releases/tag/v0.1.0
