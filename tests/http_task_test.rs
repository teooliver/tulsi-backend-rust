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
async fn create_task_returns_201_with_body() {
    let server = setup().await;

    let response = server
        .post("/tasks")
        .json(&json!({"title": "My task"}))
        .await;

    response.assert_status(StatusCode::CREATED);
    let body: serde_json::Value = response.json();
    assert_eq!(body["title"], "My task");
    assert_eq!(body["description"], "");
    assert!(body["id"].as_str().is_some());
    assert!(body["created_at"].as_str().is_some());
}

#[tokio::test]
async fn create_task_with_all_fields() {
    let server = setup().await;

    let response = server
        .post("/tasks")
        .json(&json!({
            "title": "Full task",
            "description": "A detailed description"
        }))
        .await;

    response.assert_status(StatusCode::CREATED);
    let body: serde_json::Value = response.json();
    assert_eq!(body["title"], "Full task");
    assert_eq!(body["description"], "A detailed description");
}

#[tokio::test]
async fn list_tasks_returns_200_with_empty_array() {
    let server = setup().await;

    let response = server.get("/tasks").await;

    response.assert_status_ok();
    let body: Vec<serde_json::Value> = response.json();
    assert!(body.is_empty());
}

#[tokio::test]
async fn list_tasks_returns_created_tasks() {
    let server = setup().await;

    server
        .post("/tasks")
        .json(&json!({"title": "Task 1"}))
        .await;
    server
        .post("/tasks")
        .json(&json!({"title": "Task 2"}))
        .await;

    let response = server.get("/tasks").await;

    response.assert_status_ok();
    let body: Vec<serde_json::Value> = response.json();
    assert_eq!(body.len(), 2);
}

#[tokio::test]
async fn get_task_by_id_returns_200() {
    let server = setup().await;

    let created: serde_json::Value = server
        .post("/tasks")
        .json(&json!({"title": "Find me"}))
        .await
        .json();
    let id = created["id"].as_str().unwrap();

    let response = server.get(&format!("/tasks/{id}")).await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["title"], "Find me");
    assert_eq!(body["id"], id);
}

#[tokio::test]
async fn get_task_not_found_returns_404() {
    let server = setup().await;
    let id = Uuid::new_v4();

    let response = server.get(&format!("/tasks/{id}")).await;

    response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn get_task_invalid_uuid_returns_400() {
    let server = setup().await;

    let response = server.get("/tasks/not-a-uuid").await;

    response.assert_status(StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn update_task_returns_200() {
    let server = setup().await;

    let created: serde_json::Value = server
        .post("/tasks")
        .json(&json!({"title": "Original"}))
        .await
        .json();
    let id = created["id"].as_str().unwrap();

    let response = server
        .put(&format!("/tasks/{id}"))
        .json(&json!({"title": "Updated"}))
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["title"], "Updated");
}

#[tokio::test]
async fn update_task_partial_fields() {
    let server = setup().await;

    let created: serde_json::Value = server
        .post("/tasks")
        .json(&json!({"title": "Original", "description": "Keep me"}))
        .await
        .json();
    let id = created["id"].as_str().unwrap();

    let response = server
        .put(&format!("/tasks/{id}"))
        .json(&json!({"title": "Changed"}))
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["title"], "Changed");
    assert_eq!(body["description"], "Keep me");
}

#[tokio::test]
async fn update_task_not_found_returns_404() {
    let server = setup().await;
    let id = Uuid::new_v4();

    let response = server
        .put(&format!("/tasks/{id}"))
        .json(&json!({"title": "Nope"}))
        .await;

    response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_task_returns_204() {
    let server = setup().await;

    let created: serde_json::Value = server
        .post("/tasks")
        .json(&json!({"title": "Delete me"}))
        .await
        .json();
    let id = created["id"].as_str().unwrap();

    let response = server.delete(&format!("/tasks/{id}")).await;

    response.assert_status(StatusCode::NO_CONTENT);

    // Verify it's gone
    server
        .get(&format!("/tasks/{id}"))
        .await
        .assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_task_not_found_returns_404() {
    let server = setup().await;
    let id = Uuid::new_v4();

    let response = server.delete(&format!("/tasks/{id}")).await;

    response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn create_task_missing_title_returns_422() {
    let server = setup().await;

    let response = server.post("/tasks").json(&json!({})).await;

    response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn create_task_invalid_json_returns_422() {
    let server = setup().await;

    let response = server
        .post("/tasks")
        .content_type("application/json")
        .bytes("not valid json".into())
        .await;

    // Axum returns 400 for malformed JSON
    response.assert_status(StatusCode::BAD_REQUEST);
}
