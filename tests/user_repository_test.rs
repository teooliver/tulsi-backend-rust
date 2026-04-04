mod common;

use tulsi_rust_backend::models::task::CreateTask;
use tulsi_rust_backend::models::user::{CreateUser, UpdateUser};
use tulsi_rust_backend::repositories::task_repository::TaskRepository;
use tulsi_rust_backend::repositories::user_repository::UserRepository;
use uuid::Uuid;

async fn setup() -> UserRepository {
    let pool = common::setup_test_db().await;
    UserRepository::new(pool, None)
}

#[tokio::test]
async fn test_create_user() {
    let repo = setup().await;

    let user = repo
        .create(CreateUser {
            name: "Alice".to_string(),
            email: "alice@example.com".to_string(),
        })
        .await
        .unwrap();

    assert_eq!(user.name, "Alice");
    assert_eq!(user.email, "alice@example.com");
}

#[tokio::test]
async fn test_create_user_duplicate_email_fails() {
    let repo = setup().await;

    repo.create(CreateUser {
        name: "Alice".to_string(),
        email: "dupe@example.com".to_string(),
    })
    .await
    .unwrap();

    let result = repo
        .create(CreateUser {
            name: "Bob".to_string(),
            email: "dupe@example.com".to_string(),
        })
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_find_all() {
    let repo = setup().await;

    let users = repo.find_all().await.unwrap();
    assert!(users.is_empty());

    repo.create(CreateUser {
        name: "Alice".to_string(),
        email: "alice_all@example.com".to_string(),
    })
    .await
    .unwrap();
    repo.create(CreateUser {
        name: "Bob".to_string(),
        email: "bob_all@example.com".to_string(),
    })
    .await
    .unwrap();

    let users = repo.find_all().await.unwrap();
    assert_eq!(users.len(), 2);
    assert_eq!(users[0].name, "Bob");
    assert_eq!(users[1].name, "Alice");
}

#[tokio::test]
async fn test_find_by_id() {
    let repo = setup().await;

    let user = repo
        .create(CreateUser {
            name: "Alice".to_string(),
            email: "alice_find@example.com".to_string(),
        })
        .await
        .unwrap();

    let found = repo.find_by_id(user.id).await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "Alice");
}

#[tokio::test]
async fn test_find_by_id_not_found() {
    let repo = setup().await;

    let found = repo.find_by_id(Uuid::new_v4()).await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn test_update_user() {
    let repo = setup().await;

    let user = repo
        .create(CreateUser {
            name: "Alice".to_string(),
            email: "alice_update@example.com".to_string(),
        })
        .await
        .unwrap();

    let updated = repo
        .update(
            user.id,
            UpdateUser {
                name: Some("Alicia".to_string()),
                email: None,
            },
        )
        .await
        .unwrap()
        .unwrap();

    assert_eq!(updated.name, "Alicia");
    assert_eq!(updated.email, "alice_update@example.com");
    assert!(updated.updated_at > user.updated_at);
}

#[tokio::test]
async fn test_update_user_not_found() {
    let repo = setup().await;

    let result = repo
        .update(
            Uuid::new_v4(),
            UpdateUser {
                name: Some("Nope".to_string()),
                email: None,
            },
        )
        .await
        .unwrap();

    assert!(result.is_none());
}

#[tokio::test]
async fn test_delete_user() {
    let repo = setup().await;

    let user = repo
        .create(CreateUser {
            name: "Alice".to_string(),
            email: "alice_delete@example.com".to_string(),
        })
        .await
        .unwrap();

    let deleted = repo.delete(user.id).await.unwrap();
    assert!(deleted);

    let found = repo.find_by_id(user.id).await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn test_delete_user_not_found() {
    let repo = setup().await;

    let deleted = repo.delete(Uuid::new_v4()).await.unwrap();
    assert!(!deleted);
}

#[tokio::test]
async fn test_find_tasks() {
    let pool = common::setup_test_db().await;
    let user_repo = UserRepository::new(pool.clone(), None);
    let task_repo = TaskRepository::new(pool, None);

    let user = user_repo
        .create(CreateUser {
            name: "Alice".to_string(),
            email: "alice_tasks@example.com".to_string(),
        })
        .await
        .unwrap();

    for title in ["Task 1", "Task 2"] {
        task_repo
            .create(CreateTask {
                title: title.to_string(),
                description: None,
                project_id: None,
                assigned_to: Some(user.id),
                column_id: None,
            })
            .await
            .unwrap();
    }

    let tasks = user_repo.find_tasks(user.id).await.unwrap();
    assert_eq!(tasks.len(), 2);
}
