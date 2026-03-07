# TODO

## Database Schema Setup

- [x] **SQLite schema auto-migration** — `SqliteStore::migrate()` creates all tables (`supernets`, `allocations`, `allocation_tags`, `audit_log`, `schema_version`) via embedded migrations on startup.
- [x] **PostgreSQL backend** — `PostgresStore` in `src/ipam/postgres/`, feature-gated behind `ipam-postgres`. Uses `sqlx` with `PgPool`, embedded migrations matching SQLite schema. Supports `--ipam-db-url`, `IPCALC_IPAM_DB_URL` env, and `[ipam.postgres]` config section.
- [x] **Migration versioning for PostgreSQL** — Uses same embedded Rust constants pattern as SQLite with `schema_version` table for tracking applied migrations.
- [x] **PostgreSQL integration tests** — Docker-based tests against `postgres:16-alpine` covering full CRUD, tags, audit log, and operations layer.
- [ ] **Trait-contract test suite for backend parity** — Shared test suite run against `&dyn IpamStore` per backend (SQLite in-memory by default, Postgres via Docker). Not yet implemented as a reusable harness.
