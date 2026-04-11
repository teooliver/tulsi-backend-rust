mod common;

use tulsi_rust_backend::models::task::{CreateTask, UpdateTask};
use tulsi_rust_backend::repositories::task_repository::TaskRepository;
use uuid::Uuid;

async fn setup() -> TaskRepository {
    let pool = common::setup_test_db().await;
    TaskRepository::new(pool, None)
}

#[tokio::test]
async fn test_create_task() {
    let repo = setup().await;

    let input = CreateTask {
        title: "Test task".to_string(),
        description: Some("A description".to_string()),
        project_id: None,
        author: None,
        assigned_to: None,
        column_id: None,
    };

    let task = repo.create(input).await.unwrap();

    assert_eq!(task.title, "Test task");
    assert_eq!(task.description, "A description");
    assert!(task.project_id.is_none());
    assert!(task.assigned_to.is_none());
    assert!(task.column_id.is_none());
}

#[tokio::test]
async fn test_create_task_with_default_description() {
    let repo = setup().await;

    let input = CreateTask {
        title: "No desc task".to_string(),
        description: None,
        project_id: None,
        author: None,
        assigned_to: None,
        column_id: None,
    };

    let task = repo.create(input).await.unwrap();

    assert_eq!(task.title, "No desc task");
    assert_eq!(task.description, "");
}

#[tokio::test]
async fn test_find_all() {
    let repo = setup().await;

    // Start empty
    let tasks = repo.find_all().await.unwrap();
    assert!(tasks.is_empty());

    // Create two tasks
    for title in ["First", "Second"] {
        repo.create(CreateTask {
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

    let tasks = repo.find_all().await.unwrap();
    assert_eq!(tasks.len(), 2);
    // Ordered by created_at DESC, so "Second" should be first
    assert_eq!(tasks[0].title, "Second");
    assert_eq!(tasks[1].title, "First");
}

#[tokio::test]
async fn test_find_by_id() {
    let repo = setup().await;

    let task = repo
        .create(CreateTask {
            title: "Find me".to_string(),
            description: None,
            project_id: None,
            author: None,
            assigned_to: None,
            column_id: None,
        })
        .await
        .unwrap();

    let found = repo.find_by_id(task.id).await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().title, "Find me");
}

#[tokio::test]
async fn test_find_by_id_not_found() {
    let repo = setup().await;

    let found = repo.find_by_id(Uuid::new_v4()).await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn test_update_task() {
    let repo = setup().await;

    let task = repo
        .create(CreateTask {
            title: "Original".to_string(),
            description: Some("Original desc".to_string()),
            project_id: None,
            author: None,
            assigned_to: None,
            column_id: None,
        })
        .await
        .unwrap();

    let updated = repo
        .update(
            task.id,
            UpdateTask {
                title: Some("Updated".to_string()),
                description: None,
                project_id: None,
                author: None,
                assigned_to: None,
                column_id: None,
            },
        )
        .await
        .unwrap()
        .unwrap();

    assert_eq!(updated.title, "Updated");
    assert_eq!(updated.description, "Original desc"); // unchanged
    assert!(updated.updated_at > task.updated_at);
}

#[tokio::test]
async fn test_update_task_not_found() {
    let repo = setup().await;

    let result = repo
        .update(
            Uuid::new_v4(),
            UpdateTask {
                title: Some("Nope".to_string()),
                description: None,
                project_id: None,
                author: None,
                assigned_to: None,
                column_id: None,
            },
        )
        .await
        .unwrap();

    assert!(result.is_none());
}

#[tokio::test]
async fn test_delete_task() {
    let repo = setup().await;

    let task = repo
        .create(CreateTask {
            title: "Delete me".to_string(),
            description: None,
            project_id: None,
            author: None,
            assigned_to: None,
            column_id: None,
        })
        .await
        .unwrap();

    let deleted = repo.delete(task.id).await.unwrap();
    assert!(deleted);

    // Verify it's gone
    let found = repo.find_by_id(task.id).await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn test_delete_task_not_found() {
    let repo = setup().await;

    let deleted = repo.delete(Uuid::new_v4()).await.unwrap();
    assert!(!deleted);
}

#[tokio::test]
async fn test_update_partial_fields() {
    let repo = setup().await;

    let task = repo
        .create(CreateTask {
            title: "Original title".to_string(),
            description: Some("Original desc".to_string()),
            project_id: None,
            author: None,
            assigned_to: None,
            column_id: None,
        })
        .await
        .unwrap();

    // Update only description
    let updated = repo
        .update(
            task.id,
            UpdateTask {
                title: None,
                description: Some("New desc".to_string()),
                project_id: None,
                author: None,
                assigned_to: None,
                column_id: None,
            },
        )
        .await
        .unwrap()
        .unwrap();

    assert_eq!(updated.title, "Original title"); // unchanged
    assert_eq!(updated.description, "New desc");
}
