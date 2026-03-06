# IPAM Persistence Layer — TODO

Reference: `prd/prd-ipam-persistence.md`
Branch: `feat/ipam-persistence-layer`

---

## Phase 1: Storage Trait & SQLite Backend — DONE

- [x] `src/ipam/store.rs` — `IpamStore` async trait
- [x] `src/ipam/models.rs` — Supernet, Allocation, Tag, AuditEntry, filter/input types
- [x] `src/ipam/operations.rs` — `IpamOps` business logic (conflict detection, auto-allocation, free space, utilization, IP/resource lookup, audit)
- [x] `src/ipam/config.rs` — `IpamConfig`, `SqliteConfig`, DB path resolution
- [x] `src/ipam/output.rs` — `TextOutput` + `CsvOutput` for IPAM types
- [x] `src/ipam/sqlite/mod.rs` — `SqliteStore` with r2d2 pool, WAL mode
- [x] `src/ipam/sqlite/migrations.rs` — embedded schema migrations
- [x] `src/ipam/mod.rs` — public API, `create_store()` factory
- [x] 23 tests (supernet CRUD, allocation lifecycle, conflict detection, auto-allocation, free space, utilization, IP lookup, audit log, tags, config resolution, range arithmetic)

## Phase 1.5: Pre-merge Housekeeping — DONE

- [x] Open PR for Phase 1 (PR #48, merged)
- [x] Add `[Unreleased]` entry in `CHANGELOG.md` for IPAM persistence layer
- [x] Update `README.md` with IPAM overview
- [x] Create `src/validation.rs` shared input validation module (35 tests)

## Phase 2: CLI Integration — DONE

- [x] Add `ipam` subcommand group to `cli.rs` via clap derive
- [x] Supernet commands: `ipam supernet create|list|get|delete`
- [x] Allocation commands: `ipam allocate`, `ipam auto-allocate`
- [x] Release command: `ipam release <allocation-id>`
- [x] Query commands: `ipam allocation get|list`
- [x] Update command: `ipam allocation update <id> --owner --env ...`
- [x] Free space command: `ipam free-blocks <supernet-id> [--prefix N]`
- [x] Utilization command: `ipam utilization <supernet-id>`
- [x] Lookup commands: `ipam find-ip <address>`, `ipam find-resource <id>`
- [x] Audit command: `ipam audit [--entity-type --entity-id --action --limit]`
- [x] Tags commands: `ipam tags get|set`
- [x] Wire all subcommands to `ipam::operations` via `ipam_cli.rs`
- [x] Respect `--format json|text|csv|yaml` and `--output <file>`
- [x] DB path flag: `--db <path>` on `ipam` subcommand
- [x] 8 integration tests: supernet lifecycle, allocation workflow, utilization, find-ip, audit, tags, overlap rejection, CSV output (PR #49)

## Phase 3: API Integration

- [ ] Add `/ipam/` route group to `api.rs`
- [ ] Inject `Arc<dyn IpamStore>` into Axum state
- [ ] `POST /ipam/supernets` — create supernet
- [ ] `GET /ipam/supernets` — list supernets
- [ ] `GET /ipam/supernets/:id` — get supernet detail
- [ ] `DELETE /ipam/supernets/:id` — delete supernet
- [ ] `POST /ipam/supernets/:id/allocate` — auto-allocate block(s)
- [ ] `POST /ipam/supernets/:id/allocate-specific` — allocate specific CIDR
- [ ] `GET /ipam/supernets/:id/allocations` — list allocations in supernet
- [ ] `GET /ipam/supernets/:id/free` — free blocks
- [ ] `GET /ipam/supernets/:id/utilization` — utilization stats
- [ ] `GET /ipam/allocations/:id` — get allocation detail
- [ ] `PATCH /ipam/allocations/:id` — update allocation metadata
- [ ] `POST /ipam/allocations/:id/release` — release allocation
- [ ] `GET /ipam/find-ip/:address` — find allocation by IP
- [ ] `GET /ipam/find-resource/:resource_id` — find by resource
- [ ] `GET /ipam/audit` — query audit log
- [ ] Swagger/OpenAPI schema for all IPAM endpoints
- [ ] `--ipam-enabled`, `--ipam-backend`, `--ipam-db` flags on `serve`
- [ ] API integration tests (tower oneshot pattern)

## Phase 4: MCP IPAM Tools

- [ ] `ipam_create_supernet` tool
- [ ] `ipam_list_supernets` tool
- [ ] `ipam_allocate` tool
- [ ] `ipam_allocate_specific` tool
- [ ] `ipam_release` tool
- [ ] `ipam_list_allocations` tool
- [ ] `ipam_free_blocks` tool
- [ ] `ipam_utilization` tool
- [ ] `ipam_find_ip` tool
- [ ] `ipam_find_resource` tool
- [ ] MCP integration tests for IPAM tools

## Phase 5: Free Space & Utilization Enhancements

- [ ] Gap-walking algorithm: load active allocations sorted by network address, identify gaps
- [ ] Aligned CIDR fitting: for each gap, compute largest aligned blocks that fit
- [ ] Target prefix mode: given a /N, count how many fit in available space
- [ ] Utilization rollup: allocated vs total IPs, breakdown by status (active/reserved/released)

## Phase 6: Additional Storage Backends

### PostgreSQL (`ipam-postgres` feature)
- [ ] `src/ipam/postgres/mod.rs` — `PostgresStore` implementation
- [ ] `sqlx` with `postgres` runtime, async connection pooling
- [ ] Native `INET`/`CIDR` column types
- [ ] Row-level locking for multi-writer concurrency
- [ ] Embedded `sqlx` migrations
- [ ] Trait contract tests via testcontainer

### MySQL (`ipam-mysql` feature)
- [ ] `src/ipam/mysql/mod.rs` — `MysqlStore` implementation
- [ ] `sqlx` with `mysql` runtime
- [ ] Embedded `sqlx` migrations
- [ ] Trait contract tests via testcontainer

## Cross-cutting / Deferred

- [x] `src/validation.rs` — shared input scrubbing (CIDR, IPs, prefix lengths, freeform text)
- [ ] Property-based tests for conflict detection (random CIDR pairs, no false negatives)
- [ ] Fuzz targets for CIDR inputs to allocation functions
- [ ] Migration upgrade path tests (v0 -> vN with sample data)
- [ ] JSON export/import (`ipam dump` / `ipam load`) — deferred to v2
- [ ] IPv6 IPAM implementation (schema supports it, code is IPv4-only for v1)
- [ ] Reservation TTL/expiry — deferred to v2
- [x] Replace Node.js MCP server with Rust-native implementation using `rmcp` (official Rust SDK) — eliminates Node dependency, reduces supply-chain attack surface, calls library functions directly
- [ ] MCP server remote backend option — add a configuration flag (e.g. `--api-url`) so the MCP server can proxy tool calls to a running `ipcalc serve` HTTP API instead of calling local library functions directly. Useful when the MCP server runs on a different host or when IPAM state must be shared across clients.
