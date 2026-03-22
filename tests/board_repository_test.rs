mod common;

use tulsi_rust_backend::models::board::{CreateBoard, UpdateBoard};
use tulsi_rust_backend::repositories::board_repository::BoardRepository;
use uuid::Uuid;

async fn setup() -> BoardRepository {
    let pool = common::setup_test_db().await;
    BoardRepository::new(pool)
}

#[tokio::test]
async fn test_create_board() {
    let repo = setup().await;

    let board = repo
        .create(CreateBoard {
            name: "My Board".to_string(),
            description: Some("A board".to_string()),
        })
        .await
        .unwrap();

    assert_eq!(board.name, "My Board");
    assert_eq!(board.description, "A board");
}

#[tokio::test]
async fn test_create_board_default_description() {
    let repo = setup().await;

    let board = repo
        .create(CreateBoard {
            name: "No desc".to_string(),
            description: None,
        })
        .await
        .unwrap();

    assert_eq!(board.description, "");
}

#[tokio::test]
async fn test_find_all() {
    let repo = setup().await;

    let boards = repo.find_all().await.unwrap();
    assert!(boards.is_empty());

    for name in ["Board A", "Board B"] {
        repo.create(CreateBoard {
            name: name.to_string(),
            description: None,
        })
        .await
        .unwrap();
    }

    let boards = repo.find_all().await.unwrap();
    assert_eq!(boards.len(), 2);
    assert_eq!(boards[0].name, "Board B");
    assert_eq!(boards[1].name, "Board A");
}

#[tokio::test]
async fn test_find_by_id() {
    let repo = setup().await;

    let board = repo
        .create(CreateBoard {
            name: "Find me".to_string(),
            description: None,
        })
        .await
        .unwrap();

    let found = repo.find_by_id(board.id).await.unwrap();
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
async fn test_update_board() {
    let repo = setup().await;

    let board = repo
        .create(CreateBoard {
            name: "Original".to_string(),
            description: Some("Original desc".to_string()),
        })
        .await
        .unwrap();

    let updated = repo
        .update(
            board.id,
            UpdateBoard {
                name: Some("Updated".to_string()),
                description: None,
            },
        )
        .await
        .unwrap()
        .unwrap();

    assert_eq!(updated.name, "Updated");
    assert_eq!(updated.description, "Original desc");
    assert!(updated.updated_at > board.updated_at);
}

#[tokio::test]
async fn test_update_board_not_found() {
    let repo = setup().await;

    let result = repo
        .update(
            Uuid::new_v4(),
            UpdateBoard {
                name: Some("Nope".to_string()),
                description: None,
            },
        )
        .await
        .unwrap();

    assert!(result.is_none());
}

#[tokio::test]
async fn test_delete_board() {
    let repo = setup().await;

    let board = repo
        .create(CreateBoard {
            name: "Delete me".to_string(),
            description: None,
        })
        .await
        .unwrap();

    let deleted = repo.delete(board.id).await.unwrap();
    assert!(deleted);

    let found = repo.find_by_id(board.id).await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn test_delete_board_not_found() {
    let repo = setup().await;

    let deleted = repo.delete(Uuid::new_v4()).await.unwrap();
    assert!(!deleted);
}

#[tokio::test]
async fn test_find_projects() {
    let pool = common::setup_test_db().await;
    let board_repo = BoardRepository::new(pool.clone());

    let board = board_repo
        .create(CreateBoard {
            name: "Board with projects".to_string(),
            description: None,
        })
        .await
        .unwrap();

    for name in ["Project 1", "Project 2"] {
        sqlx::query("INSERT INTO projects (name, description, board_id) VALUES ($1, '', $2)")
            .bind(name)
            .bind(board.id)
            .execute(&pool)
            .await
            .unwrap();
    }

    let projects = board_repo.find_projects(board.id).await.unwrap();
    assert_eq!(projects.len(), 2);
}
