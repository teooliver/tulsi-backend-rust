mod common;

use axum::http::StatusCode;
use axum_test::TestServer;
use serde_json::json;
use uuid::Uuid;

async fn setup() -> TestServer {
    common::fake_authed_server().await
}

#[tokio::test]
async fn list_users_returns_200_with_empty_array() {
    let server = setup().await;

    let response = server.get("/users").await;

    response.assert_status_ok();
    let body: Vec<serde_json::Value> = response.json();
    assert!(body.is_empty());
}

#[tokio::test]
async fn list_users_returns_created_users() {
    let server = setup().await;

    common::register_user(&server, "Alice", "alice@test.com").await;
    common::register_user(&server, "Bob", "bob@test.com").await;

    let response = server.get("/users").await;

    response.assert_status_ok();
    let body: Vec<serde_json::Value> = response.json();
    assert_eq!(body.len(), 2);
}

#[tokio::test]
async fn get_user_by_id_returns_200() {
    let server = setup().await;

    let created = common::register_user(&server, "Alice", "alice@test.com").await;
    let id = created["id"].as_str().unwrap();

    let response = server.get(&format!("/users/{id}")).await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["name"], "Alice");
    assert_eq!(body["email"], "alice@test.com");
}

#[tokio::test]
async fn get_user_not_found_returns_404() {
    let server = setup().await;
    let id = Uuid::new_v4();

    let response = server.get(&format!("/users/{id}")).await;

    response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn get_user_invalid_uuid_returns_400() {
    let server = setup().await;

    let response = server.get("/users/not-a-uuid").await;

    response.assert_status(StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn update_user_returns_200() {
    let server = setup().await;

    let created = common::register_user(&server, "Alice", "alice@test.com").await;
    let id = created["id"].as_str().unwrap();

    let response = server
        .put(&format!("/users/{id}"))
        .json(&json!({"name": "Alice Updated"}))
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["name"], "Alice Updated");
    assert_eq!(body["email"], "alice@test.com");
}

#[tokio::test]
async fn update_user_partial_fields() {
    let server = setup().await;

    let created = common::register_user(&server, "Alice", "alice@test.com").await;
    let id = created["id"].as_str().unwrap();

    let response = server
        .put(&format!("/users/{id}"))
        .json(&json!({"email": "newemail@test.com"}))
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["name"], "Alice");
    assert_eq!(body["email"], "newemail@test.com");
}

#[tokio::test]
async fn update_user_not_found_returns_404() {
    let server = setup().await;
    let id = Uuid::new_v4();

    let response = server
        .put(&format!("/users/{id}"))
        .json(&json!({"name": "Nope"}))
        .await;

    response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_user_returns_204() {
    let server = setup().await;

    let created = common::register_user(&server, "Alice", "alice@test.com").await;
    let id = created["id"].as_str().unwrap();

    let response = server.delete(&format!("/users/{id}")).await;

    response.assert_status(StatusCode::NO_CONTENT);

    server
        .get(&format!("/users/{id}"))
        .await
        .assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_user_not_found_returns_404() {
    let server = setup().await;
    let id = Uuid::new_v4();

    let response = server.delete(&format!("/users/{id}")).await;

    response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn list_user_tasks_returns_empty_for_new_user() {
    let server = setup().await;

    let user = common::register_user(&server, "Alice", "alice@test.com").await;
    let user_id = user["id"].as_str().unwrap();

    let response = server.get(&format!("/users/{user_id}/tasks")).await;

    response.assert_status_ok();
    let body: Vec<serde_json::Value> = response.json();
    assert!(body.is_empty());
}

#[tokio::test]
async fn list_user_tasks_not_found_returns_404() {
    let server = setup().await;
    let id = Uuid::new_v4();

    let response = server.get(&format!("/users/{id}/tasks")).await;

    response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn list_user_tasks_returns_assigned_tasks() {
    // Creating tasks needs a real user: the Author is a foreign key to users.
    let server = common::authed_server().await;

    let user = common::register_user(&server, "Alice", "alice@test.com").await;
    let user_id = user["id"].as_str().unwrap();

    server
        .post("/tasks")
        .json(&json!({"title": "Alice's task", "assigned_to": user_id}))
        .await;
    // Create a task NOT assigned to this user
    server
        .post("/tasks")
        .json(&json!({"title": "Unassigned task"}))
        .await;

    let response = server.get(&format!("/users/{user_id}/tasks")).await;

    response.assert_status_ok();
    let body: Vec<serde_json::Value> = response.json();
    assert_eq!(body.len(), 1);
    assert_eq!(body[0]["title"], "Alice's task");
}
