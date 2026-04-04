mod common;

use tulsi_rust_backend::models::board::CreateBoard;
use tulsi_rust_backend::models::column::{CreateColumn, UpdateColumn};
use tulsi_rust_backend::models::task::CreateTask;
use tulsi_rust_backend::repositories::board_repository::BoardRepository;
use tulsi_rust_backend::repositories::column_repository::ColumnRepository;
use tulsi_rust_backend::repositories::task_repository::TaskRepository;
use uuid::Uuid;

async fn setup() -> (ColumnRepository, BoardRepository, sqlx::PgPool) {
    let pool = common::setup_test_db().await;
    let column_repo = ColumnRepository::new(pool.clone(), None);
    let board_repo = BoardRepository::new(pool.clone(), None);
    (column_repo, board_repo, pool)
}

async fn create_test_board(board_repo: &BoardRepository) -> Uuid {
    board_repo
        .create(CreateBoard {
            name: "Test Board".to_string(),
            description: None,
        })
        .await
        .unwrap()
        .id
}

#[tokio::test]
async fn test_create_column() {
    let (repo, board_repo, _pool) = setup().await;
    let board_id = create_test_board(&board_repo).await;

    let column = repo
        .create(
            board_id,
            CreateColumn {
                name: "To Do".to_string(),
                position: Some(0),
            },
        )
        .await
        .unwrap();

    assert_eq!(column.name, "To Do");
    assert_eq!(column.position, 0);
    assert_eq!(column.board_id, board_id);
}

#[tokio::test]
async fn test_create_column_default_position() {
    let (repo, board_repo, _pool) = setup().await;
    let board_id = create_test_board(&board_repo).await;

    let column = repo
        .create(
            board_id,
            CreateColumn {
                name: "Backlog".to_string(),
                position: None,
            },
        )
        .await
        .unwrap();

    assert_eq!(column.position, 0);
}

#[tokio::test]
async fn test_find_by_board_id() {
    let (repo, board_repo, _pool) = setup().await;
    let board_id = create_test_board(&board_repo).await;

    let columns = repo.find_by_board_id(board_id).await.unwrap();
    assert!(columns.is_empty());

    for (name, pos) in [("To Do", 0), ("In Progress", 1), ("Done", 2)] {
        repo.create(
            board_id,
            CreateColumn {
                name: name.to_string(),
                position: Some(pos),
            },
        )
        .await
        .unwrap();
    }

    let columns = repo.find_by_board_id(board_id).await.unwrap();
    assert_eq!(columns.len(), 3);
    // Ordered by position ASC
    assert_eq!(columns[0].name, "To Do");
    assert_eq!(columns[1].name, "In Progress");
    assert_eq!(columns[2].name, "Done");
}

#[tokio::test]
async fn test_find_by_id() {
    let (repo, board_repo, _pool) = setup().await;
    let board_id = create_test_board(&board_repo).await;

    let column = repo
        .create(
            board_id,
            CreateColumn {
                name: "Find me".to_string(),
                position: Some(0),
            },
        )
        .await
        .unwrap();

    let found = repo.find_by_id(column.id).await.unwrap();
    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "Find me");
}

#[tokio::test]
async fn test_find_by_id_not_found() {
    let (repo, _board_repo, _pool) = setup().await;

    let found = repo.find_by_id(Uuid::new_v4()).await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn test_update_column() {
    let (repo, board_repo, _pool) = setup().await;
    let board_id = create_test_board(&board_repo).await;

    let column = repo
        .create(
            board_id,
            CreateColumn {
                name: "Original".to_string(),
                position: Some(0),
            },
        )
        .await
        .unwrap();

    let updated = repo
        .update(
            column.id,
            UpdateColumn {
                name: Some("Renamed".to_string()),
                position: None,
            },
        )
        .await
        .unwrap()
        .unwrap();

    assert_eq!(updated.name, "Renamed");
    assert_eq!(updated.position, 0); // unchanged
    assert!(updated.updated_at > column.updated_at);
}

#[tokio::test]
async fn test_update_column_position() {
    let (repo, board_repo, _pool) = setup().await;
    let board_id = create_test_board(&board_repo).await;

    let column = repo
        .create(
            board_id,
            CreateColumn {
                name: "Column".to_string(),
                position: Some(0),
            },
        )
        .await
        .unwrap();

    let updated = repo
        .update(
            column.id,
            UpdateColumn {
                name: None,
                position: Some(5),
            },
        )
        .await
        .unwrap()
        .unwrap();

    assert_eq!(updated.name, "Column"); // unchanged
    assert_eq!(updated.position, 5);
}

#[tokio::test]
async fn test_update_column_not_found() {
    let (repo, _board_repo, _pool) = setup().await;

    let result = repo
        .update(
            Uuid::new_v4(),
            UpdateColumn {
                name: Some("Nope".to_string()),
                position: None,
            },
        )
        .await
        .unwrap();

    assert!(result.is_none());
}

#[tokio::test]
async fn test_delete_column() {
    let (repo, board_repo, _pool) = setup().await;
    let board_id = create_test_board(&board_repo).await;

    let column = repo
        .create(
            board_id,
            CreateColumn {
                name: "Delete me".to_string(),
                position: Some(0),
            },
        )
        .await
        .unwrap();

    let deleted = repo.delete(column.id).await.unwrap();
    assert!(deleted);

    let found = repo.find_by_id(column.id).await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn test_delete_column_not_found() {
    let (repo, _board_repo, _pool) = setup().await;

    let deleted = repo.delete(Uuid::new_v4()).await.unwrap();
    assert!(!deleted);
}

#[tokio::test]
async fn test_find_tasks_in_column() {
    let (repo, board_repo, pool) = setup().await;
    let task_repo = TaskRepository::new(pool, None);
    let board_id = create_test_board(&board_repo).await;

    let column = repo
        .create(
            board_id,
            CreateColumn {
                name: "To Do".to_string(),
                position: Some(0),
            },
        )
        .await
        .unwrap();

    for title in ["Task 1", "Task 2"] {
        task_repo
            .create(CreateTask {
                title: title.to_string(),
                description: None,
                project_id: None,
                assigned_to: None,
                column_id: Some(column.id),
            })
            .await
            .unwrap();
    }

    let tasks = repo.find_tasks(column.id).await.unwrap();
    assert_eq!(tasks.len(), 2);
}

#[tokio::test]
async fn test_move_task_to_column() {
    let (repo, board_repo, pool) = setup().await;
    let task_repo = TaskRepository::new(pool, None);
    let board_id = create_test_board(&board_repo).await;

    let col_todo = repo
        .create(
            board_id,
            CreateColumn {
                name: "To Do".to_string(),
                position: Some(0),
            },
        )
        .await
        .unwrap();

    let col_done = repo
        .create(
            board_id,
            CreateColumn {
                name: "Done".to_string(),
                position: Some(1),
            },
        )
        .await
        .unwrap();

    let task = task_repo
        .create(CreateTask {
            title: "Move me".to_string(),
            description: None,
            project_id: None,
            assigned_to: None,
            column_id: Some(col_todo.id),
        })
        .await
        .unwrap();

    assert_eq!(task.column_id, Some(col_todo.id));

    let moved = repo.move_task(task.id, col_done.id).await.unwrap().unwrap();
    assert_eq!(moved.column_id, Some(col_done.id));
    assert!(moved.updated_at > task.updated_at);

    // Verify the task is no longer in the old column
    let todo_tasks = repo.find_tasks(col_todo.id).await.unwrap();
    assert!(todo_tasks.is_empty());

    // Verify the task is in the new column
    let done_tasks = repo.find_tasks(col_done.id).await.unwrap();
    assert_eq!(done_tasks.len(), 1);
    assert_eq!(done_tasks[0].title, "Move me");
}

#[tokio::test]
async fn test_move_task_not_found() {
    let (repo, board_repo, _pool) = setup().await;
    let board_id = create_test_board(&board_repo).await;

    let column = repo
        .create(
            board_id,
            CreateColumn {
                name: "To Do".to_string(),
                position: Some(0),
            },
        )
        .await
        .unwrap();

    let result = repo.move_task(Uuid::new_v4(), column.id).await.unwrap();
    assert!(result.is_none());
}
