.PHONY: all build release test lint fmt clean docker docker-run help

# Variables
BINARY_NAME := ipcalc
VERSION := $(shell grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
DOCKER_IMAGE := ipcalc
DOCKER_TAG := $(VERSION)

# Default target
all: build

# Build debug binary
build:
	cargo build

# Build release binary
release:
	cargo build --release

# Run all tests
test:
	cargo test

# Run tests with output
test-verbose:
	cargo test -- --nocapture

# Run clippy linter
lint:
	cargo clippy -- -D warnings

# Format code
fmt:
	cargo fmt

# Check formatting without modifying
fmt-check:
	cargo fmt -- --check

# Clean build artifacts
clean:
	cargo clean

# Build Docker image
docker:
	docker build -t $(DOCKER_IMAGE):$(DOCKER_TAG) -t $(DOCKER_IMAGE):latest .

# Run Docker container (API server)
docker-run:
	docker run --rm -p 8080:8080 $(DOCKER_IMAGE):latest serve --address 0.0.0.0

# Install locally
install:
	cargo install --path .

# Uninstall
uninstall:
	cargo uninstall $(BINARY_NAME)

# Run the API server locally
serve:
	cargo run -- serve

# Run with debug logging
serve-debug:
	cargo run -- serve --log-level debug

# Check everything (format, lint, test)
check: fmt-check lint test

# CI pipeline target
ci: check
	cargo build --release

# Print version
version:
	@echo $(VERSION)

# Help
help:
	@echo "ipcalc Makefile"
	@echo ""
	@echo "Usage: make [target]"
	@echo ""
	@echo "Targets:"
	@echo "  build         Build debug binary"
	@echo "  release       Build release binary"
	@echo "  test          Run all tests"
	@echo "  test-verbose  Run tests with output"
	@echo "  lint          Run clippy linter"
	@echo "  fmt           Format code"
	@echo "  fmt-check     Check formatting"
	@echo "  clean         Clean build artifacts"
	@echo "  docker        Build Docker image"
	@echo "  docker-run    Run API server in Docker"
	@echo "  install       Install binary locally"
	@echo "  uninstall     Uninstall binary"
	@echo "  serve         Run API server locally"
	@echo "  serve-debug   Run API server with debug logging"
	@echo "  check         Run fmt-check, lint, and test"
	@echo "  ci            Full CI pipeline"
	@echo "  version       Print version"
	@echo "  help          Show this help"
