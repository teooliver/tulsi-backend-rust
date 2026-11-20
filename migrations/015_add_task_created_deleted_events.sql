-- Task History also records a task being created and deleted. Existing tasks
-- are not backfilled.
ALTER TYPE task_event_type ADD VALUE IF NOT EXISTS 'task_created';
ALTER TYPE task_event_type ADD VALUE IF NOT EXISTS 'task_deleted';
