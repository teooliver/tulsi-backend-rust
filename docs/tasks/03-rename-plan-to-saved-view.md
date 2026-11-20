# 03 — Rename Plan to Saved View

## Why

`GLOSSARY.md` renamed **Plan** to **Saved View**: a named set of **Filters** owned by one User (its **Owner**), which can be **Executed** to list the Tasks that currently match. "Plan" is now a word to avoid, partly so it can't be confused with a future roadmap feature. The rename should go all the way through: database, Rust names, URLs and docs.

This is a breaking API change: `/plans` becomes `/saved-views`. Don't keep `/plans` as an alias.

## What to change

1. **Migration** `migrations/0NN_rename_plans_to_saved_views.sql`:
   ```sql
   ALTER TABLE plans RENAME TO saved_views;
   ALTER INDEX idx_plans_owner_id RENAME TO idx_saved_views_owner_id;
   ```
   Leave `009_create_plans.sql` untouched.
2. **Models**: rename `crates/models/src/plan.rs` → `saved_view.rs` and update `pub mod` in `crates/models/src/lib.rs`. Types:
   `Plan` → `SavedView`, `PlanResponse` → `SavedViewResponse`, `PlanFilters` → `SavedViewFilters`, `CreatePlan` → `CreateSavedView`, `UpdatePlan` → `UpdateSavedView`. The JSON field names (`filters`, `owner_id`, …) stay the same.
3. **Repository**: `crates/repositories/src/plan_repository.rs` → `saved_view_repository.rs`, `PlanRepository` → `SavedViewRepository`, update `crates/repositories/src/lib.rs`.
   - SQL: `plans` → `saved_views` in every query.
   - Cache keys: `plan:{id}` → `saved_view:{id}`, `plans:owner:{owner_id}` → `saved_views:owner:{owner_id}`. The prefix change means old cached entries simply expire (60 s TTL), so no flush is needed.
   - Method `execute` (builds `SELECT * FROM tasks WHERE 1=1 ...`) keeps its name.
4. **Handlers**: `crates/api/src/handlers/plan_handler.rs` → `saved_view_handler.rs`. Rename `list_plans`, `get_plan`, `create_plan`, `update_plan`, `delete_plan`, `execute_plan` → `list_saved_views`, `get_saved_view`, `create_saved_view`, `update_saved_view`, `delete_saved_view`, `execute_saved_view`. In each `#[utoipa::path]`, set `path = "/saved-views..."`, `tag = "Saved Views"`, and reword descriptions ("Plan" → "Saved View"). Update `handlers/mod.rs`.
5. **Routes**: `crates/api/src/routes/plan_routes.rs` → `saved_view_routes.rs` with `/saved-views`, `/saved-views/{id}`, `/saved-views/{id}/execute`. Update `routes/mod.rs`.
6. **App** `crates/api/src/lib.rs`: imports, `plan_repo` → `saved_view_repo`, the router merge, and in `ApiDoc` the `paths(...)` entries, the `components(schemas(...))` entries (`models::plan::*` → `models::saved_view::*`) and the tag `(name = "Plans", ...)` → `(name = "Saved Views", description = "Saved task filters")`.
7. **Tests**:
   - Rename `crates/api/tests/plan_repository_test.rs` → `saved_view_repository_test.rs` and `http_plan_test.rs` → `http_saved_view_test.rs`, and update their URLs and type names.
   - `crates/api/tests/full_user_journey_test.rs` uses `/plans`; update it.
   - `crates/api/tests/common/mod.rs`: change the `TRUNCATE ... plans ...` list to `saved_views`. **This is required:** with the old name the TRUNCATE fails once the table is renamed.
8. **Docs**: in `AGENTS.md`, `README.md` and `docs/test-database.md` (the TRUNCATE snippet), replace Plan/plans with Saved View/saved_views, including the endpoints table and the crate layout comment listing models.

## Done when

- `grep -rni "plan" crates AGENTS.md README.md docs/test-database.md` finds no remaining references to the old concept. `migrations/009_create_plans.sql` is expected to keep its name.
- Swagger shows a "Saved Views" tag with `/saved-views` endpoints.
- Clippy and the full suite pass.
