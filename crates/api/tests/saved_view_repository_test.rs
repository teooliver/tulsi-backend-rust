mod common;

use models::saved_view::{CreateSavedView, SavedViewFilters, UpdateSavedView};
use models::task::CreateTask;
use models::user::CreateUser;
use repositories::saved_view_repository::SavedViewRepository;
use repositories::task_repository::TaskRepository;
use repositories::user_repository::UserRepository;
use uuid::Uuid;

struct Repos {
    saved_views: SavedViewRepository,
    tasks: TaskRepository,
    users: UserRepository,
    author: Uuid,
}

async fn setup() -> Repos {
    let pool = common::setup_test_db().await;
    Repos {
        saved_views: SavedViewRepository::new(pool.clone(), None),
        tasks: TaskRepository::new(pool.clone(), None),
        users: UserRepository::new(pool.clone(), None),
        author: common::make_user(&pool).await,
    }
}

async fn make_user(users: &UserRepository) -> Uuid {
    users
        .create(CreateUser {
            name: "Test User".to_string(),
            email: format!("user_{}@test.com", Uuid::new_v4()),
        })
        .await
        .unwrap()
        .id
}

fn empty_saved_view(name: &str) -> CreateSavedView {
    CreateSavedView {
        name: name.to_string(),
        description: None,
        filters: SavedViewFilters::default(),
    }
}

// ── CRUD ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_create_saved_view() {
    let r = setup().await;
    let owner = make_user(&r.users).await;

    let saved_view = r.saved_views.create(owner, empty_saved_view("Sprint 1")).await.unwrap();

    assert_eq!(saved_view.name, "Sprint 1");
    assert_eq!(saved_view.description, "");
    assert_eq!(saved_view.owner_id, owner);
}

#[tokio::test]
async fn test_create_saved_view_with_description() {
    let r = setup().await;
    let owner = make_user(&r.users).await;

    let saved_view = r
        .saved_views
        .create(
            owner,
            CreateSavedView {
                name: "Q4 Backlog".to_string(),
                description: Some("All Q4 work".to_string()),
                filters: SavedViewFilters::default(),
            },
        )
        .await
        .unwrap();

    assert_eq!(saved_view.description, "All Q4 work");
}

#[tokio::test]
async fn test_find_all_by_owner_scoped_to_owner() {
    let r = setup().await;
    let owner1 = make_user(&r.users).await;
    let owner2 = make_user(&r.users).await;

    r.saved_views.create(owner1, empty_saved_view("View A")).await.unwrap();
    r.saved_views.create(owner1, empty_saved_view("View B")).await.unwrap();
    r.saved_views.create(owner2, empty_saved_view("View C")).await.unwrap();

    let owner1_saved_views = r.saved_views.find_all_by_owner(owner1).await.unwrap();
    let owner2_saved_views = r.saved_views.find_all_by_owner(owner2).await.unwrap();

    assert_eq!(owner1_saved_views.len(), 2);
    assert_eq!(owner2_saved_views.len(), 1);
    assert_eq!(owner2_saved_views[0].name, "View C");
}

#[tokio::test]
async fn test_find_by_id_and_owner_returns_saved_view() {
    let r = setup().await;
    let owner = make_user(&r.users).await;
    let saved_view = r.saved_views.create(owner, empty_saved_view("My Saved View")).await.unwrap();

    let found = r
        .saved_views
        .find_by_id_and_owner(saved_view.id, owner)
        .await
        .unwrap();

    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "My Saved View");
}

#[tokio::test]
async fn test_find_by_id_and_owner_wrong_owner_returns_none() {
    let r = setup().await;
    let owner = make_user(&r.users).await;
    let other = make_user(&r.users).await;
    let saved_view = r.saved_views.create(owner, empty_saved_view("My Saved View")).await.unwrap();

    let found = r
        .saved_views
        .find_by_id_and_owner(saved_view.id, other)
        .await
        .unwrap();

    assert!(found.is_none());
}

#[tokio::test]
async fn test_update_saved_view_name() {
    let r = setup().await;
    let owner = make_user(&r.users).await;
    let saved_view = r.saved_views.create(owner, empty_saved_view("Old Name")).await.unwrap();

    let updated = r
        .saved_views
        .update(
            saved_view.id,
            owner,
            UpdateSavedView {
                name: Some("New Name".to_string()),
                description: None,
                filters: None,
            },
        )
        .await
        .unwrap()
        .unwrap();

    assert_eq!(updated.name, "New Name");
    assert_eq!(updated.description, ""); // unchanged
}

#[tokio::test]
async fn test_update_saved_view_filters() {
    let r = setup().await;
    let owner = make_user(&r.users).await;
    let saved_view = r.saved_views.create(owner, empty_saved_view("Saved View")).await.unwrap();
    let user_id = make_user(&r.users).await;

    let updated = r
        .saved_views
        .update(
            saved_view.id,
            owner,
            UpdateSavedView {
                name: None,
                description: None,
                filters: Some(SavedViewFilters {
                    assigned_to: Some(vec![user_id]),
                    ..Default::default()
                }),
            },
        )
        .await
        .unwrap()
        .unwrap();

    assert_eq!(updated.filters.0.assigned_to, Some(vec![user_id]));
}

#[tokio::test]
async fn test_update_saved_view_wrong_owner_returns_none() {
    let r = setup().await;
    let owner = make_user(&r.users).await;
    let other = make_user(&r.users).await;
    let saved_view = r.saved_views.create(owner, empty_saved_view("Saved View")).await.unwrap();

    let result = r
        .saved_views
        .update(
            saved_view.id,
            other,
            UpdateSavedView {
                name: Some("Hijacked".to_string()),
                description: None,
                filters: None,
            },
        )
        .await
        .unwrap();

    assert!(result.is_none());
}

#[tokio::test]
async fn test_delete_saved_view() {
    let r = setup().await;
    let owner = make_user(&r.users).await;
    let saved_view = r.saved_views.create(owner, empty_saved_view("To Delete")).await.unwrap();

    let deleted = r.saved_views.delete(saved_view.id, owner).await.unwrap();
    assert!(deleted);

    let found = r.saved_views.find_by_id_and_owner(saved_view.id, owner).await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn test_delete_saved_view_wrong_owner_returns_false() {
    let r = setup().await;
    let owner = make_user(&r.users).await;
    let other = make_user(&r.users).await;
    let saved_view = r.saved_views.create(owner, empty_saved_view("Saved View")).await.unwrap();

    let deleted = r.saved_views.delete(saved_view.id, other).await.unwrap();
    assert!(!deleted);
}

// ── Filter execution ──────────────────────────────────────────────────────────

#[tokio::test]
async fn test_find_tasks_for_saved_view_not_found_returns_none() {
    let r = setup().await;
    let owner = make_user(&r.users).await;

    let result = r
        .saved_views
        .find_tasks_for_saved_view(Uuid::new_v4(), owner)
        .await
        .unwrap();

    assert!(result.is_none());
}

#[tokio::test]
async fn test_find_tasks_for_saved_view_empty_filters_returns_all_tasks() {
    let r = setup().await;
    let owner = make_user(&r.users).await;

    for title in ["Task 1", "Task 2", "Task 3"] {
        r.tasks
            .create(r.author, CreateTask {
                title: title.to_string(),
                description: None,
                project_id: None,
                assigned_to: None,
                column_id: None,
            })
            .await
            .unwrap();
    }

    let saved_view = r
        .saved_views
        .create(
            owner,
            CreateSavedView {
                name: "All Tasks".to_string(),
                description: None,
                filters: SavedViewFilters::default(),
            },
        )
        .await
        .unwrap();

    let tasks = r
        .saved_views
        .find_tasks_for_saved_view(saved_view.id, owner)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(tasks.len(), 3);
}

#[tokio::test]
async fn test_find_tasks_for_saved_view_assigned_to_filter() {
    let r = setup().await;
    let owner = make_user(&r.users).await;
    let assignee = make_user(&r.users).await;

    r.tasks
        .create(r.author, CreateTask {
            title: "Assigned".to_string(),
            description: None,
            project_id: None,
            assigned_to: Some(assignee),
            column_id: None,
        })
        .await
        .unwrap();
    r.tasks
        .create(r.author, CreateTask {
            title: "Unassigned".to_string(),
            description: None,
            project_id: None,
            assigned_to: None,
            column_id: None,
        })
        .await
        .unwrap();

    let saved_view = r
        .saved_views
        .create(
            owner,
            CreateSavedView {
                name: "Assignee's tasks".to_string(),
                description: None,
                filters: SavedViewFilters {
                    assigned_to: Some(vec![assignee]),
                    ..Default::default()
                },
            },
        )
        .await
        .unwrap();

    let tasks = r
        .saved_views
        .find_tasks_for_saved_view(saved_view.id, owner)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].title, "Assigned");
}

#[tokio::test]
async fn test_find_tasks_for_saved_view_multiple_assignees() {
    let r = setup().await;
    let owner = make_user(&r.users).await;
    let alice = make_user(&r.users).await;
    let bob = make_user(&r.users).await;

    for (title, assignee) in [("Alice task", alice), ("Bob task", bob)] {
        r.tasks
            .create(r.author, CreateTask {
                title: title.to_string(),
                description: None,
                project_id: None,
                assigned_to: Some(assignee),
                column_id: None,
            })
            .await
            .unwrap();
    }
    r.tasks
        .create(r.author, CreateTask {
            title: "No one's task".to_string(),
            description: None,
            project_id: None,
            assigned_to: None,
            column_id: None,
        })
        .await
        .unwrap();

    let saved_view = r
        .saved_views
        .create(
            owner,
            CreateSavedView {
                name: "Alice and Bob".to_string(),
                description: None,
                filters: SavedViewFilters {
                    assigned_to: Some(vec![alice, bob]),
                    ..Default::default()
                },
            },
        )
        .await
        .unwrap();

    let tasks = r
        .saved_views
        .find_tasks_for_saved_view(saved_view.id, owner)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(tasks.len(), 2);
}

#[tokio::test]
async fn test_find_tasks_for_saved_view_author_filter() {
    let r = setup().await;
    let owner = make_user(&r.users).await;
    let author = make_user(&r.users).await;

    r.tasks
        .create(author, CreateTask {
            title: "By author".to_string(),
            description: None,
            project_id: None,
            assigned_to: None,
            column_id: None,
        })
        .await
        .unwrap();
    r.tasks
        .create(r.author, CreateTask {
            title: "By another user".to_string(),
            description: None,
            project_id: None,
            assigned_to: None,
            column_id: None,
        })
        .await
        .unwrap();

    let saved_view = r
        .saved_views
        .create(
            owner,
            CreateSavedView {
                name: "Author's tasks".to_string(),
                description: None,
                filters: SavedViewFilters {
                    author: Some(vec![author]),
                    ..Default::default()
                },
            },
        )
        .await
        .unwrap();

    let tasks = r
        .saved_views
        .find_tasks_for_saved_view(saved_view.id, owner)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].title, "By author");
}

#[tokio::test]
async fn test_find_tasks_for_saved_view_date_filter() {
    let r = setup().await;
    let owner = make_user(&r.users).await;

    // Insert task, then capture its created_at for the filter
    let task = r
        .tasks
        .create(r.author, CreateTask {
            title: "Old task".to_string(),
            description: None,
            project_id: None,
            assigned_to: None,
            column_id: None,
        })
        .await
        .unwrap();

    // created_before = task's own created_at excludes it; use a future time to include it
    let future = chrono::Utc::now() + chrono::Duration::hours(1);

    let saved_view = r
        .saved_views
        .create(
            owner,
            CreateSavedView {
                name: "Recent tasks".to_string(),
                description: None,
                filters: SavedViewFilters {
                    created_after: Some(task.created_at - chrono::Duration::seconds(1)),
                    created_before: Some(future),
                    ..Default::default()
                },
            },
        )
        .await
        .unwrap();

    let tasks = r
        .saved_views
        .find_tasks_for_saved_view(saved_view.id, owner)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].id, task.id);
}
