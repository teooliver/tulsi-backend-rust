-- Author is the User who created the Task. Clear ids that don't match an
-- existing user so the foreign key can be added.
UPDATE tasks SET author = NULL
WHERE author IS NOT NULL AND author NOT IN (SELECT id FROM users);

ALTER TABLE tasks
    ADD CONSTRAINT tasks_author_fkey
    FOREIGN KEY (author) REFERENCES users(id) ON DELETE SET NULL;
