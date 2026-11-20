# Tasks: bring the code in line with GLOSSARY.md

Each file is a self-contained task. Do them in this order (later ones assume earlier migrations exist), one commit each.

| # | Task | Depends on |
|---|------|------------|
| 05 | [Record Task created and deleted History Events](./05-record-task-created-and-deleted-events.md) | 04 (done) |

Rules that apply to every task (from `AGENTS.md`):

- Never edit a committed migration. Add a new file using the **next unused number** in `migrations/` (011 at the time of writing; if several tasks land, each takes the next number).
- SQL lives only in `crates/repositories/`. Handlers stay thin.
- The test suite runs with Redis disabled, so cache invalidation is **not** tested. Check cache keys by hand.
- When done, update `AGENTS.md` (entities table, relationships, endpoints) so it describes the new state, and run `cargo clippy --workspace --all-targets` and `cargo test --workspace` (needs `docker compose up -d db-test`).
- Delete the task file when it's done.
