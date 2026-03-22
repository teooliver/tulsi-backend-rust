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
async fn create_board_returns_201_with_body() {
    let server = setup().await;

    let response = server
        .post("/boards")
        .json(&json!({"name": "Sprint Board"}))
        .await;

    response.assert_status(StatusCode::CREATED);
    let body: serde_json::Value = response.json();
    assert_eq!(body["name"], "Sprint Board");
    assert_eq!(body["description"], "");
    assert!(body["id"].as_str().is_some());
}

#[tokio::test]
async fn create_board_with_description() {
    let server = setup().await;

    let response = server
        .post("/boards")
        .json(&json!({"name": "Board", "description": "My board"}))
        .await;

    response.assert_status(StatusCode::CREATED);
    let body: serde_json::Value = response.json();
    assert_eq!(body["description"], "My board");
}

#[tokio::test]
async fn list_boards_returns_200_with_empty_array() {
    let server = setup().await;

    let response = server.get("/boards").await;

    response.assert_status_ok();
    let body: Vec<serde_json::Value> = response.json();
    assert!(body.is_empty());
}

#[tokio::test]
async fn list_boards_returns_created_boards() {
    let server = setup().await;

    server
        .post("/boards")
        .json(&json!({"name": "Board 1"}))
        .await;
    server
        .post("/boards")
        .json(&json!({"name": "Board 2"}))
        .await;

    let response = server.get("/boards").await;

    response.assert_status_ok();
    let body: Vec<serde_json::Value> = response.json();
    assert_eq!(body.len(), 2);
}

#[tokio::test]
async fn get_board_by_id_returns_200() {
    let server = setup().await;

    let created: serde_json::Value = server
        .post("/boards")
        .json(&json!({"name": "Find me"}))
        .await
        .json();
    let id = created["id"].as_str().unwrap();

    let response = server.get(&format!("/boards/{id}")).await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["name"], "Find me");
}

#[tokio::test]
async fn get_board_not_found_returns_404() {
    let server = setup().await;
    let id = Uuid::new_v4();

    let response = server.get(&format!("/boards/{id}")).await;

    response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn get_board_invalid_uuid_returns_400() {
    let server = setup().await;

    let response = server.get("/boards/not-a-uuid").await;

    response.assert_status(StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn update_board_returns_200() {
    let server = setup().await;

    let created: serde_json::Value = server
        .post("/boards")
        .json(&json!({"name": "Original"}))
        .await
        .json();
    let id = created["id"].as_str().unwrap();

    let response = server
        .put(&format!("/boards/{id}"))
        .json(&json!({"name": "Updated"}))
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["name"], "Updated");
}

#[tokio::test]
async fn update_board_partial_fields() {
    let server = setup().await;

    let created: serde_json::Value = server
        .post("/boards")
        .json(&json!({"name": "Board", "description": "Keep me"}))
        .await
        .json();
    let id = created["id"].as_str().unwrap();

    let response = server
        .put(&format!("/boards/{id}"))
        .json(&json!({"name": "New Name"}))
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["name"], "New Name");
    assert_eq!(body["description"], "Keep me");
}

#[tokio::test]
async fn update_board_not_found_returns_404() {
    let server = setup().await;
    let id = Uuid::new_v4();

    let response = server
        .put(&format!("/boards/{id}"))
        .json(&json!({"name": "Nope"}))
        .await;

    response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_board_returns_204() {
    let server = setup().await;

    let created: serde_json::Value = server
        .post("/boards")
        .json(&json!({"name": "Delete me"}))
        .await
        .json();
    let id = created["id"].as_str().unwrap();

    let response = server.delete(&format!("/boards/{id}")).await;

    response.assert_status(StatusCode::NO_CONTENT);

    server
        .get(&format!("/boards/{id}"))
        .await
        .assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_board_not_found_returns_404() {
    let server = setup().await;
    let id = Uuid::new_v4();

    let response = server.delete(&format!("/boards/{id}")).await;

    response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn create_board_missing_name_returns_422() {
    let server = setup().await;

    let response = server.post("/boards").json(&json!({})).await;

    response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn list_board_projects_returns_empty_for_new_board() {
    let server = setup().await;

    let board: serde_json::Value = server
        .post("/boards")
        .json(&json!({"name": "Board"}))
        .await
        .json();
    let board_id = board["id"].as_str().unwrap();

    let response = server.get(&format!("/boards/{board_id}/projects")).await;

    response.assert_status_ok();
    let body: Vec<serde_json::Value> = response.json();
    assert!(body.is_empty());
}

#[tokio::test]
async fn list_board_projects_not_found_returns_404() {
    let server = setup().await;
    let id = Uuid::new_v4();

    let response = server.get(&format!("/boards/{id}/projects")).await;

    response.assert_status(StatusCode::NOT_FOUND);
}
