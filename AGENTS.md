# AGENTS.md — Tulsi Rust Backend

## Project Overview

Tulsi is a task management backend built with **Rust**, **Axum**, and **PostgreSQL**. The project follows the **Repository Pattern** to separate data access logic from business logic and HTTP handling.

## Tech Stack

- **Language:** Rust (edition 2024)
- **Web Framework:** Axum 0.8
- **Database:** PostgreSQL 16 (via SQLx with compile-time query checking disabled — uses `query_as` at runtime)
- **Async Runtime:** Tokio
- **Serialization:** Serde / serde_json
- **IDs:** UUID v4
- **Timestamps:** chrono (UTC)
- **Logging:** tracing + tracing-subscriber
- **CORS:** tower-http

## Architecture

This project strictly follows the **Repository Pattern**. All layers are separated:

```
src/
├── main.rs                       # Entrypoint: DB connection, migrations, server startup
├── models/
│   ├── mod.rs
│   ├── project.rs                # Project, CreateProject, UpdateProject
│   └── task.rs                   # Task, CreateTask, UpdateTask
├── repositories/
│   ├── mod.rs
│   ├── project_repository.rs     # Project CRUD + find tasks by project
│   └── task_repository.rs        # Task CRUD (supports project_id)
├── handlers/
│   ├── mod.rs
│   ├── project_handler.rs        # Project HTTP handlers
│   └── task_handler.rs           # Task HTTP handlers
├── routes/
│   ├── mod.rs
│   ├── project_routes.rs         # Project route definitions
│   └── task_routes.rs            # Task route definitions
migrations/
├── 001_create_tasks.sql          # Tasks table
├── 002_create_projects.sql       # Projects table + project_id FK on tasks
docker-compose.yml                # Postgres container
```

### Entities & Relationships

| Entity | Table | Key Fields |
|--------|-------|------------|
| **Project** | `projects` | `id` (UUID), `name`, `description`, `created_at`, `updated_at` |
| **Task** | `tasks` | `id` (UUID), `title`, `description`, `project_id` (nullable FK → `projects`), `created_at`, `updated_at` |

- A **Project** can have many **Tasks** (one-to-many).
- A **Task** optionally belongs to a **Project** (`project_id` is nullable).
- Deleting a project sets `project_id` to `NULL` on linked tasks (`ON DELETE SET NULL`).

### API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/tasks` | List all tasks |
| POST | `/tasks` | Create a task (optional `project_id`) |
| GET | `/tasks/{id}` | Get a task |
| PUT | `/tasks/{id}` | Update a task |
| DELETE | `/tasks/{id}` | Delete a task |
| GET | `/projects` | List all projects |
| POST | `/projects` | Create a project |
| GET | `/projects/{id}` | Get a project |
| PUT | `/projects/{id}` | Update a project |
| DELETE | `/projects/{id}` | Delete a project |
| GET | `/projects/{id}/tasks` | List tasks for a project |

### Layer Responsibilities

| Layer | Directory | Responsibility |
|-------|-----------|---------------|
| **Models** | `src/models/` | Domain types, DTOs. Derive `Serialize`, `Deserialize`, `FromRow` as needed. |
| **Repositories** | `src/repositories/` | All database queries live here. Each repository takes a `PgPool` and exposes async methods returning `Result<T, sqlx::Error>`. |
| **Handlers** | `src/handlers/` | Axum handler functions. Extract state (`Arc<Repository>`), path params, and JSON bodies. Map repo results to HTTP responses. |
| **Routes** | `src/routes/` | Build `axum::Router` with route paths, methods, and shared state. |

### Adding a New Resource

When adding a new resource (e.g., `Project`, `User`), follow this pattern:

1. **Model** — Create `src/models/<resource>.rs` with the domain struct and any Create/Update DTOs. Re-export from `src/models/mod.rs`.
2. **Migration** — Add a new SQL file in `migrations/` with the next sequence number (e.g., `002_create_projects.sql`). Add the `include_str!` + `raw_sql` call in `main.rs`.
3. **Repository** — Create `src/repositories/<resource>_repository.rs` implementing CRUD methods. Re-export from `src/repositories/mod.rs`.
4. **Handlers** — Create `src/handlers/<resource>_handler.rs` with Axum handler functions. Re-export from `src/handlers/mod.rs`.
5. **Routes** — Create `src/routes/<resource>_routes.rs` returning a `Router`. Wire it into `main.rs` using `.merge()` or `.nest()`.

### Conventions

- Repositories are shared across handlers via `Arc<Repository>` passed as Axum state.
- Route path parameters use `{param}` syntax (Axum 0.8).
- Handlers return `Result<impl IntoResponse, StatusCode>`.
- Use `sqlx::query_as` for typed queries — no compile-time checked macros.
- Migrations are applied at startup via `sqlx::raw_sql(include_str!(...))`.

## Database

PostgreSQL runs in Docker. Start it with:

```sh
docker compose up -d
```

Connection defaults (from `docker-compose.yml`):
- **Host:** localhost
- **Port:** 5432
- **User:** db_user_test
- **Password:** 12345
- **Database:** tulsi_test_db

Override at runtime via the `DATABASE_URL` environment variable.

## Running the Project

```sh
# Start Postgres
docker compose up -d

# Run the server
cargo run
```

The server listens on `http://0.0.0.0:3000`.

## Guidelines for AI Agents

- **Always follow the Repository Pattern.** Never put SQL queries in handlers or route files.
- **Do not introduce ORMs.** Use SQLx directly with raw SQL queries.
- **Keep handlers thin.** They should only extract inputs, call the repository, and map results to HTTP responses.
- **Use `Arc<T>` for sharing repositories** via Axum's `State` extractor.
- **Postgres must run in Docker.** Any new infrastructure dependencies should be added to `docker-compose.yml`.
- **Do not use `sqlx::query!` macros** (compile-time checked queries). Use `sqlx::query_as` or `sqlx::query` instead, as there is no build-time database connection configured.
- **Naming:** files use `snake_case`. Structs use `PascalCase`. Follow the existing `<resource>_repository.rs`, `<resource>_handler.rs`, `<resource>_routes.rs` naming pattern.
- **Error handling:** Repositories return `Result<T, sqlx::Error>`. Handlers map errors to appropriate `StatusCode` values.
- **No `.env` files in the repo.** Use environment variables or hardcoded defaults for local development.
- **Keep `AGENTS.md` up to date.** After making structural changes (new entities, endpoints, migrations, architectural decisions), update this file to reflect the current state of the project.
