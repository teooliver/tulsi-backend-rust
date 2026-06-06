# Tulsi Rust Backend

Axum + sqlx + Postgres backend for the Tulsi project management app. Provides JWT-authenticated CRUD APIs for tasks, projects, boards, columns, plans, labels, and users, with Swagger UI for interactive docs.

## Quick start with a Dev Container

The fastest way in: open the repo in a [Dev Container](https://containers.dev)-aware
editor (VS Code with the *Dev Containers* extension, or GitHub Codespaces) and
choose **Reopen in Container**. This builds a Rust toolchain container wired to
the Postgres, test-Postgres, and Redis services, with `DATABASE_URL`,
`TEST_DATABASE_URL`, and `REDIS_URL` already set. Once it finishes, just run:

```bash
cargo run -p server   # then open http://localhost:8080/swagger-ui/
cargo test --workspace
```

The manual setup below is for working without a dev container.

## Prerequisites

- Rust (stable, edition 2024)
- Docker + Docker Compose (for Postgres + Redis)

## 1. Start dependencies

Bring up Postgres (port 5432), the test DB (5433), and Redis (6379):

```bash
docker compose up -d db db-test redis
```

To run the full stack (app included) in Docker instead:

```bash
docker compose up -d
```

## 2. Run the server locally

Migrations run automatically on startup.

```bash
cargo run -p server
```

Default environment (overridable):

- `DATABASE_URL` — `postgres://db_user_test:12345@localhost:5432/tulsi_test_db`
- `REDIS_URL` — `redis://localhost:6379` (optional; app runs without it)
- `JWT_SECRET` — `dev-secret-change-me` (set this in production)
- `PORT` — `8080`

The server listens on `http://localhost:8080`.

## 3. Swagger UI

With the server running, open:

```
http://localhost:8080/swagger-ui/
```

Raw OpenAPI spec:

```
http://localhost:8080/api-docs/openapi.json
```

## 4. Initial curl checks

### Health

```bash
curl http://localhost:8080/health
```

### Register a user

```bash
curl -X POST http://localhost:8080/auth/register \
  -H "Content-Type: application/json" \
  -d '{"name":"Teo","email":"teo@example.com","password":"hunter2hunter2"}'
```

The response contains a `token`. Export it for the next calls:

```bash
TOKEN=$(curl -s -X POST http://localhost:8080/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"teo@example.com","password":"hunter2hunter2"}' \
  | jq -r .token)
```

### Current user

```bash
curl http://localhost:8080/auth/me \
  -H "Authorization: Bearer $TOKEN"
```

### List tasks (protected)

```bash
curl http://localhost:8080/tasks \
  -H "Authorization: Bearer $TOKEN"
```

### Create a project

```bash
curl -X POST http://localhost:8080/projects \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name":"My Project"}'
```

## 5. Tests

```bash
cargo test --workspace
```

The integration tests live in `crates/api/tests/` and use the `db-test` instance on port 5433. See `docs/test-database.md` for why tests need a separate DB and how they serialize against each other.

## Workspace layout

The backend is split into focused crates under `crates/` to keep compile times
down (editing a handler doesn't recompile the OpenTelemetry stack) and boundaries
clear:

| Crate | Responsibility |
|-------|----------------|
| `models` | Domain types & DTOs (serde / sqlx `FromRow` / utoipa `ToSchema`) |
| `auth` | JWT creation, `require_auth` middleware, `AuthUser` extractor |
| `cache` | `RedisCache` wrapper |
| `observability` | tracing + OTLP init, `/health` and `/metrics` handlers |
| `repositories` | All SQL; one `*_repository.rs` per resource (deps: `models`, `cache`) |
| `api` | Axum handlers, routes, `build_app()`, OpenAPI doc; hosts integration tests |
| `server` | Thin binary: config, migrations, Redis wiring, serves `api::build_app()` |
| `seed` | Standalone DB seeding binary (`cargo run -p seed`) |

## Useful files

- `OPENAPI.md` — why we use utoipa and how to generate TS types on the frontend
- `OBSERVABILITY.md` — Prometheus / OpenTelemetry setup
- `AGENTS.md` — project conventions
