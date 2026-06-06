# Test Database Setup

## Overview

Integration tests (`crates/api/tests/`) run against a **dedicated Postgres
instance**, the `db-test` service, separate from the dev/runtime `db` service.
This is not redundancy for its own sake — the test harness wipes the entire
database between every test, so it needs a throwaway DB that is never your dev
data.

| | `db` (dev/runtime) | `db-test` (tests) |
|---|---|---|
| Host port | 5432 | 5433 |
| Database name | `tulsi_test_db` | `tulsi_test_db_test` |
| Volume | `pgdata` | `pgdata_test` |
| Used by | `cargo run -p server`, the `app` container | `cargo test` via `TEST_DATABASE_URL` |

The separate **volume** matters too (`docker-compose.yml`): the test DB's data
lives in `pgdata_test`, so the constant truncating and migrating never touches
`pgdata`.

Tests connect via `TEST_DATABASE_URL`, defaulting to
`postgres://db_user_test:12345@localhost:5433/tulsi_test_db_test`
(`crates/api/tests/common/mod.rs`). The dev container pre-sets this variable, so
tests find `db-test` automatically.

## Why the database is wiped between tests

Every test calls `setup_test_db()`, which truncates all tables and resets
identity sequences before running:

```rust
sqlx::raw_sql(
    "TRUNCATE tasks, columns, projects, boards, users, plans, labels RESTART IDENTITY CASCADE",
)
```

This is the standard way to guarantee a clean, deterministic starting state per
test. But it is destructive: if tests pointed at the dev `db` on port 5432,
running `cargo test` would silently wipe everything you had been working with
locally. Hence the dedicated instance.

## Serializing tests with a session-level advisory lock

"Serialize" here means **force the tests to run one at a time instead of in
parallel**. The advisory lock is the mechanism that enforces it.

### The problem it solves

Rust runs tests concurrently by default — multiple threads within a test binary,
and `cargo test --workspace` runs the separate test binaries (each `*_test.rs`
compiles to its own binary) alongside each other too. So several tests can hit
`db-test` at the same moment.

Because every test starts with a global `TRUNCATE`, overlapping tests cause a
data race *in the database*:

```
Test A: TRUNCATE
Test A: register a user
Test B: TRUNCATE          ← wipes A's user out from under it
Test A: GET /tasks → expects A's data → FAILS (it's gone)
```

That is the classic source of flaky, "passes alone but fails in the suite"
tests.

### What an advisory lock is

A Postgres **advisory lock** is an application-defined mutex that Postgres
tracks by a number you choose (`8723451276` here — arbitrary, but it must be the
*same* number in every test so they all compete for the same lock). It does not
lock any actual table or row; it is purely a named flag that Postgres promises
only one session can hold at a time. `pg_advisory_lock(key)` **blocks** until it
can acquire it.

So the flow in `setup_test_db()` (`crates/api/tests/common/mod.rs`) becomes:

```
Test A: pg_advisory_lock(...)  → acquired immediately
Test A: TRUNCATE, run migrations, do its work
Test B: pg_advisory_lock(...)  → BLOCKS, waits for A
Test A: finishes, lock released
Test B: pg_advisory_lock(...)  → now acquired, TRUNCATE, do its work
```

Each test grabs the lock at the top, so only one is ever inside the
truncate-and-run critical section. That is the serialization.

### Why a database lock and not a Rust `Mutex`

A Rust `static Mutex` only synchronizes threads within one process. But the
contention is across **separate test binaries** (separate OS processes) — the
code comment calls this out as "across binaries." Those processes share nothing
in memory; the only thing they all share is the `db-test` database. So the lock
has to live where the shared resource is.

### Why "session-level", and the single-connection pool

Postgres has two flavors of advisory lock:

- `pg_advisory_xact_lock` — auto-releases at the end of the current transaction
- `pg_advisory_lock` (used here) — **session-level**: held until you explicitly
  unlock *or the connection/session ends*

This is where `max_connections(1)` comes in. The pool has exactly one
connection, so the lock is tied to that one session. The code never calls
`pg_advisory_unlock` — instead it relies on the `pool` variable going out of
scope at the end of the test: the pool drops → the single connection closes →
the session ends → Postgres **automatically releases the lock**.

The payoff: even if a test **panics**, the pool still drops during unwinding,
the connection still closes, and the next test is not left waiting forever on a
lock nobody will release. It is self-cleaning by construction rather than
relying on explicit teardown.

## Trade-offs and alternatives

This approach makes the DB-touching tests effectively **sequential**, so the
suite is slower than fully parallel tests. It was chosen for its simplicity and
robustness (self-releasing on panic, works across binaries with no shared
in-process state).

Common alternatives that preserve parallelism, at the cost of more setup
complexity:

- **Transaction-per-test with rollback**: each test runs inside a transaction
  that is never committed, so its changes vanish automatically. Fast and
  isolated, but awkward when the code under test manages its own transactions.
- **Schema- or database-per-test**: each test gets its own namespace, fully
  isolated. More isolation, but more setup/teardown machinery and connection
  overhead.

## Summary

`db-test` is wiped clean between every test, and the session-level advisory lock
guarantees those wipes never overlap with another test's work. Together they
give each test a pristine, exclusive database — which is only a safe strategy
*because* it runs against a dedicated throwaway instance and not the dev `db`.
