# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Development Commands

```bash
# Essential commands
make check          # Run fmt-check, lint, and test (use before commits)
make test           # Run all tests
make lint           # Run clippy with -D warnings
make fmt            # Format code

# Build
make build          # Debug build
make release        # Release build
cargo install --path .  # Install binary locally

# Run single test
cargo test test_name

# API server
make serve          # Run on localhost:8080
make serve-debug    # Run with debug logging
```

**Important**: Run `make setup` after cloning to install git hooks that enforce formatting and linting on commits.

## Architecture

This is a Rust CLI/API for IPv4 and IPv6 subnet calculations.

**Core flow**: CLI (`main.rs`) parses args via clap (`cli.rs`) → routes to calculation modules (`ipv4.rs`, `ipv6.rs`, `subnet_generator.rs`) → formats output (`output.rs`).

**Key modules**:
- `ipv4.rs` / `ipv6.rs` - Subnet calculation logic using bitwise operations (u32/u128)
- `subnet_generator.rs` - Splits supernets into smaller subnets (supports `--count` or `--max`)
- `api.rs` - Axum HTTP server with 6 endpoints sharing the same data structures as CLI
- `error.rs` - Custom `IpCalcError` enum with `Result<T>` type alias used throughout
- `output.rs` - `TextOutput` trait for JSON/text formatting

**Data structures** (`Ipv4Subnet`, `Ipv6Subnet`) are serializable and shared between CLI and API.

## Code Patterns

- Error handling: `thiserror` derive macros, all functions return `Result<T>`
- Logging: `tracing` with `#[instrument]` on API handlers
- CLI: clap derive with subcommands (`v4`, `v6`, `split`, `serve`)
- Tests: Unit tests in modules, integration tests in `tests/` call binary via subprocess

## CLI Commands

```bash
ipcalc v4 192.168.1.0/24              # IPv4 subnet info
ipcalc v6 2001:db8::/48               # IPv6 prefix info
ipcalc split 10.0.0.0/8 -p 16 -n 10   # Generate 10 /16 subnets
ipcalc split 10.0.0.0/8 -p 16 --max   # Generate all possible /16 subnets
ipcalc serve                          # Start API server
```

Global options: `--format json|text`, `--output <file>`
