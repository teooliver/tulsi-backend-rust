# Tasks: bring the code in line with GLOSSARY.md

Each file is a self-contained task. Do them in this order (later ones assume earlier migrations exist), one commit each.

| # | Task | Depends on |
|---|------|------------|
| 01 | [Remove the Project → Board link](./01-remove-project-board-link.md) | — |
| 02 | [Author is the Task's creator](./02-author-is-the-creator.md) | — |
| 03 | [Rename Plan to Saved View](./03-rename-plan-to-saved-view.md) | — |
| 04 | [Keep Deleted Tasks instead of removing them](./04-keep-deleted-tasks.md) | — |
| 05 | [Record Task created and deleted History Events](./05-record-task-created-and-deleted-events.md) | 04 |

Rules that apply to every task (from `AGENTS.md`):

- Never edit a committed migration. Add a new file using the **next unused number** in `migrations/` (011 at the time of writing; if several tasks land, each takes the next number).
- SQL lives only in `crates/repositories/`. Handlers stay thin.
- The test suite runs with Redis disabled, so cache invalidation is **not** tested. Check cache keys by hand.
- When done, update `AGENTS.md` (entities table, relationships, endpoints) so it describes the new state, and run `cargo clippy --workspace --all-targets` and `cargo test --workspace` (needs `docker compose up -d db-test`).
- Delete the task file when it's done.
