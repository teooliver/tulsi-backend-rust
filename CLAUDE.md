# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

The canonical project conventions (architecture, entities, endpoints, layer rules, "adding a new resource" checklist) live in `AGENTS.md` and are imported here — keep `AGENTS.md` as the single source of truth and update it after structural changes.

@AGENTS.md

## Commands

```sh
docker compose up -d db db-test redis      # deps only (dev DB 5432, test DB 5433, Redis 6379)
cargo run -p server                        # API on :8080, migrations run on startup
PORT=9090 cargo run -p server              # alternate port
cargo run -p seed                          # populate dev DB with fake data

cargo build --workspace
cargo clippy --workspace --all-targets     # lints configured in [workspace.lints]
cargo fmt --all

cargo test --workspace                     # needs db-test running on 5433
cargo test -p api --test http_task_test                    # one test file (each tests/*.rs is its own binary)
cargo test -p api --test http_task_test create_task        # single test by name filter
```

## Gotchas

- `default-members = ["crates/server"]` in the root `Cargo.toml`: a bare `cargo test` / `cargo build` only covers the `server` crate. Use `--workspace` or `-p api` to run the integration tests.
- Integration tests are fully serialized via a Postgres advisory lock in `crates/api/tests/common/mod.rs` and wipe the DB with a `TRUNCATE` before each test. When adding a new table that isn't reachable via `CASCADE` from the listed tables, add it to that `TRUNCATE` statement. Background in `docs/test-database.md`.
- Tasks are never physically deleted: `DELETE /tasks/{id}` sets `deleted_at`. Any new query reading or updating `tasks` must add `deleted_at IS NULL` (see `docs/adr/0001-deleted-tasks-are-kept.md`).
- `setup_test_app()` builds the app with `cache = None`, so Redis caching paths and cache-key invalidation in repositories are **not** exercised by the test suite. When changing a repository write method, manually check that every affected cache key (e.g. `boards:all`, per-id keys, and related list keys like a board's projects) is deleted. See `docs/redis-caching.md`.
- Handlers that reference the auth user via FK (creating tasks, task history, saved views) need a real registered user — use `auth_token()`/`authed_server()`. There is no `POST /users`; create extra users in HTTP tests with `common::register_user()`. `mint_token()`/`fake_authed_server()` skip the DB and only work for handlers that don't persist the user id. Repository tests that create tasks get an Author from `common::make_user(&pool)`.
- `JWT_SECRET` is read in both `api::build_app()` and the test helper `mint_token()`; they must agree (both default to `dev-secret-change-me`).

## Further docs

- `OPENAPI.md` — utoipa usage and generating frontend TS types from `/api-docs/openapi.json`
- `OBSERVABILITY.md` — Prometheus / OpenTelemetry setup
- `docs/redis-caching.md`, `docs/test-database.md`
- `docs/improvements.md` — backlog of known weaknesses deferred for later; add an entry when you spot one you're not fixing
