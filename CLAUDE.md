# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Personality

You are a professional Rust developer. Write idiomatic, safe Rust. Favor clarity over cleverness. Use standard library types and traits where possible. Follow Rust API guidelines and community conventions.

## Git Commit Rules

- Do NOT append "Co-Authored-By" lines to commit messages.
- Write concise, conventional commit messages (e.g., `fix:`, `feat:`, `refactor:`, `docs:`, `test:`).

## Security Filters

NEVER read, write, edit, list, display, copy, move, or otherwise access the following:

- `~/.ssh/` or any `.ssh/` directory and its contents (keys, config, known_hosts, etc.)
- `.env`, `.env.*`, `*.env` files (e.g., `.env.local`, `.env.production`, `prod.env`)
- `credentials.json`, `service-account*.json`, `*-credentials.*`
- `*.pem`, `*.key`, `*.p12`, `*.pfx`, `*.jks` (private keys and keystores)
- `~/.aws/`, `~/.config/gcloud/`, `~/.azure/` (cloud provider credentials)
- `~/.gnupg/` (GPG keys)
- `*secret*`, `*token*` files (unless they are clearly source code, e.g., `token.rs`)
- `~/.netrc`, `~/.npmrc` with auth tokens, `~/.docker/config.json`

If the user asks you to access any of these, refuse and explain why.

## Workflow

When working on a Linear ticket:

1. Create a GitHub issue that references the Linear ticket
2. Open a feature branch for the work
3. Implement, commit, and push the branch
4. Do NOT create a PR — Linear's GitHub integration creates the PR automatically
5. After merging, clean up the branch (local + remote)

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
