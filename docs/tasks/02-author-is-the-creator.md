# 02 — Author is the Task's creator

## Why

`GLOSSARY.md`: the **Author** is the User who created a Task. It is set once, at creation, and never changes.

Current state:
- `create_task` (`crates/api/src/handlers/task_handler.rs` ≈line 72) already overwrites `input.author` with the logged-in user. Good, keep that.
- But `CreateTask` and `UpdateTask` (`crates/models/src/task.rs`) both still expose `author`, so it appears in the API docs as client-settable, and `PUT /tasks/{id}` **can change it** (`task_repository.rs` `update`, ≈line 83: `author = CASE WHEN $6 THEN $7 ELSE author END`).
- `tasks.author` has **no foreign key** to `users` (migration `006` added a bare `UUID`), unlike `assigned_to`.

## What to change

1. **Model** `crates/models/src/task.rs`:
   - Remove `author` from `UpdateTask`.
   - Remove `author` from `CreateTask`. Pass the Author to the repository separately: change `TaskRepository::create(&self, input: CreateTask)` to `create(&self, author: Uuid, input: CreateTask)`. The handler passes `auth_user.user_id`.
2. **Repository** `crates/repositories/src/task_repository.rs`:
   - `create`: bind the new `author` argument.
   - `update`: remove the `author = CASE ...` line and renumber the binds after it.
3. **Migration** `migrations/0NN_add_author_fk_to_tasks.sql`. Existing rows may hold ids of users that don't exist, so null those first:
   ```sql
   UPDATE tasks SET author = NULL
   WHERE author IS NOT NULL AND author NOT IN (SELECT id FROM users);

   ALTER TABLE tasks
       ADD CONSTRAINT tasks_author_fkey
       FOREIGN KEY (author) REFERENCES users(id) ON DELETE SET NULL;
   ```
   `ON DELETE SET NULL` matches `assigned_to`: deleting a User leaves their Tasks in place without an Author. `author` stays nullable for that reason.
4. **Seed** `crates/seed/src/main.rs` (≈line 65) inserts tasks without an author. Set the author to one of the seeded users.
5. **Tests**: every `CreateTask { ..., author: None, ... }` / `UpdateTask { ... }` literal in `crates/api/tests/*_repository_test.rs` must drop the field. Repository tests that call `create` need a real user id now that there is a foreign key: create a user with `UserRepository` first. `grep -rn "author:" crates/api/tests` lists them (task, user, project, column, label, plan repository tests).
   Add HTTP tests in `crates/api/tests/http_task_test.rs` (use `authed_server()`, since the Author must be a real user):
   - Creating a Task sets `author` to the logged-in user, even if the body includes another `author`.
   - `PUT /tasks/{id}` with `"author": "<other uuid>"` leaves `author` unchanged.
6. **Docs** `AGENTS.md`: change the Task row's `author` to `(nullable FK → users, set from the logged-in user on create, immutable)`.

## Done when

- The OpenAPI schema for `CreateTask` and `UpdateTask` has no `author`.
- The two new HTTP tests pass, as do Clippy and the full suite.
