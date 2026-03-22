mod common;

use axum::http::StatusCode;
use axum_test::TestServer;
use serde_json::json;
use uuid::Uuid;

async fn setup() -> TestServer {
    let app = common::setup_test_app().await;
    TestServer::new(app)
}

async fn create_board(server: &TestServer) -> String {
    let board: serde_json::Value = server
        .post("/boards")
        .json(&json!({"name": "Test Board"}))
        .await
        .json();
    board["id"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn create_column_returns_201() {
    let server = setup().await;
    let board_id = create_board(&server).await;

    let response = server
        .post(&format!("/boards/{board_id}/columns"))
        .json(&json!({"name": "To Do", "position": 0}))
        .await;

    response.assert_status(StatusCode::CREATED);
    let body: serde_json::Value = response.json();
    assert_eq!(body["name"], "To Do");
    assert_eq!(body["position"], 0);
    assert_eq!(body["board_id"], board_id);
}

#[tokio::test]
async fn create_column_with_default_position() {
    let server = setup().await;
    let board_id = create_board(&server).await;

    let response = server
        .post(&format!("/boards/{board_id}/columns"))
        .json(&json!({"name": "Column"}))
        .await;

    response.assert_status(StatusCode::CREATED);
    let body: serde_json::Value = response.json();
    assert_eq!(body["name"], "Column");
}

#[tokio::test]
async fn list_board_columns_returns_empty() {
    let server = setup().await;
    let board_id = create_board(&server).await;

    let response = server.get(&format!("/boards/{board_id}/columns")).await;

    response.assert_status_ok();
    let body: Vec<serde_json::Value> = response.json();
    assert!(body.is_empty());
}

#[tokio::test]
async fn list_board_columns_ordered_by_position() {
    let server = setup().await;
    let board_id = create_board(&server).await;

    server
        .post(&format!("/boards/{board_id}/columns"))
        .json(&json!({"name": "Done", "position": 2}))
        .await;
    server
        .post(&format!("/boards/{board_id}/columns"))
        .json(&json!({"name": "To Do", "position": 0}))
        .await;
    server
        .post(&format!("/boards/{board_id}/columns"))
        .json(&json!({"name": "In Progress", "position": 1}))
        .await;

    let response = server.get(&format!("/boards/{board_id}/columns")).await;

    response.assert_status_ok();
    let body: Vec<serde_json::Value> = response.json();
    assert_eq!(body.len(), 3);
    assert_eq!(body[0]["name"], "To Do");
    assert_eq!(body[1]["name"], "In Progress");
    assert_eq!(body[2]["name"], "Done");
}

#[tokio::test]
async fn get_column_by_id_returns_200() {
    let server = setup().await;
    let board_id = create_board(&server).await;

    let created: serde_json::Value = server
        .post(&format!("/boards/{board_id}/columns"))
        .json(&json!({"name": "To Do", "position": 0}))
        .await
        .json();
    let column_id = created["id"].as_str().unwrap();

    let response = server
        .get(&format!("/boards/{board_id}/columns/{column_id}"))
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["name"], "To Do");
}

#[tokio::test]
async fn get_column_not_found_returns_404() {
    let server = setup().await;
    let board_id = create_board(&server).await;
    let column_id = Uuid::new_v4();

    let response = server
        .get(&format!("/boards/{board_id}/columns/{column_id}"))
        .await;

    response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn update_column_returns_200() {
    let server = setup().await;
    let board_id = create_board(&server).await;

    let created: serde_json::Value = server
        .post(&format!("/boards/{board_id}/columns"))
        .json(&json!({"name": "To Do", "position": 0}))
        .await
        .json();
    let column_id = created["id"].as_str().unwrap();

    let response = server
        .put(&format!("/boards/{board_id}/columns/{column_id}"))
        .json(&json!({"name": "Backlog", "position": 1}))
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["name"], "Backlog");
    assert_eq!(body["position"], 1);
}

#[tokio::test]
async fn update_column_not_found_returns_404() {
    let server = setup().await;
    let board_id = create_board(&server).await;
    let column_id = Uuid::new_v4();

    let response = server
        .put(&format!("/boards/{board_id}/columns/{column_id}"))
        .json(&json!({"name": "Nope"}))
        .await;

    response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_column_returns_204() {
    let server = setup().await;
    let board_id = create_board(&server).await;

    let created: serde_json::Value = server
        .post(&format!("/boards/{board_id}/columns"))
        .json(&json!({"name": "Delete me", "position": 0}))
        .await
        .json();
    let column_id = created["id"].as_str().unwrap();

    let response = server
        .delete(&format!("/boards/{board_id}/columns/{column_id}"))
        .await;

    response.assert_status(StatusCode::NO_CONTENT);

    server
        .get(&format!("/boards/{board_id}/columns/{column_id}"))
        .await
        .assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_column_not_found_returns_404() {
    let server = setup().await;
    let board_id = create_board(&server).await;
    let column_id = Uuid::new_v4();

    let response = server
        .delete(&format!("/boards/{board_id}/columns/{column_id}"))
        .await;

    response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn create_column_missing_name_returns_422() {
    let server = setup().await;
    let board_id = create_board(&server).await;

    let response = server
        .post(&format!("/boards/{board_id}/columns"))
        .json(&json!({}))
        .await;

    response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn list_column_tasks_returns_empty() {
    let server = setup().await;
    let board_id = create_board(&server).await;

    let column: serde_json::Value = server
        .post(&format!("/boards/{board_id}/columns"))
        .json(&json!({"name": "To Do", "position": 0}))
        .await
        .json();
    let column_id = column["id"].as_str().unwrap();

    let response = server
        .get(&format!("/boards/{board_id}/columns/{column_id}/tasks"))
        .await;

    response.assert_status_ok();
    let body: Vec<serde_json::Value> = response.json();
    assert!(body.is_empty());
}

#[tokio::test]
async fn list_column_tasks_not_found_returns_404() {
    let server = setup().await;
    let board_id = create_board(&server).await;
    let column_id = Uuid::new_v4();

    let response = server
        .get(&format!("/boards/{board_id}/columns/{column_id}/tasks"))
        .await;

    response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn list_column_tasks_returns_tasks_in_column() {
    let server = setup().await;
    let board_id = create_board(&server).await;

    let column: serde_json::Value = server
        .post(&format!("/boards/{board_id}/columns"))
        .json(&json!({"name": "To Do", "position": 0}))
        .await
        .json();
    let column_id = column["id"].as_str().unwrap();

    server
        .post("/tasks")
        .json(&json!({"title": "In column", "column_id": column_id}))
        .await;
    server
        .post("/tasks")
        .json(&json!({"title": "Not in column"}))
        .await;

    let response = server
        .get(&format!("/boards/{board_id}/columns/{column_id}/tasks"))
        .await;

    response.assert_status_ok();
    let body: Vec<serde_json::Value> = response.json();
    assert_eq!(body.len(), 1);
    assert_eq!(body[0]["title"], "In column");
}

#[tokio::test]
async fn move_task_to_column() {
    let server = setup().await;
    let board_id = create_board(&server).await;

    let col_todo: serde_json::Value = server
        .post(&format!("/boards/{board_id}/columns"))
        .json(&json!({"name": "To Do", "position": 0}))
        .await
        .json();
    let col_todo_id = col_todo["id"].as_str().unwrap();

    let col_done: serde_json::Value = server
        .post(&format!("/boards/{board_id}/columns"))
        .json(&json!({"name": "Done", "position": 1}))
        .await
        .json();
    let col_done_id = col_done["id"].as_str().unwrap();

    let task: serde_json::Value = server
        .post("/tasks")
        .json(&json!({"title": "Move me", "column_id": col_todo_id}))
        .await
        .json();
    let task_id = task["id"].as_str().unwrap();

    // Move task to Done
    let response = server
        .put(&format!("/tasks/{task_id}/move"))
        .json(&json!({"column_id": col_done_id}))
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["column_id"], col_done_id);

    // Verify task is in Done column
    let done_tasks: Vec<serde_json::Value> = server
        .get(&format!("/boards/{board_id}/columns/{col_done_id}/tasks"))
        .await
        .json();
    assert_eq!(done_tasks.len(), 1);
    assert_eq!(done_tasks[0]["title"], "Move me");

    // Verify task is NOT in To Do column
    let todo_tasks: Vec<serde_json::Value> = server
        .get(&format!("/boards/{board_id}/columns/{col_todo_id}/tasks"))
        .await
        .json();
    assert!(todo_tasks.is_empty());
}

#[tokio::test]
async fn move_task_not_found_returns_404() {
    let server = setup().await;
    let board_id = create_board(&server).await;

    let column: serde_json::Value = server
        .post(&format!("/boards/{board_id}/columns"))
        .json(&json!({"name": "Col", "position": 0}))
        .await
        .json();
    let column_id = column["id"].as_str().unwrap();

    let task_id = Uuid::new_v4();

    let response = server
        .put(&format!("/tasks/{task_id}/move"))
        .json(&json!({"column_id": column_id}))
        .await;

    response.assert_status(StatusCode::NOT_FOUND);
}
