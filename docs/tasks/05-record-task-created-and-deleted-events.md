# 05 — Record Task created and deleted History Events

**Depends on task 04** (Deleted Tasks are kept; otherwise the "deleted" event is cascade-deleted with the Task).

## Why

`GLOSSARY.md`: **Task History** is the record of a Task's whole life, including its creation and deletion. The **Event Type** list includes the Task being created or deleted. Today only the five field-change events exist (`crates/models/src/task_history.rs` `TaskEventType`, Postgres enum `task_event_type` from migration `008`).

## What to change

1. **Migration** `migrations/0NN_add_task_created_deleted_events.sql`:
   ```sql
   ALTER TYPE task_event_type ADD VALUE IF NOT EXISTS 'task_created';
   ALTER TYPE task_event_type ADD VALUE IF NOT EXISTS 'task_deleted';
   ```
   On Postgres 16 this works inside sqlx's migration transaction, as long as the new values aren't *used* in the same migration. Don't backfill history for existing tasks.
2. **Model** `crates/models/src/task_history.rs`: add `TaskCreated` and `TaskDeleted` to `TaskEventType`. It is `rename_all = "snake_case"`, so these map to the new enum values automatically. They are also filterable via `?event_type=task_created` on `GET /tasks/{id}/history`.
3. **Handlers** `crates/api/src/handlers/task_handler.rs`:
   - `create_task`: it doesn't take the history repo yet. Add `Extension(history_repo): Extension<Arc<TaskHistoryRepository>>` like `update_task`, and after a successful create, record `TaskCreated` with `user_id = auth_user.user_id`, `old_value = None`, `new_value = Some(task.title.clone())`.
   - `delete_task`: add `auth_user: AuthUser` and the history repo extension. After a successful delete (`Ok(true)`), record `TaskDeleted` with `old_value = Some(<title>)` and `new_value = None`. To get the title, either `find_by_id` before deleting, or have `TaskRepository::delete` return the deleted `Task` (`Option<Task>`) via `RETURNING *`. The latter avoids a second query and fits the `RETURNING` already needed for cache invalidation in task 04.
   - Follow the existing pattern in `record_task_changes`: a failure to record history is logged with `tracing::warn!` and does **not** fail the request.
   - Check that the `Extension<Arc<TaskHistoryRepository>>` layer in `build_app()` (`crates/api/src/lib.rs`) covers the routes for `POST /tasks` and `DELETE /tasks/{id}`. It's added for `update_task` today, so it most likely does.
4. **Tests** (`crates/api/tests/http_task_test.rs`, use `authed_server()`, since `task_history.user_id` is a foreign key to `users`, so `fake_authed_server()` won't work):
   - After `POST /tasks`, the history has exactly one `task_created` event, by the current user, with `new_value` = the title.
   - After `DELETE /tasks/{id}`, `GET /tasks/{id}/history` ends with a `task_deleted` event with `old_value` = the title.
   - `?event_type=task_created` filters correctly.
   - Existing tests that assert an exact history length after updates may now be off by one, because of the `task_created` event. Fix them.
5. **Docs** `AGENTS.md`: update the TaskHistory bullet. Event types now include `task_created` and `task_deleted`, written by the create and delete handlers; label changes are still not recorded.

## Done when

New tests pass, the existing history tests have been adjusted, and Clippy and the full suite pass.
