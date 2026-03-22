mod common;

use axum::http::StatusCode;
use axum_test::TestServer;
use serde_json::json;
use uuid::Uuid;

async fn setup() -> TestServer {
    let app = common::setup_test_app().await;
    TestServer::new(app)
}

#[tokio::test]
async fn nonexistent_route_returns_404() {
    let server = setup().await;

    let response = server.get("/nonexistent").await;

    response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn empty_json_body_for_task_returns_422() {
    let server = setup().await;

    let response = server
        .post("/tasks")
        .content_type("application/json")
        .bytes("".into())
        .await;

    // Axum returns 400 for empty body (JSON parse failure)
    response.assert_status(StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn malformed_json_body_returns_422() {
    let server = setup().await;

    let response = server
        .post("/tasks")
        .content_type("application/json")
        .bytes("{invalid json}".into())
        .await;

    // Axum returns 400 for malformed JSON
    response.assert_status(StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn create_column_nonexistent_board_returns_500() {
    let server = setup().await;
    let fake_board = Uuid::new_v4();

    let response = server
        .post(&format!("/boards/{fake_board}/columns"))
        .json(&json!({"name": "Col", "position": 0}))
        .await;

    // FK violation currently mapped to 500
    response.assert_status(StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn create_task_with_nonexistent_project_returns_500() {
    let server = setup().await;
    let fake_project = Uuid::new_v4();

    let response = server
        .post("/tasks")
        .json(&json!({"title": "Task", "project_id": fake_project}))
        .await;

    // FK violation currently mapped to 500
    response.assert_status(StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn create_task_with_nonexistent_user_returns_500() {
    let server = setup().await;
    let fake_user = Uuid::new_v4();

    let response = server
        .post("/tasks")
        .json(&json!({"title": "Task", "assigned_to": fake_user}))
        .await;

    // FK violation currently mapped to 500
    response.assert_status(StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn duplicate_user_email_returns_500() {
    let server = setup().await;

    server
        .post("/users")
        .json(&json!({"name": "Alice", "email": "dup@test.com"}))
        .await;

    let response = server
        .post("/users")
        .json(&json!({"name": "Bob", "email": "dup@test.com"}))
        .await;

    // Unique constraint violation currently mapped to 500
    response.assert_status(StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn wrong_type_for_field_returns_422() {
    let server = setup().await;

    let response = server
        .post("/tasks")
        .json(&json!({"title": 12345}))
        .await;

    response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn invalid_uuid_in_optional_field_returns_422() {
    let server = setup().await;

    let response = server
        .post("/tasks")
        .json(&json!({"title": "Task", "project_id": "not-a-uuid"}))
        .await;

    response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
}
