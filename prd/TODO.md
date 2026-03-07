# TODO

## Database Schema Setup

- [x] **SQLite schema auto-migration** — `SqliteStore::migrate()` creates all tables (`supernets`, `allocations`, `allocation_tags`, `audit_log`, `schema_version`) via embedded migrations on startup.
- [x] **PostgreSQL backend** — `PostgresStore` in `src/ipam/postgres/`, feature-gated behind `ipam-postgres`. Uses `sqlx` with `PgPool`, embedded migrations matching SQLite schema. Supports `--ipam-db-url`, `IPCALC_IPAM_DB_URL` env, and `[ipam.postgres]` config section.
- [x] **Migration versioning for PostgreSQL** — Uses same embedded Rust constants pattern as SQLite with `schema_version` table for tracking applied migrations.
- [ ] **MySQL backend** — Not yet implemented. PRD Phase 6 calls for `src/ipam/mysql/` with `sqlx` + `mysql` feature. Feature-gated behind `ipam-mysql`.
- [ ] **Trait-contract test suite for backend parity** — PRD specifies a shared test suite run against `&dyn IpamStore` per backend (SQLite in-memory by default, Postgres/MySQL via testcontainers). Not yet implemented as a reusable harness.
- [ ] **PostgreSQL integration tests** — Requires a running PostgreSQL instance (testcontainers or local). Current tests only cover SQLite.
