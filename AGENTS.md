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

This project strictly follows the **Repository Pattern**. All layers are separated:

```
src/
├── main.rs                          # Entrypoint: DB connection, migrations, Redis, server startup
├── lib.rs                           # build_app(): wires repositories, routes, middleware, OpenAPI
├── auth.rs                          # JWT issuance/verification, require_auth middleware
├── cache.rs                         # RedisCache wrapper used by repositories
├── observability.rs                 # tracing + OTLP init, /health and /metrics handlers
├── models/                          # Domain types and DTOs (auth, user, project, board, column,
│                                    # task, task_history, plan, label)
├── repositories/                    # One *_repository.rs per resource — all SQL lives here
├── handlers/                        # Axum handler functions per resource
└── routes/                          # Router builders per resource
migrations/                          # Numbered SQL files (001_..010_); applied via sqlx::migrate!
crates/tulsi-seed/                   # Workspace member for seed data tooling
tests/                               # Integration tests; tests/common/mod.rs has shared setup
docker-compose.yml                   # Postgres (dev + test), Redis, app
```

### Entities & Relationships

| Entity | Table | Key Fields |
|--------|-------|------------|
| **User** | `users` | `id` (UUID), `name`, `email` (UNIQUE), `password_hash`, `created_at`, `updated_at` |
| **Board** | `boards` | `id` (UUID), `name`, `description`, `created_at`, `updated_at` |
| **Project** | `projects` | `id` (UUID), `name`, `description`, `board_id` (nullable FK → `boards`), `created_at`, `updated_at` |
| **Column** | `columns` | `id` (UUID), `name`, `position`, `board_id` (FK → `boards`), `created_at`, `updated_at` |
| **Task** | `tasks` | `id` (UUID), `title`, `description`, `project_id` (nullable FK → `projects`), `column_id` (nullable FK → `columns`), `author` (nullable UUID → `users`), `created_at`, `updated_at` |
| **TaskHistory** | `task_history` | `id`, `task_id` (FK → `tasks`), `user_id` (FK → `users`), `event_type`, `payload` (JSONB), `created_at` |
| **Plan** | `plans` | Saved task-filter configurations owned by a user |
| **Label** | `labels` | `id` (UUID), `name` (UNIQUE), `color` (nullable), `created_at`, `updated_at` |
| **TaskLabel** | `task_labels` | `task_id` (FK → `tasks`), `label_id` (FK → `labels`) — many-to-many join |

- A **Board** has many **Projects** and many **Columns**.
- A **Project** can have many **Tasks** (one-to-many). `project_id` on tasks is nullable; deleting a project sets it to `NULL`.
- A **Task** optionally lives in a **Column** (Kanban placement) and is authored by a **User**.
- **TaskHistory** records change events per task — written by handlers on create/update/delete and column moves.
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
| GET | `/users/{id}/tasks` | List tasks authored by a user |
| GET / POST / GET / PUT / DELETE | `/projects`, `/projects/{id}` | Project CRUD |
| GET | `/projects/{id}/tasks` | List tasks for a project |
| GET / POST / GET / PUT / DELETE | `/boards`, `/boards/{id}` | Board CRUD |
| GET | `/boards/{id}/projects` | List projects on a board |
| GET / POST / GET / PUT / DELETE | `/tasks`, `/tasks/{id}` | Task CRUD |
| GET | `/tasks/{id}/history` | Task change history |
| GET | `/boards/{board_id}/columns` | List columns for a board |
| POST / GET / PUT / DELETE | `/columns`, `/columns/{id}` | Column CRUD |
| GET | `/columns/{id}/tasks` | List tasks in a column |
| POST | `/columns/{id}/tasks/{task_id}` | Move task to column |
| GET / POST / GET / PUT / DELETE | `/plans`, `/plans/{id}` | Plan CRUD |
| POST | `/plans/{id}/execute` | Execute a saved plan and return matching tasks |
| GET / POST / GET / PUT / DELETE | `/labels`, `/labels/{id}` | Label CRUD |
| GET / POST / DELETE | `/tasks/{task_id}/labels[/{label_id}]` | Task ↔ label associations |

See `/swagger-ui` for the authoritative, generated spec.

### Layer Responsibilities

| Layer | Directory | Responsibility |
|-------|-----------|---------------|
| **Models** | `src/models/` | Domain types, DTOs. Derive `Serialize`, `Deserialize`, `FromRow`, and `ToSchema` (utoipa) as needed. |
| **Repositories** | `src/repositories/` | All database queries live here. Each repository takes a `PgPool` (and an optional `RedisCache`) and exposes async methods returning `Result<T, sqlx::Error>`. |
| **Handlers** | `src/handlers/` | Axum handler functions. Extract state (`Arc<Repository>`), path params, and JSON bodies. Map repo results to HTTP responses. Annotate with `#[utoipa::path(...)]` for OpenAPI. |
| **Routes** | `src/routes/` | Build `axum::Router` with route paths, methods, and shared state. Wired into `build_app()` in `lib.rs`. |
| **Auth** | `src/auth.rs` | `require_auth` middleware verifies the JWT and injects `AuthUser` into request extensions. |
| **Cache** | `src/cache.rs` | `RedisCache` is passed to repositories; cache misses fall back to the DB transparently. |

### Adding a New Resource

When adding a new resource (e.g., `Tag`, `Comment`), follow this pattern:

1. **Model** — Create `src/models/<resource>.rs` with the domain struct and any Create/Update DTOs. Re-export from `src/models/mod.rs`. Derive `ToSchema` if the type appears in handler signatures.
2. **Migration** — Add a new SQL file in `migrations/` with the next sequence number (e.g., `011_create_<resource>.sql`). It is picked up automatically by `sqlx::migrate!("./migrations")` — no `main.rs` change needed. Migrations are immutable once committed; create a new file to alter applied schema.
3. **Repository** — Create `src/repositories/<resource>_repository.rs` implementing CRUD methods. Re-export from `src/repositories/mod.rs`. Wire into `build_app()` in `lib.rs`.
4. **Handlers** — Create `src/handlers/<resource>_handler.rs` with Axum handler functions and `#[utoipa::path(...)]` annotations. Re-export from `src/handlers/mod.rs`. Add each handler to the `paths(...)` list in `lib.rs::ApiDoc`.
5. **Routes** — Create `src/routes/<resource>_routes.rs` returning a `Router`. Merge it into the appropriate (public vs protected) group in `build_app()`.

### Conventions

- Repositories are shared across handlers via `Arc<Repository>` passed as Axum state.
- Route path parameters use `{param}` syntax (Axum 0.8).
- Handlers return `Result<impl IntoResponse, StatusCode>`.
- Use `sqlx::query_as` for typed queries — no compile-time checked macros (no `DATABASE_URL` at build time).
- Migrations are applied at startup via `sqlx::migrate!("./migrations").run(&pool)` and tracked in the `_sqlx_migrations` table.
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
cargo run
```

The server listens on `http://0.0.0.0:8080`. Swagger UI: `http://localhost:8080/swagger-ui`.

## Testing

Integration tests live under `tests/` and use the dedicated `db-test` instance on port 5433. The shared helper `tests/common/mod.rs` exposes:

- `setup_test_db()` — connects, applies migrations via `sqlx::migrate!`, and truncates all tables before each test. Uses a Postgres advisory lock on a single-connection pool to serialize tests across binaries running in parallel.
- `setup_test_app()` — returns a `Router` backed by the test pool.
- `authed_server()` / `fake_authed_server()` — `TestServer` pre-loaded with an `Authorization: Bearer` header.
- `auth_token()` / `mint_token()` — JWT helpers.

```sh
cargo test
```

## Guidelines for AI Agents

- **Always follow the Repository Pattern.** Never put SQL queries in handlers or route files.
- **Do not introduce ORMs.** Use SQLx directly.
- **Keep handlers thin.** They should only extract inputs, call the repository, and map results to HTTP responses.
- **Use `Arc<T>` for sharing repositories** via Axum's `State` extractor.
- **Postgres and Redis run in Docker.** Any new infrastructure dependencies should be added to `docker-compose.yml`.
- **Do not use `sqlx::query!` macros** (compile-time checked queries). Use `sqlx::query_as` or `sqlx::query` — there is no build-time database connection configured.
- **Never edit a migration that has already been committed.** Add a new numbered migration file to evolve the schema. `sqlx::migrate!` validates checksums against `_sqlx_migrations`.
- **Naming:** files use `snake_case`. Structs use `PascalCase`. Follow the existing `<resource>_repository.rs`, `<resource>_handler.rs`, `<resource>_routes.rs` pattern.
- **Error handling:** Repositories return `Result<T, sqlx::Error>`. Handlers map errors to appropriate `StatusCode` values.
- **OpenAPI:** new handlers must be annotated with `#[utoipa::path(...)]` and registered in `lib.rs::ApiDoc`.
- **Auth:** new endpoints are protected by default — merge into the protected group in `build_app()`. Only add to public if the route is genuinely unauthenticated (health, metrics, swagger, auth).
- **No `.env` files in the repo.** Use environment variables or hardcoded defaults for local development.
- **Keep `AGENTS.md` up to date.** After making structural changes (new entities, endpoints, migrations, architectural decisions), update this file to reflect the current state of the project.
