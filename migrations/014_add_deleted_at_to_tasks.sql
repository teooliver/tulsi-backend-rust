-- Deleting a task marks it instead of removing the row, so its history
-- survives (see docs/adr/0001-deleted-tasks-are-kept.md).
ALTER TABLE tasks ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ;
CREATE INDEX IF NOT EXISTS idx_tasks_live ON tasks(created_at) WHERE deleted_at IS NULL;
