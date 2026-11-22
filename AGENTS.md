# AGENTS.md — Tulsi Rust Backend

## Project Overview

Tulsi is a task management backend built with **Rust**, **Axum**, and **PostgreSQL**. The project follows the **Repository Pattern** to separate data access logic from business logic and HTTP handling.

## Tech Stack

- **Language:** Rust (edition 2024)
- **Web Framework:** Axum 0.8
- **Database:** PostgreSQL 16 (via SQLx, runtime-checked queries — `query_as` / `query`, no compile-time macros)
- **Migrations:** `sqlx::migrate!` against the `migrations/` directory (tracked in `_sqlx_migrations`)
- **Cache:** Redis 7 (optional — app runs without it)
- **Async Runtime:** Tokio
- **Auth:** JWT (`jsonwebtoken`) + Argon2 password hashing
- **Serialization:** Serde / serde_json
- **IDs:** UUID v4
- **Timestamps:** chrono (UTC)
- **Logging / Tracing:** tracing + tracing-subscriber, OpenTelemetry OTLP (optional)
- **Metrics:** `metrics` + Prometheus exporter at `/metrics`
- **API Docs:** utoipa + Swagger UI at `/swagger-ui`
- **CORS:** tower-http (permissive in dev)

## Architecture

This project strictly follows the **Repository Pattern**, split across a Cargo
**workspace** of focused crates so the layers are enforced by the crate graph
(and so editing a handler doesn't recompile the heavy OpenTelemetry stack):

```
Cargo.toml                           # Virtual workspace manifest + [workspace.dependencies]
migrations/                          # Numbered SQL files (001_..015_); applied via sqlx::migrate!
crates/
├── models/        src/lib.rs + *.rs # Domain types and DTOs (auth, user, project, board, column,
│                                    # task, task_history, saved_view, label)
├── auth/          src/lib.rs        # JWT issuance/verification, require_auth middleware, AuthUser
├── cache/         src/lib.rs        # RedisCache wrapper used by repositories
├── observability/ src/lib.rs        # tracing + OTLP init, /health and /metrics handlers
├── repositories/  src/lib.rs + *.rs # One *_repository.rs per resource — all SQL lives here
│                                    #   (deps: models, cache)
├── api/           src/lib.rs        # build_app() + OpenAPI doc; handlers/ and routes/ submodules
│                  src/handlers/     # Axum handler functions per resource
│                  src/routes/       # Router builders per resource
│                  tests/            # Integration tests; tests/common/mod.rs has shared setup
│                                    #   (deps: models, auth, cache, observability, repositories)
├── server/        src/main.rs       # Binary: DB connection, migrations, Redis, server startup
└── seed/          src/main.rs       # Standalone seed-data binary (cargo run -p seed)
docker-compose.yml                   # Postgres (dev + test), Redis, app
```

Dependency direction is acyclic: `models`/`auth`/`cache`/`observability` are
leaf crates; `repositories` builds on `models`+`cache`; `api` builds on all of
them; `server` is a thin binary over `api`.

### Entities & Relationships

| Entity | Table | Key Fields |
|--------|-------|------------|
| **User** | `users` | `id` (UUID), `name`, `email` (UNIQUE), `password_hash`, `created_at`, `updated_at` |
| **Board** | `boards` | `id` (UUID), `name`, `description`, `created_at`, `updated_at` |
| **Project** | `projects` | `id` (UUID), `name`, `description`, `created_at`, `updated_at` |
| **Column** | `columns` | `id` (UUID), `name`, `position`, `board_id` (FK → `boards`), `created_at`, `updated_at` |
| **Task** | `tasks` | `id` (UUID), `title`, `description`, `project_id` (nullable FK → `projects`), `column_id` (nullable FK → `columns`), `author` (nullable FK → `users`; set from the logged-in user on create, never changes), `assigned_to` (nullable FK → `users`), `deleted_at` (nullable; set when the task is deleted), `created_at`, `updated_at` |
| **TaskHistory** | `task_history` | `id`, `task_id` (FK → `tasks`), `user_id` (FK → `users`), `event_type` (`task_event_type` enum), `old_value` (nullable text), `new_value` (nullable text), `created_at` |
| **SavedView** | `saved_views` | `id`, `name`, `description`, `owner_id` (FK → `users`), `filters` (JSONB), `created_at`, `updated_at` — saved task filters owned by a user |
| **Label** | `labels` | `id` (UUID), `name` (UNIQUE), `color` (nullable), `created_at`, `updated_at` |
| **TaskLabel** | `task_labels` | `task_id` (FK → `tasks`), `label_id` (FK → `labels`) — many-to-many join |

- A **Board** has many **Columns**. A **Task** is on a Board only through its Column; **Projects** are not linked to Boards.
- A **Project** can have many **Tasks** (one-to-many). `project_id` on tasks is nullable; deleting a project sets it to `NULL`.
- A **Task** optionally lives in a **Column** (Kanban placement) has an optional **Author** and an optional **Assignee** (both **Users**).
- Deleting a **Task** marks it deleted (`deleted_at`) instead of removing the row, so its history survives. Every task query must filter `deleted_at IS NULL`; a deleted task behaves as not found everywhere except `GET /tasks/{id}/history` (see `docs/adr/0001-deleted-tasks-are-kept.md`).
- **TaskHistory** records a task's life — written by the task create, update, delete and move handlers. Event types: `task_created`, `task_deleted`, `title_changed`, `description_changed`, `column_changed`, `assignment_changed`, `project_changed` (snake_case in the database, PascalCase such as `TaskCreated` in JSON and the `?event_type=` filter). Label changes are **not** recorded.
- A **Task** can have many **Labels** and vice versa via `task_labels` (many-to-many). Deleting a task or label cascades to the join rows.

### API Endpoints

Public:

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | Liveness check (also pings DB) |
| GET | `/metrics` | Prometheus metrics |
| GET | `/swagger-ui` | OpenAPI / Swagger UI |
| POST | `/auth/register` | Register a new user, returns JWT |
| POST | `/auth/login` | Log in, returns JWT |
| GET | `/auth/me` | Current user (requires JWT) |

Protected (require `Authorization: Bearer <jwt>`):

| Method | Path | Description |
|--------|------|-------------|
| GET / POST / GET / PUT / DELETE | `/users`, `/users/{id}` | User CRUD |
| GET | `/users/{id}/tasks` | List tasks assigned to a user |
| GET / POST / GET / PUT / DELETE | `/projects`, `/projects/{id}` | Project CRUD |
| GET | `/projects/{id}/tasks` | List tasks for a project |
| GET / POST / GET / PUT / DELETE | `/boards`, `/boards/{id}` | Board CRUD |
| GET / POST / GET / PUT / DELETE | `/tasks`, `/tasks/{id}` | Task CRUD |
| GET | `/tasks/{id}/history` | Task change history |
| GET / POST | `/boards/{board_id}/columns` | List / create columns for a board |
| GET / PUT / DELETE | `/boards/{board_id}/columns/{column_id}` | Column read / update / delete |
| GET | `/boards/{board_id}/columns/{column_id}/tasks` | List tasks in a column |
| PUT | `/tasks/{task_id}/move` | Move task to a column (body: `{ "column_id": ... }`) |
| GET / POST / GET / PUT / DELETE | `/saved-views`, `/saved-views/{id}` | Saved View CRUD (scoped to the owner) |
| GET | `/saved-views/{id}/tasks` | Execute a Saved View and return matching tasks |
| GET / POST / GET / PUT / DELETE | `/labels`, `/labels/{id}` | Label CRUD |
| GET / POST / DELETE | `/tasks/{task_id}/labels[/{label_id}]` | Task ↔ label associations |

See `/swagger-ui` for the authoritative, generated spec.

### Layer Responsibilities

| Layer | Location | Responsibility |
|-------|----------|---------------|
| **Models** | `crates/models/` | Domain types, DTOs. Derive `Serialize`, `Deserialize`, `FromRow`, and `ToSchema` (utoipa) as needed. |
| **Repositories** | `crates/repositories/` | All database queries live here. Each repository takes a `PgPool` (and an optional `RedisCache`) and exposes async methods returning `Result<T, sqlx::Error>`. |
| **Handlers** | `crates/api/src/handlers/` | Axum handler functions. Extract state (`Arc<Repository>`), path params, and JSON bodies. Map repo results to HTTP responses. Annotate with `#[utoipa::path(...)]` for OpenAPI. |
| **Routes** | `crates/api/src/routes/` | Build `axum::Router` with route paths, methods, and shared state. Wired into `build_app()` in `crates/api/src/lib.rs`. |
| **Auth** | `crates/auth/` | `require_auth` middleware verifies the JWT and injects `AuthUser` into request extensions. |
| **Cache** | `crates/cache/` | `RedisCache` is passed to repositories; cache misses fall back to the DB transparently. |

### Adding a New Resource

When adding a new resource (e.g., `Tag`, `Comment`), follow this pattern:

1. **Model** — Create `crates/models/src/<resource>.rs` with the domain struct and any Create/Update DTOs. Re-export from `crates/models/src/lib.rs`. Derive `ToSchema` if the type appears in handler signatures.
2. **Migration** — Add a new SQL file in `migrations/` with the next sequence number (e.g., `011_create_<resource>.sql`). It is picked up automatically by `sqlx::migrate!` (wired in `crates/server/src/main.rs`) — no code change needed. Migrations are immutable once committed; create a new file to alter applied schema.
3. **Repository** — Create `crates/repositories/src/<resource>_repository.rs` implementing CRUD methods. Re-export from `crates/repositories/src/lib.rs`. Wire into `build_app()` in `crates/api/src/lib.rs`.
4. **Handlers** — Create `crates/api/src/handlers/<resource>_handler.rs` with Axum handler functions and `#[utoipa::path(...)]` annotations. Re-export from `crates/api/src/handlers/mod.rs`. Add each handler to the `paths(...)` list in `crates/api/src/lib.rs::ApiDoc`.
5. **Routes** — Create `crates/api/src/routes/<resource>_routes.rs` returning a `Router`. Merge it into the appropriate (public vs protected) group in `build_app()`.

### Conventions

- Repositories are shared across handlers via `Arc<Repository>` passed as Axum state.
- Route path parameters use `{param}` syntax (Axum 0.8).
- Handlers return `Result<impl IntoResponse, StatusCode>`.
- Use `sqlx::query_as` for typed queries — no compile-time checked macros (no `DATABASE_URL` at build time).
- Migrations are applied at startup via `sqlx::migrate!("../../migrations").run(&pool)` (in `crates/server/src/main.rs`; the path is relative to the crate, and the `migrations/` dir stays at the workspace root) and tracked in the `_sqlx_migrations` table.
- Protected routes go under the `require_auth` middleware group in `build_app()`. Public routes (health, metrics, swagger, auth endpoints) go in the public group.

## Database

PostgreSQL runs in Docker. Two instances:

```sh
docker compose up -d   # starts db (5432), db-test (5433), redis (6379)
```

| Service | Port | Database |
|---------|------|----------|
| `db` (dev) | 5432 | `tulsi_test_db` |
| `db-test` | 5433 | `tulsi_test_db_test` |
| `redis` | 6379 | — |

Defaults (overridable via env):
- `DATABASE_URL` — defaults to `postgres://db_user_test:12345@localhost:5432/tulsi_test_db`
- `TEST_DATABASE_URL` — defaults to `postgres://db_user_test:12345@localhost:5433/tulsi_test_db_test`
- `REDIS_URL` — optional; unset means run without cache
- `JWT_SECRET` — defaults to `dev-secret-change-me` (override in production)
- `OTEL_EXPORTER_OTLP_ENDPOINT` — optional; unset disables OTLP export
- `PORT` — defaults to `8080`

## Running the Project

```sh
docker compose up -d
cargo run -p server
```

The server listens on `http://0.0.0.0:8080`. Swagger UI: `http://localhost:8080/swagger-ui`.

## Testing

Integration tests live under `crates/api/tests/` and use the dedicated `db-test` instance on port 5433. The shared helper `crates/api/tests/common/mod.rs` exposes:

- `setup_test_db()` — connects, applies migrations via `sqlx::migrate!`, and truncates all tables before each test. Uses a Postgres advisory lock on a single-connection pool to serialize tests across binaries running in parallel.
- `setup_test_app()` — returns a `Router` backed by the test pool.
- `authed_server()` / `fake_authed_server()` — `TestServer` pre-loaded with an `Authorization: Bearer` header.
- `auth_token()` / `mint_token()` — JWT helpers.

```sh
cargo test --workspace
```

## Guidelines for AI Agents

- **Use the domain vocabulary in `../../tulsi-home/GLOSSARY.md`** (the Tulsi Home Repo, checked out beside this repo's parent folder; if you cloned this repo alone, read it at https://github.com/teooliver/tulsi-home/blob/main/GLOSSARY.md). Name things with its canonical terms in code, docs and responses, and avoid the synonyms it lists.

- **Always follow the Repository Pattern.** Never put SQL queries in handlers or route files.
- **Do not introduce ORMs.** Use SQLx directly.
- **Keep handlers thin.** They should only extract inputs, call the repository, and map results to HTTP responses.
- **Use `Arc<T>` for sharing repositories** via Axum's `State` extractor.
- **Postgres and Redis run in Docker.** Any new infrastructure dependencies should be added to `docker-compose.yml`.
- **Do not use `sqlx::query!` macros** (compile-time checked queries). Use `sqlx::query_as` or `sqlx::query` — there is no build-time database connection configured.
- **Never edit a migration that has already been committed.** Add a new numbered migration file to evolve the schema. `sqlx::migrate!` validates checksums against `_sqlx_migrations`.
- **Naming:** files use `snake_case`. Structs use `PascalCase`. Follow the existing `<resource>_repository.rs`, `<resource>_handler.rs`, `<resource>_routes.rs` pattern.
- **Error handling:** Repositories return `Result<T, sqlx::Error>`. Handlers map errors to appropriate `StatusCode` values.
- **OpenAPI:** new handlers must be annotated with `#[utoipa::path(...)]` and registered in `crates/api/src/lib.rs::ApiDoc`.
- **Auth:** new endpoints are protected by default — merge into the protected group in `build_app()`. Only add to public if the route is genuinely unauthenticated (health, metrics, swagger, auth).
- **No `.env` files in the repo.** Use environment variables or hardcoded defaults for local development.
- **Keep `AGENTS.md` up to date.** After making structural changes (new entities, endpoints, migrations, architectural decisions), update this file to reflect the current state of the project.
