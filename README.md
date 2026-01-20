# ipcalc

[![CI](https://github.com/wingnut128/ipcalc/actions/workflows/ci.yml/badge.svg)](https://github.com/wingnut128/ipcalc/actions/workflows/ci.yml)
[![CodeQL](https://github.com/wingnut128/ipcalc/actions/workflows/codeql.yml/badge.svg)](https://github.com/wingnut128/ipcalc/actions/workflows/codeql.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A fast IPv4 and IPv6 subnet calculator written in Rust. Available as both a CLI tool and HTTP API.

## Features

- **IPv4 subnet calculations**: network address, broadcast, subnet mask, wildcard mask, host ranges, network class detection
- **IPv6 prefix calculations**: network address, address ranges, hextet breakdown, address type detection (global unicast, link-local, ULA, etc.)
- **Subnet splitting**: generate N subnets of a given prefix from a supernet
- **Interactive TUI**: Terminal user interface with real-time calculations and split mode (optional feature)
- **Multiple output formats**: JSON (default) or plain text
- **File output**: write results directly to a file
- **HTTP API**: REST endpoints for all calculations
- **OpenAPI documentation**: Machine-readable API specification for easy integration with tools like Swagger Editor, Postman, and Insomnia

## Installation

### From Source

```bash
git clone https://github.com/wingnut128/ipcalc.git
cd ipcalc
cargo build --release
```

The binary will be at `target/release/ipcalc`.

### Using Cargo

```bash
cargo install --path .
```

## Usage

### Subnet Calculation

The CLI auto-detects IPv4 or IPv6 based on the CIDR notation:

```bash
# JSON output (default)
ipcalc 192.168.1.0/24

# Plain text output
ipcalc 192.168.1.0/24 --format text

# Output to file
ipcalc 10.0.0.0/8 -o results.json

# IPv6 prefix
ipcalc 2001:db8::/32
ipcalc fe80::1/64 --format text
```

Example JSON output:
```json
{
  "input": "192.168.1.0/24",
  "network_address": "192.168.1.0",
  "broadcast_address": "192.168.1.255",
  "subnet_mask": "255.255.255.0",
  "wildcard_mask": "0.0.0.255",
  "prefix_length": 24,
  "first_host": "192.168.1.1",
  "last_host": "192.168.1.254",
  "total_hosts": 256,
  "usable_hosts": 254,
  "network_class": "C",
  "is_private": true
}
```

### Subnet Splitting

Generate smaller subnets from a larger supernet:

```bash
# Generate 10 /27 subnets from a /22
ipcalc split 192.168.0.0/22 -p 27 -n 10

# Generate 5 /48 subnets from a /32
ipcalc split 2001:db8::/32 -p 48 -n 5
```

### Interactive TUI

Launch an interactive terminal user interface for real-time subnet calculations and splitting:

```bash
# Build with TUI support
cargo build --release --features tui

# Run the TUI
ipcalc --tui
```

**TUI Features:**

- **Calculate Mode**: Enter any CIDR notation for instant subnet information display
  - Network address, netmask, broadcast address
  - First/last host, total hosts
  - Real-time validation and updates

- **Split Mode**: Interactive subnet splitting with live results
  - Press **TAB** to switch between Calculate and Split modes
  - Enter CIDR, target prefix length, and count
  - Press **M** to toggle MAX mode for generating all possible subnets
  - Use **↑↓** arrow keys to scroll through generated subnet lists
  - Press **ENTER** to cycle through input fields

- **Keyboard Controls**:
  - `TAB` - Switch between Calculate and Split modes
  - `ENTER` - Move to next input field (Split mode)
  - `M` - Toggle MAX mode for subnet count (Split mode)
  - `↑↓` - Scroll through results
  - `ESC` - Quit

The TUI automatically detects IPv4/IPv6 and provides color-coded input fields with real-time error messages.

**Note:** The TUI feature is optional and must be enabled at build time with the `tui` feature flag. It is not included in the default build to keep the binary size smaller.

### HTTP API Server

```bash
# Start server on default port 8080
ipcalc serve

# Custom address and port
ipcalc serve --address 0.0.0.0 --port 3000

# With logging
ipcalc serve --log-level debug --log-file /var/log/ipcalc.log
```

#### API Endpoints

| Endpoint | Description | Example |
|----------|-------------|---------|
| `GET /health` | Health check | `/health` |
| `GET /version` | Version information | `/version` |
| `GET /v4?cidr=<cidr>` | IPv4 calculation | `/v4?cidr=192.168.1.0/24` |
| `GET /v6?cidr=<cidr>` | IPv6 calculation | `/v6?cidr=2001:db8::/32` |
| `GET /v4/split?cidr=<cidr>&prefix=<n>&count=<n>` | Split IPv4 supernet | `/v4/split?cidr=10.0.0.0/8&prefix=16&count=5` |
| `GET /v6/split?cidr=<cidr>&prefix=<n>&count=<n>` | Split IPv6 supernet | `/v6/split?cidr=2001:db8::/32&prefix=48&count=10` |
| `GET /swagger-ui` | Interactive Swagger UI | `/swagger-ui` |
| `GET /api-docs/openapi.json` | OpenAPI 3.0 specification | `/api-docs/openapi.json` |

#### Example API Requests

```bash
# IPv4 calculation
curl "http://localhost:8080/v4?cidr=192.168.1.0/24"

# IPv6 calculation
curl "http://localhost:8080/v6?cidr=2001:db8::/32"

# Split a /22 into /27 subnets
curl "http://localhost:8080/v4/split?cidr=192.168.0.0/22&prefix=27&count=10"

# Get OpenAPI specification
curl "http://localhost:8080/api-docs/openapi.json"
```

#### OpenAPI Documentation

The API provides interactive Swagger UI documentation and a complete OpenAPI 3.0 specification:

```bash
# Access interactive Swagger UI in your browser
open http://localhost:8080/swagger-ui

# Get the OpenAPI spec
curl http://localhost:8080/api-docs/openapi.json > openapi.json

# Import into Postman
# Import the openapi.json file into Postman to generate a collection

# Import into Swagger Editor
# Visit https://editor.swagger.io and import the openapi.json file

# Use with other tools
# The spec is compatible with Insomnia, API clients, and code generators
```

**Interactive Features:**
- Try out API endpoints directly from the browser at `/swagger-ui`
- View request/response schemas with examples
- Execute requests and see live responses

**Building without OpenAPI support:**

The OpenAPI documentation feature is optional and enabled by default. To build a smaller binary without it:

```bash
cargo build --release --no-default-features
```

## CLI Reference

```
ipcalc [OPTIONS] [CIDR] [COMMAND]

Arguments:
  [CIDR]  IP address in CIDR notation (e.g., 192.168.1.0/24 or 2001:db8::/48)

Commands:
  split  Generate subnets from a supernet
  serve  Start the HTTP API server
  help   Print help for a command

Options:
  -f, --format <FORMAT>  Output format [default: json] [possible values: json, text]
  -o, --output <OUTPUT>  Output file path (prints to stdout if not specified)
      --tui              Launch interactive TUI mode (requires tui feature)
  -h, --help             Print help
  -V, --version          Print version
```

**Notes:**
- The legacy `v4` and `v6` subcommands are still supported for backwards compatibility but are deprecated
- The `--tui` flag is only available when built with the `tui` feature: `cargo build --features tui`

## Docker

```bash
# Build the image
docker build -t ipcalc .

# Run CLI
docker run --rm ipcalc 192.168.1.0/24

# Run API server
docker run --rm -p 8080:8080 ipcalc serve --address 0.0.0.0
```

## Development

```bash
# Setup git hooks (required for development)
make setup

# Build
make build

# Run tests
make test

# Run linter
make lint

# Build release binary
make release

# Build Docker image
make docker
```

The `make setup` command installs a pre-commit hook that automatically runs `cargo fmt --check` and `cargo clippy` before each commit.

## License

MIT License - see [LICENSE](LICENSE) for details.
