# 01 — Remove the Project → Board link

## Why

`GLOSSARY.md`: a **Project** is not tied to any **Board**; its Tasks may sit on different Boards. A Task is on a Board only through its **Column**. The code still has a `projects.board_id` column and an endpoint listing a Board's Projects. Nothing in the API can set `board_id` (neither `CreateProject` nor `UpdateProject` accepts it), so in practice it is always `NULL`.

## What to change

1. **Migration** `migrations/0NN_drop_board_id_from_projects.sql`:
   ```sql
   ALTER TABLE projects DROP COLUMN IF EXISTS board_id;
   ```
2. **Model** `crates/models/src/project.rs`: remove `board_id` from `Project`.
3. **Repository** `crates/repositories/src/board_repository.rs`:
   - Delete `find_projects` (≈line 116) and the `Project` import if it becomes unused.
   - In `delete` (≈line 107–110), stop deleting the `board:{id}:projects` cache key.
4. **Handler** `crates/api/src/handlers/board_handler.rs`: delete `list_board_projects` (≈line 121–145) and its `#[utoipa::path]`.
5. **Route** `crates/api/src/routes/board_routes.rs`: remove the `/boards/{id}/projects` route.
6. **OpenAPI** `crates/api/src/lib.rs`: remove `handlers::board_handler::list_board_projects` from `ApiDoc` `paths(...)`.
7. **Tests**:
   - `crates/api/tests/board_repository_test.rs`: delete `test_find_projects` (≈line 165), which inserts `board_id` directly.
   - `crates/api/tests/http_board_test.rs`: delete `list_board_projects_returns_empty_for_new_board` and `list_board_projects_not_found_returns_404` (≈line 207–235).
8. **Docs** `AGENTS.md`: remove `board_id` from the Project row, remove "A **Board** has many **Projects**", and remove the `GET /boards/{id}/projects` endpoint row.

## Done when

- `GET /boards/{id}/projects` returns 404 (no route).
- `Project` JSON has no `board_id`.
- `grep -rn "board_id" crates/models/src/project.rs crates/repositories/src/project_repository.rs` finds nothing, and `grep -rn "find_projects\|:projects" crates` finds nothing.
- Clippy and the test suite pass.
