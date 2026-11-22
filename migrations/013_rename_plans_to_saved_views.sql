-- "Plan" was renamed to "Saved View" (see GLOSSARY.md).
ALTER TABLE plans RENAME TO saved_views;
ALTER INDEX idx_plans_owner_id RENAME TO idx_saved_views_owner_id;
