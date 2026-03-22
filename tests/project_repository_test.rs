mod common;

use tulsi_rust_backend::models::project::{CreateProject, UpdateProject};
use tulsi_rust_backend::models::task::CreateTask;
use tulsi_rust_backend::repositories::project_repository::ProjectRepository;
use tulsi_rust_backend::repositories::task_repository::TaskRepository;
use uuid::Uuid;

async fn setup() -> ProjectRepository {
    let pool = common::setup_test_db().await;
    ProjectRepository::new(pool)
}

#[tokio::test]
async fn test_create_project() {
    let repo = setup().await;

    let project = repo
        .create(CreateProject {
            name: "My Project".to_string(),
            description: Some("A project".to_string()),
        })
        .await
        .unwrap();

    assert_eq!(project.name, "My Project");
    assert_eq!(project.description, "A project");
    assert!(project.board_id.is_none());
}

#[tokio::test]
async fn test_create_project_default_description() {
    let repo = setup().await;

    let project = repo
        .create(CreateProject {
            name: "No desc".to_string(),
            description: None,
        })
        .await
        .unwrap();

    assert_eq!(project.description, "");
}

#[tokio::test]
async fn test_find_all() {
    let repo = setup().await;

    let projects = repo.find_all().await.unwrap();
    assert!(projects.is_empty());

    for name in ["Project A", "Project B"] {
        repo.create(CreateProject {
            name: name.to_string(),
            description: None,
        })
        .await
        .unwrap();
    }

    let projects = repo.find_all().await.unwrap();
    assert_eq!(projects.len(), 2);
    assert_eq!(projects[0].name, "Project B");
    assert_eq!(projects[1].name, "Project A");
}

#[tokio::test]
async fn test_find_by_id() {
    let repo = setup().await;

    let project = repo
        .create(CreateProject {
            name: "Find me".to_string(),
            description: None,
        })
        .await
        .unwrap();

    let found = repo.find_by_id(project.id).await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "Find me");
}

#[tokio::test]
async fn test_find_by_id_not_found() {
    let repo = setup().await;

    let found = repo.find_by_id(Uuid::new_v4()).await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn test_update_project() {
    let repo = setup().await;

    let project = repo
        .create(CreateProject {
            name: "Original".to_string(),
            description: Some("Original desc".to_string()),
        })
        .await
        .unwrap();

    let updated = repo
        .update(
            project.id,
            UpdateProject {
                name: Some("Updated".to_string()),
                description: None,
            },
        )
        .await
        .unwrap()
        .unwrap();

    assert_eq!(updated.name, "Updated");
    assert_eq!(updated.description, "Original desc");
    assert!(updated.updated_at > project.updated_at);
}

#[tokio::test]
async fn test_update_project_not_found() {
    let repo = setup().await;

    let result = repo
        .update(
            Uuid::new_v4(),
            UpdateProject {
                name: Some("Nope".to_string()),
                description: None,
            },
        )
        .await
        .unwrap();

    assert!(result.is_none());
}

#[tokio::test]
async fn test_delete_project() {
    let repo = setup().await;

    let project = repo
        .create(CreateProject {
            name: "Delete me".to_string(),
            description: None,
        })
        .await
        .unwrap();

    let deleted = repo.delete(project.id).await.unwrap();
    assert!(deleted);

    let found = repo.find_by_id(project.id).await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn test_delete_project_not_found() {
    let repo = setup().await;

    let deleted = repo.delete(Uuid::new_v4()).await.unwrap();
    assert!(!deleted);
}

#[tokio::test]
async fn test_find_tasks() {
    let pool = common::setup_test_db().await;
    let project_repo = ProjectRepository::new(pool.clone());
    let task_repo = TaskRepository::new(pool);

    let project = project_repo
        .create(CreateProject {
            name: "Project with tasks".to_string(),
            description: None,
        })
        .await
        .unwrap();

    for title in ["Task 1", "Task 2"] {
        task_repo
            .create(CreateTask {
                title: title.to_string(),
                description: None,
                project_id: Some(project.id),
                assigned_to: None,
                column_id: None,
            })
            .await
            .unwrap();
    }

    let tasks = project_repo.find_tasks(project.id).await.unwrap();
    assert_eq!(tasks.len(), 2);
}
