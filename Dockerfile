# Build stage
FROM rust:1.83-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev

WORKDIR /app

# Copy manifests first for better caching
COPY Cargo.toml Cargo.lock ./

# Create dummy src to build dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy actual source code
COPY src ./src
COPY tests ./tests

# Build the release binary
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM alpine:3.21

# Install runtime dependencies (if any)
RUN apk add --no-cache ca-certificates

# Create non-root user
RUN addgroup -g 1000 ipcalc && \
    adduser -u 1000 -G ipcalc -s /bin/sh -D ipcalc

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/ipcalc /usr/local/bin/ipcalc

# Switch to non-root user
USER ipcalc

# Default port for API server
EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD wget -qO- http://localhost:8080/health || exit 1

# Default command
ENTRYPOINT ["ipcalc"]
CMD ["--help"]
