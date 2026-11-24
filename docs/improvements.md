# Improvements backlog

Known weaknesses we've chosen not to fix yet. Each entry says what's wrong, why it matters, and roughly what a fix looks like, so it can be picked up without re-investigating. Remove an entry once it's done.

## Users can edit and delete other Users

**Problem:** `PUT /users/{id}` and `DELETE /users/{id}` only check that the caller is logged in. Any User can rename another User, change their email, or delete their account.

**Why it matters:** Changing someone's email takes over their account, because login is by email. Deleting a User removes someone else's account and clears their task assignments.

**Fix:** In `update_user` and `delete_user` (`crates/api/src/handlers/user_handler.rs`), compare the path `id` with the caller's `AuthUser.user_id` and return `403 Forbidden` when they differ. Saved Views already scope access to their owner the same way. If admins should be able to manage other Users, that needs roles first. A cleaner option may be to replace these with `PUT /auth/me` and `DELETE /auth/me`, which can only act on the caller.

**Context:** Noted when `POST /users` was removed in favour of `/auth/register`, the only way to create a User now.
