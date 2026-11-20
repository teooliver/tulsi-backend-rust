# Deleted Tasks are kept, not removed

Task History must record a Task being deleted, but history rows cascade-delete with their Task, so a hard delete would erase the very event it records. We decided that deleting a Task marks it as deleted (`tasks.deleted_at`) instead of removing the row: its History, Labels and references all survive, and every query that reads Tasks must exclude Deleted Tasks.

## Considered Options

- **Keep the row, mark it deleted (chosen).** History stays intact and can still show the Task's title. Cost: every Task read must filter `deleted_at IS NULL`, and forgetting one leaks Deleted Tasks into results.
- **Let history outlive the Task** (drop the foreign key from `task_history.task_id`). Simpler reads, but history would point at Tasks that no longer exist and have no title to show.
