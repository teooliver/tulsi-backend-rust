# 04 — Keep Deleted Tasks instead of removing them

## Why

See `docs/adr/0001-deleted-tasks-are-kept.md`. Task History must be able to record a Task's deletion (task 05), but `task_history.task_id` is `ON DELETE CASCADE`, so a hard delete erases the history. `GLOSSARY.md` defines a **Deleted Task** as one that no longer appears anywhere and can't be changed, but whose History is kept.

## Behaviour

- `DELETE /tasks/{id}` marks the Task as deleted. It returns 204, or 404 if the Task doesn't exist or is **already** deleted.
- A Deleted Task is treated as not found everywhere: it is excluded from every list and from Saved View execution, and `GET`/`PUT /tasks/{id}`, `PUT /tasks/{id}/move` and the `/tasks/{task_id}/labels` endpoints return 404 for it.
- **Exception:** `GET /tasks/{id}/history` still returns a Deleted Task's history. That is the whole point.
- Label attachments and history rows of a Deleted Task are left as they are.
- `Task` JSON doesn't change: don't expose `deleted_at`. With `SELECT *` + `FromRow`, the extra column is simply ignored.

## What to change

1. **Migration** `migrations/0NN_add_deleted_at_to_tasks.sql`:
   ```sql
   ALTER TABLE tasks ADD COLUMN deleted_at TIMESTAMPTZ;
   CREATE INDEX IF NOT EXISTS idx_tasks_live ON tasks(created_at) WHERE deleted_at IS NULL;
   ```
2. **Repositories**: add `deleted_at IS NULL` to **every** query that reads or modifies tasks. As of writing, the full list is:
   | File | Query |
   |------|-------|
   | `task_repository.rs` | `find_all`, `find_by_id`, `update` (`WHERE id = $1 AND deleted_at IS NULL`, so it returns `None` → 404) |
   | `task_repository.rs` | `delete`: `UPDATE tasks SET deleted_at = NOW(), updated_at = NOW() WHERE id = $1 AND deleted_at IS NULL`; `rows_affected() > 0` |
   | `column_repository.rs` | tasks in a column (≈line 153); `move_task` read (≈173) and update (≈181) |
   | `project_repository.rs` | tasks in a project (≈line 126) |
   | `user_repository.rs` | tasks assigned to a user (≈line 155) |
   | `saved_view_repository.rs` | `find_tasks_for_saved_view`: start from `SELECT * FROM tasks WHERE deleted_at IS NULL` instead of `WHERE 1=1` |
   | `label_repository.rs` | `find_labels_for_task`, `attach_to_task`, `detach_from_task`: these must fail for a Deleted Task. The FK still passes, because the row exists. Either add a `TaskRepository::exists(id)` check in the label handlers that returns 404, or guard the SQL (`... WHERE EXISTS (SELECT 1 FROM tasks WHERE id = $1 AND deleted_at IS NULL)`). Match whatever 404 behaviour these endpoints have today for a nonexistent task. |
   Re-run `grep -rn "FROM tasks\|UPDATE tasks" crates/repositories/src` afterwards to make sure nothing was missed. The seed binary and tests may insert directly; those are fine.
3. **Cache invalidation on delete** (`task_repository.rs` `delete`, ≈line 119): it currently clears only `task:{id}` and `tasks:all`. A Deleted Task would keep appearing in the cached lists `column:{column_id}:tasks`, `project:{project_id}:tasks` and `user:{assigned_to}:tasks` for up to 60 s. Use `UPDATE ... RETURNING column_id, project_id, assigned_to` and delete those keys too.
4. **Handler** `crates/api/src/handlers/task_handler.rs` `delete_task`: no signature change needed. Update the OpenAPI description to say the Task is marked deleted and its history is kept.
5. **Tests** (`crates/api/tests/http_task_test.rs`, plus repository tests as needed):
   - After deleting: `GET /tasks/{id}` → 404, `GET /tasks` doesn't include it, a second `DELETE` → 404, `PUT` → 404.
   - It disappears from `/boards/{b}/columns/{c}/tasks`, `/projects/{id}/tasks`, `/users/{id}/tasks` and Saved View execution.
   - `GET /tasks/{id}/history` still returns its earlier events (e.g. make a title change before deleting).
   - Label endpoints on a Deleted Task → 404.
   - Any existing test asserting the row is physically gone (e.g. via `SELECT COUNT(*) FROM tasks`) needs updating.
6. **Docs** `AGENTS.md`: add `deleted_at` to the Task row and a relationship bullet: "Deleting a task marks it deleted (`deleted_at`) instead of removing it; all task queries must filter `deleted_at IS NULL` (see `docs/adr/0001-deleted-tasks-are-kept.md`)." Add the same rule to the Gotchas in `CLAUDE.md`.

## Done when

All the behaviours above have passing tests, the grep in step 2 shows a `deleted_at` filter on every task read and update, and Clippy and the full suite pass.
