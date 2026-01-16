# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[Unreleased]: https://github.com/wingnut128/ipcalc/compare/v0.1.3...HEAD
[0.1.3]: https://github.com/wingnut128/ipcalc/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/wingnut128/ipcalc/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/wingnut128/ipcalc/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/wingnut128/ipcalc/releases/tag/v0.1.0
