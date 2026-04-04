# Redis Caching (Cache-Aside Pattern)

## Overview

Redis caching was added to the Rust backend using the cache-aside pattern:
- **Reads**: Check Redis first → fall back to Postgres → write result to Redis (60s TTL)
- **Writes**: Invalidate relevant cache keys (individual entity + list keys)

Redis is optional — if `REDIS_URL` is not set or Redis is unreachable, the app runs without caching. All Redis failures are logged as warnings and silently ignored (graceful degradation).

## Changes

### New file: `src/cache.rs`

`RedisCache` wrapper around `redis::aio::ConnectionManager` with generic methods:
- `get<T>(key)` — deserialize cached JSON, return `None` on miss or error
- `set<T>(key, value)` — serialize to JSON and store with 60s TTL
- `delete(keys)` — remove one or more keys

### `Cargo.toml`

Added `redis` 0.27 with `tokio-comp` and `connection-manager` features.

### `main.rs`

Connects to Redis via the `REDIS_URL` environment variable. If the var is missing or connection fails, the app runs without cache.

### `lib.rs`

`build_app()` now accepts `Option<RedisCache>` and passes it to all repositories.

### All 5 repositories

Each repository now takes `Option<RedisCache>` as a second constructor argument:

- **Reads** (`find_all`, `find_by_id`, `find_tasks`, `find_projects`, `find_by_board_id`): Check Redis first → fall back to Postgres → write result to Redis
- **Writes** (`create`, `update`, `delete`, `move_task`): Invalidate relevant cache keys

#### Cache key scheme

| Key pattern                      | Data                          |
|----------------------------------|-------------------------------|
| `task:{id}`                      | Single task                   |
| `tasks:all`                      | All tasks list                |
| `board:{id}`                     | Single board                  |
| `boards:all`                     | All boards list               |
| `board:{id}:projects`            | Board's projects              |
| `board:{id}:columns`             | Board's columns               |
| `project:{id}`                   | Single project                |
| `projects:all`                   | All projects list             |
| `project:{id}:tasks`             | Project's tasks               |
| `user:{id}`                      | Single user                   |
| `users:all`                      | All users list                |
| `user:{id}:tasks`                | User's assigned tasks         |
| `column:{id}`                    | Single column                 |
| `column:{id}:tasks`              | Column's tasks                |

### `docker-compose.yml`

Added `redis:7-alpine` service on port 6379 with a persistent volume. The app container has `REDIS_URL=redis://redis:6379` set.

### Tests

All tests pass `None` for the cache parameter — they hit Postgres directly without Redis.

## Usage

- **K8s**: Set `REDIS_URL=redis://redis-service:6379` as an env var on the backend deployment (if in the `homelab` namespace), or `redis://redis-service.homelab.svc.cluster.local:6379` for cross-namespace access
- **Local with docker-compose**: Already configured, just `docker compose up`
- **Local without Redis**: Don't set `REDIS_URL` — the app works exactly as before
