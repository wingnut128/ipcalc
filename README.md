# ipcalc

[![CI](https://github.com/wingnut128/ipcalc/actions/workflows/ci.yml/badge.svg)](https://github.com/wingnut128/ipcalc/actions/workflows/ci.yml)
[![CodeQL](https://github.com/wingnut128/ipcalc/actions/workflows/codeql.yml/badge.svg)](https://github.com/wingnut128/ipcalc/actions/workflows/codeql.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A fast IPv4 and IPv6 subnet calculator written in Rust. Available as both a CLI tool and HTTP API.

## Features

- **IPv4 subnet calculations**: network address, broadcast, subnet mask, wildcard mask, host ranges, network class detection
- **IPv6 prefix calculations**: network address, address ranges, hextet breakdown, address type detection (global unicast, link-local, ULA, etc.)
- **Subnet splitting**: generate N subnets of a given prefix from a supernet
- **Multiple output formats**: JSON (default) or plain text
- **File output**: write results directly to a file
- **HTTP API**: REST endpoints for all calculations

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

### IPv4 Subnet Calculation

```bash
# JSON output (default)
ipcalc v4 192.168.1.0/24

# Plain text output
ipcalc v4 192.168.1.0/24 --format text

# Output to file
ipcalc v4 10.0.0.0/8 -o results.json
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

### IPv6 Prefix Calculation

```bash
ipcalc v6 2001:db8::/32
ipcalc v6 fe80::1/64 --format text
```

### Subnet Splitting

Generate smaller subnets from a larger supernet:

```bash
# Generate 10 /27 subnets from a /22
ipcalc split 192.168.0.0/22 -p 27 -n 10

# Generate 5 /48 subnets from a /32
ipcalc split 2001:db8::/32 -p 48 -n 5
```

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

#### Example API Requests

```bash
# IPv4 calculation
curl "http://localhost:8080/v4?cidr=192.168.1.0/24"

# IPv6 calculation
curl "http://localhost:8080/v6?cidr=2001:db8::/32"

# Split a /22 into /27 subnets
curl "http://localhost:8080/v4/split?cidr=192.168.0.0/22&prefix=27&count=10"
```

## CLI Reference

```
ipcalc [OPTIONS] <COMMAND>

Commands:
  v4     Calculate IPv4 subnet information
  v6     Calculate IPv6 subnet information
  split  Generate subnets from a supernet
  serve  Start the HTTP API server
  help   Print help for a command

Options:
  -f, --format <FORMAT>  Output format [default: json] [possible values: json, text]
  -o, --output <OUTPUT>  Output file path (prints to stdout if not specified)
  -h, --help             Print help
  -V, --version          Print version
```

## Docker

```bash
# Build the image
docker build -t ipcalc .

# Run CLI
docker run --rm ipcalc v4 192.168.1.0/24

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
