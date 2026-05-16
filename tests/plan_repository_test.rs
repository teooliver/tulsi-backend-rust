mod common;

use tulsi_rust_backend::models::plan::{CreatePlan, PlanFilters, UpdatePlan};
use tulsi_rust_backend::models::task::CreateTask;
use tulsi_rust_backend::models::user::CreateUser;
use tulsi_rust_backend::repositories::plan_repository::PlanRepository;
use tulsi_rust_backend::repositories::task_repository::TaskRepository;
use tulsi_rust_backend::repositories::user_repository::UserRepository;
use uuid::Uuid;

struct Repos {
    plans: PlanRepository,
    tasks: TaskRepository,
    users: UserRepository,
}

async fn setup() -> Repos {
    let pool = common::setup_test_db().await;
    Repos {
        plans: PlanRepository::new(pool.clone(), None),
        tasks: TaskRepository::new(pool.clone(), None),
        users: UserRepository::new(pool.clone(), None),
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

fn empty_plan(name: &str) -> CreatePlan {
    CreatePlan {
        name: name.to_string(),
        description: None,
        filters: PlanFilters::default(),
    }
}

// ── CRUD ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_create_plan() {
    let r = setup().await;
    let owner = make_user(&r.users).await;

    let plan = r.plans.create(owner, empty_plan("Sprint 1")).await.unwrap();

    assert_eq!(plan.name, "Sprint 1");
    assert_eq!(plan.description, "");
    assert_eq!(plan.owner_id, owner);
}

#[tokio::test]
async fn test_create_plan_with_description() {
    let r = setup().await;
    let owner = make_user(&r.users).await;

    let plan = r
        .plans
        .create(
            owner,
            CreatePlan {
                name: "Q4 Backlog".to_string(),
                description: Some("All Q4 work".to_string()),
                filters: PlanFilters::default(),
            },
        )
        .await
        .unwrap();

    assert_eq!(plan.description, "All Q4 work");
}

#[tokio::test]
async fn test_find_all_by_owner_scoped_to_owner() {
    let r = setup().await;
    let owner1 = make_user(&r.users).await;
    let owner2 = make_user(&r.users).await;

    r.plans.create(owner1, empty_plan("Plan A")).await.unwrap();
    r.plans.create(owner1, empty_plan("Plan B")).await.unwrap();
    r.plans.create(owner2, empty_plan("Plan C")).await.unwrap();

    let owner1_plans = r.plans.find_all_by_owner(owner1).await.unwrap();
    let owner2_plans = r.plans.find_all_by_owner(owner2).await.unwrap();

    assert_eq!(owner1_plans.len(), 2);
    assert_eq!(owner2_plans.len(), 1);
    assert_eq!(owner2_plans[0].name, "Plan C");
}

#[tokio::test]
async fn test_find_by_id_and_owner_returns_plan() {
    let r = setup().await;
    let owner = make_user(&r.users).await;
    let plan = r.plans.create(owner, empty_plan("My Plan")).await.unwrap();

    let found = r
        .plans
        .find_by_id_and_owner(plan.id, owner)
        .await
        .unwrap();

    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "My Plan");
}

#[tokio::test]
async fn test_find_by_id_and_owner_wrong_owner_returns_none() {
    let r = setup().await;
    let owner = make_user(&r.users).await;
    let other = make_user(&r.users).await;
    let plan = r.plans.create(owner, empty_plan("My Plan")).await.unwrap();

    let found = r
        .plans
        .find_by_id_and_owner(plan.id, other)
        .await
        .unwrap();

    assert!(found.is_none());
}

#[tokio::test]
async fn test_update_plan_name() {
    let r = setup().await;
    let owner = make_user(&r.users).await;
    let plan = r.plans.create(owner, empty_plan("Old Name")).await.unwrap();

    let updated = r
        .plans
        .update(
            plan.id,
            owner,
            UpdatePlan {
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
async fn test_update_plan_filters() {
    let r = setup().await;
    let owner = make_user(&r.users).await;
    let plan = r.plans.create(owner, empty_plan("Plan")).await.unwrap();
    let user_id = make_user(&r.users).await;

    let updated = r
        .plans
        .update(
            plan.id,
            owner,
            UpdatePlan {
                name: None,
                description: None,
                filters: Some(PlanFilters {
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
async fn test_update_plan_wrong_owner_returns_none() {
    let r = setup().await;
    let owner = make_user(&r.users).await;
    let other = make_user(&r.users).await;
    let plan = r.plans.create(owner, empty_plan("Plan")).await.unwrap();

    let result = r
        .plans
        .update(
            plan.id,
            other,
            UpdatePlan {
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
async fn test_delete_plan() {
    let r = setup().await;
    let owner = make_user(&r.users).await;
    let plan = r.plans.create(owner, empty_plan("To Delete")).await.unwrap();

    let deleted = r.plans.delete(plan.id, owner).await.unwrap();
    assert!(deleted);

    let found = r.plans.find_by_id_and_owner(plan.id, owner).await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn test_delete_plan_wrong_owner_returns_false() {
    let r = setup().await;
    let owner = make_user(&r.users).await;
    let other = make_user(&r.users).await;
    let plan = r.plans.create(owner, empty_plan("Plan")).await.unwrap();

    let deleted = r.plans.delete(plan.id, other).await.unwrap();
    assert!(!deleted);
}

// ── Filter execution ──────────────────────────────────────────────────────────

#[tokio::test]
async fn test_find_tasks_for_plan_not_found_returns_none() {
    let r = setup().await;
    let owner = make_user(&r.users).await;

    let result = r
        .plans
        .find_tasks_for_plan(Uuid::new_v4(), owner)
        .await
        .unwrap();

    assert!(result.is_none());
}

#[tokio::test]
async fn test_find_tasks_for_plan_empty_filters_returns_all_tasks() {
    let r = setup().await;
    let owner = make_user(&r.users).await;

    for title in ["Task 1", "Task 2", "Task 3"] {
        r.tasks
            .create(CreateTask {
                title: title.to_string(),
                description: None,
                project_id: None,
                author: None,
                assigned_to: None,
                column_id: None,
            })
            .await
            .unwrap();
    }

    let plan = r
        .plans
        .create(
            owner,
            CreatePlan {
                name: "All Tasks".to_string(),
                description: None,
                filters: PlanFilters::default(),
            },
        )
        .await
        .unwrap();

    let tasks = r
        .plans
        .find_tasks_for_plan(plan.id, owner)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(tasks.len(), 3);
}

#[tokio::test]
async fn test_find_tasks_for_plan_assigned_to_filter() {
    let r = setup().await;
    let owner = make_user(&r.users).await;
    let assignee = make_user(&r.users).await;

    r.tasks
        .create(CreateTask {
            title: "Assigned".to_string(),
            description: None,
            project_id: None,
            author: None,
            assigned_to: Some(assignee),
            column_id: None,
        })
        .await
        .unwrap();
    r.tasks
        .create(CreateTask {
            title: "Unassigned".to_string(),
            description: None,
            project_id: None,
            author: None,
            assigned_to: None,
            column_id: None,
        })
        .await
        .unwrap();

    let plan = r
        .plans
        .create(
            owner,
            CreatePlan {
                name: "Assignee's tasks".to_string(),
                description: None,
                filters: PlanFilters {
                    assigned_to: Some(vec![assignee]),
                    ..Default::default()
                },
            },
        )
        .await
        .unwrap();

    let tasks = r
        .plans
        .find_tasks_for_plan(plan.id, owner)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].title, "Assigned");
}

#[tokio::test]
async fn test_find_tasks_for_plan_multiple_assignees() {
    let r = setup().await;
    let owner = make_user(&r.users).await;
    let alice = make_user(&r.users).await;
    let bob = make_user(&r.users).await;

    for (title, assignee) in [("Alice task", alice), ("Bob task", bob)] {
        r.tasks
            .create(CreateTask {
                title: title.to_string(),
                description: None,
                project_id: None,
                author: None,
                assigned_to: Some(assignee),
                column_id: None,
            })
            .await
            .unwrap();
    }
    r.tasks
        .create(CreateTask {
            title: "No one's task".to_string(),
            description: None,
            project_id: None,
            author: None,
            assigned_to: None,
            column_id: None,
        })
        .await
        .unwrap();

    let plan = r
        .plans
        .create(
            owner,
            CreatePlan {
                name: "Alice and Bob".to_string(),
                description: None,
                filters: PlanFilters {
                    assigned_to: Some(vec![alice, bob]),
                    ..Default::default()
                },
            },
        )
        .await
        .unwrap();

    let tasks = r
        .plans
        .find_tasks_for_plan(plan.id, owner)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(tasks.len(), 2);
}

#[tokio::test]
async fn test_find_tasks_for_plan_author_filter() {
    let r = setup().await;
    let owner = make_user(&r.users).await;
    let author = make_user(&r.users).await;

    r.tasks
        .create(CreateTask {
            title: "By author".to_string(),
            description: None,
            project_id: None,
            author: Some(author),
            assigned_to: None,
            column_id: None,
        })
        .await
        .unwrap();
    r.tasks
        .create(CreateTask {
            title: "No author".to_string(),
            description: None,
            project_id: None,
            author: None,
            assigned_to: None,
            column_id: None,
        })
        .await
        .unwrap();

    let plan = r
        .plans
        .create(
            owner,
            CreatePlan {
                name: "Author's tasks".to_string(),
                description: None,
                filters: PlanFilters {
                    author: Some(vec![author]),
                    ..Default::default()
                },
            },
        )
        .await
        .unwrap();

    let tasks = r
        .plans
        .find_tasks_for_plan(plan.id, owner)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].title, "By author");
}

#[tokio::test]
async fn test_find_tasks_for_plan_date_filter() {
    let r = setup().await;
    let owner = make_user(&r.users).await;

    // Insert task, then capture its created_at for the filter
    let task = r
        .tasks
        .create(CreateTask {
            title: "Old task".to_string(),
            description: None,
            project_id: None,
            author: None,
            assigned_to: None,
            column_id: None,
        })
        .await
        .unwrap();

    // created_before = task's own created_at excludes it; use a future time to include it
    let future = chrono::Utc::now() + chrono::Duration::hours(1);

    let plan = r
        .plans
        .create(
            owner,
            CreatePlan {
                name: "Recent tasks".to_string(),
                description: None,
                filters: PlanFilters {
                    created_after: Some(task.created_at - chrono::Duration::seconds(1)),
                    created_before: Some(future),
                    ..Default::default()
                },
            },
        )
        .await
        .unwrap();

    let tasks = r
        .plans
        .find_tasks_for_plan(plan.id, owner)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].id, task.id);
}
