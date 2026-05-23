mod common;

use axum::http::{HeaderValue, StatusCode, header};
use axum_test::TestServer;
use serde_json::json;
use uuid::Uuid;

async fn setup() -> TestServer {
    let app = common::setup_test_app().await;
    TestServer::new(app)
}

fn bearer(token: &str) -> HeaderValue {
    HeaderValue::from_str(&format!("Bearer {token}")).unwrap()
}

async fn register(server: &TestServer) -> String {
    let email = format!("user_{}@example.com", Uuid::new_v4());
    let body: serde_json::Value = server
        .post("/auth/register")
        .json(&json!({
            "name": "Test User",
            "email": email,
            "password": "password123"
        }))
        .await
        .json();
    body["token"].as_str().unwrap().to_string()
}

async fn create_label(server: &TestServer, token: &str, name: &str) -> serde_json::Value {
    server
        .post("/labels")
        .add_header(header::AUTHORIZATION, bearer(token))
        .json(&json!({"name": name}))
        .await
        .json()
}

async fn create_task(server: &TestServer, token: &str, title: &str) -> serde_json::Value {
    server
        .post("/tasks")
        .add_header(header::AUTHORIZATION, bearer(token))
        .json(&json!({"title": title}))
        .await
        .json()
}

// ── CRUD ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn create_label_returns_201_with_body() {
    let server = setup().await;
    let token = register(&server).await;

    let response = server
        .post("/labels")
        .add_header(header::AUTHORIZATION, bearer(&token))
        .json(&json!({"name": "urgent", "color": "#ff0000"}))
        .await;

    response.assert_status(StatusCode::CREATED);
    let body: serde_json::Value = response.json();
    assert_eq!(body["name"], "urgent");
    assert_eq!(body["color"], "#ff0000");
    assert!(body["id"].as_str().is_some());
}

#[tokio::test]
async fn create_label_without_color() {
    let server = setup().await;
    let token = register(&server).await;

    let response = server
        .post("/labels")
        .add_header(header::AUTHORIZATION, bearer(&token))
        .json(&json!({"name": "plain"}))
        .await;

    response.assert_status(StatusCode::CREATED);
    let body: serde_json::Value = response.json();
    assert!(body["color"].is_null());
}

#[tokio::test]
async fn create_duplicate_label_returns_409() {
    let server = setup().await;
    let token = register(&server).await;
    create_label(&server, &token, "dup").await;

    let response = server
        .post("/labels")
        .add_header(header::AUTHORIZATION, bearer(&token))
        .json(&json!({"name": "dup"}))
        .await;

    response.assert_status(StatusCode::CONFLICT);
}

#[tokio::test]
async fn list_labels_returns_200() {
    let server = setup().await;
    let token = register(&server).await;

    for name in ["a", "b", "c"] {
        create_label(&server, &token, name).await;
    }

    let response = server
        .get("/labels")
        .add_header(header::AUTHORIZATION, bearer(&token))
        .await;

    response.assert_status_ok();
    let body: Vec<serde_json::Value> = response.json();
    assert_eq!(body.len(), 3);
    // ordered by name asc
    assert_eq!(body[0]["name"], "a");
    assert_eq!(body[2]["name"], "c");
}

#[tokio::test]
async fn get_label_returns_200() {
    let server = setup().await;
    let token = register(&server).await;
    let created = create_label(&server, &token, "findme").await;
    let id = created["id"].as_str().unwrap();

    let response = server
        .get(&format!("/labels/{id}"))
        .add_header(header::AUTHORIZATION, bearer(&token))
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["name"], "findme");
}

#[tokio::test]
async fn get_label_not_found_returns_404() {
    let server = setup().await;
    let token = register(&server).await;

    server
        .get(&format!("/labels/{}", Uuid::new_v4()))
        .add_header(header::AUTHORIZATION, bearer(&token))
        .await
        .assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn update_label_returns_200() {
    let server = setup().await;
    let token = register(&server).await;
    let created = create_label(&server, &token, "old").await;
    let id = created["id"].as_str().unwrap();

    let response = server
        .put(&format!("/labels/{id}"))
        .add_header(header::AUTHORIZATION, bearer(&token))
        .json(&json!({"name": "new", "color": "green"}))
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["name"], "new");
    assert_eq!(body["color"], "green");
}

#[tokio::test]
async fn update_label_not_found_returns_404() {
    let server = setup().await;
    let token = register(&server).await;

    server
        .put(&format!("/labels/{}", Uuid::new_v4()))
        .add_header(header::AUTHORIZATION, bearer(&token))
        .json(&json!({"name": "nope"}))
        .await
        .assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn update_label_to_existing_name_returns_409() {
    let server = setup().await;
    let token = register(&server).await;
    create_label(&server, &token, "taken").await;
    let other = create_label(&server, &token, "other").await;
    let other_id = other["id"].as_str().unwrap();

    server
        .put(&format!("/labels/{other_id}"))
        .add_header(header::AUTHORIZATION, bearer(&token))
        .json(&json!({"name": "taken"}))
        .await
        .assert_status(StatusCode::CONFLICT);
}

#[tokio::test]
async fn delete_label_returns_204() {
    let server = setup().await;
    let token = register(&server).await;
    let created = create_label(&server, &token, "gone").await;
    let id = created["id"].as_str().unwrap();

    server
        .delete(&format!("/labels/{id}"))
        .add_header(header::AUTHORIZATION, bearer(&token))
        .await
        .assert_status(StatusCode::NO_CONTENT);

    server
        .get(&format!("/labels/{id}"))
        .add_header(header::AUTHORIZATION, bearer(&token))
        .await
        .assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_label_not_found_returns_404() {
    let server = setup().await;
    let token = register(&server).await;

    server
        .delete(&format!("/labels/{}", Uuid::new_v4()))
        .add_header(header::AUTHORIZATION, bearer(&token))
        .await
        .assert_status(StatusCode::NOT_FOUND);
}

// ── Task associations ────────────────────────────────────────────────────────

#[tokio::test]
async fn attach_label_to_task_returns_204() {
    let server = setup().await;
    let token = register(&server).await;
    let task = create_task(&server, &token, "T").await;
    let label = create_label(&server, &token, "urgent").await;
    let task_id = task["id"].as_str().unwrap();
    let label_id = label["id"].as_str().unwrap();

    server
        .post(&format!("/tasks/{task_id}/labels/{label_id}"))
        .add_header(header::AUTHORIZATION, bearer(&token))
        .await
        .assert_status(StatusCode::NO_CONTENT);

    let listed: Vec<serde_json::Value> = server
        .get(&format!("/tasks/{task_id}/labels"))
        .add_header(header::AUTHORIZATION, bearer(&token))
        .await
        .json();

    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0]["id"], label_id);
}

#[tokio::test]
async fn attach_label_is_idempotent() {
    let server = setup().await;
    let token = register(&server).await;
    let task = create_task(&server, &token, "T").await;
    let label = create_label(&server, &token, "urgent").await;
    let task_id = task["id"].as_str().unwrap();
    let label_id = label["id"].as_str().unwrap();

    for _ in 0..2 {
        server
            .post(&format!("/tasks/{task_id}/labels/{label_id}"))
            .add_header(header::AUTHORIZATION, bearer(&token))
            .await
            .assert_status(StatusCode::NO_CONTENT);
    }

    let listed: Vec<serde_json::Value> = server
        .get(&format!("/tasks/{task_id}/labels"))
        .add_header(header::AUTHORIZATION, bearer(&token))
        .await
        .json();

    assert_eq!(listed.len(), 1);
}

#[tokio::test]
async fn attach_label_nonexistent_task_returns_404() {
    let server = setup().await;
    let token = register(&server).await;
    let label = create_label(&server, &token, "urgent").await;
    let label_id = label["id"].as_str().unwrap();

    server
        .post(&format!(
            "/tasks/{}/labels/{label_id}",
            Uuid::new_v4()
        ))
        .add_header(header::AUTHORIZATION, bearer(&token))
        .await
        .assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn attach_nonexistent_label_returns_404() {
    let server = setup().await;
    let token = register(&server).await;
    let task = create_task(&server, &token, "T").await;
    let task_id = task["id"].as_str().unwrap();

    server
        .post(&format!(
            "/tasks/{task_id}/labels/{}",
            Uuid::new_v4()
        ))
        .add_header(header::AUTHORIZATION, bearer(&token))
        .await
        .assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn detach_label_from_task_returns_204() {
    let server = setup().await;
    let token = register(&server).await;
    let task = create_task(&server, &token, "T").await;
    let label = create_label(&server, &token, "urgent").await;
    let task_id = task["id"].as_str().unwrap();
    let label_id = label["id"].as_str().unwrap();
    server
        .post(&format!("/tasks/{task_id}/labels/{label_id}"))
        .add_header(header::AUTHORIZATION, bearer(&token))
        .await;

    server
        .delete(&format!("/tasks/{task_id}/labels/{label_id}"))
        .add_header(header::AUTHORIZATION, bearer(&token))
        .await
        .assert_status(StatusCode::NO_CONTENT);

    let listed: Vec<serde_json::Value> = server
        .get(&format!("/tasks/{task_id}/labels"))
        .add_header(header::AUTHORIZATION, bearer(&token))
        .await
        .json();
    assert!(listed.is_empty());
}

#[tokio::test]
async fn detach_label_not_attached_returns_404() {
    let server = setup().await;
    let token = register(&server).await;
    let task = create_task(&server, &token, "T").await;
    let label = create_label(&server, &token, "urgent").await;
    let task_id = task["id"].as_str().unwrap();
    let label_id = label["id"].as_str().unwrap();

    server
        .delete(&format!("/tasks/{task_id}/labels/{label_id}"))
        .add_header(header::AUTHORIZATION, bearer(&token))
        .await
        .assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn list_task_labels_returns_empty_for_task_with_no_labels() {
    let server = setup().await;
    let token = register(&server).await;
    let task = create_task(&server, &token, "T").await;
    let task_id = task["id"].as_str().unwrap();

    let response = server
        .get(&format!("/tasks/{task_id}/labels"))
        .add_header(header::AUTHORIZATION, bearer(&token))
        .await;

    response.assert_status_ok();
    let body: Vec<serde_json::Value> = response.json();
    assert!(body.is_empty());
}

// ── Auth enforcement ──────────────────────────────────────────────────────────

#[tokio::test]
async fn label_endpoints_require_auth() {
    let server = setup().await;
    let fake_task = Uuid::new_v4();
    let fake_label = Uuid::new_v4();

    server.get("/labels").await.assert_status(StatusCode::UNAUTHORIZED);
    server
        .post("/labels")
        .json(&json!({"name": "x"}))
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
    server
        .get(&format!("/labels/{fake_label}"))
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
    server
        .put(&format!("/labels/{fake_label}"))
        .json(&json!({"name": "x"}))
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
    server
        .delete(&format!("/labels/{fake_label}"))
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
    server
        .get(&format!("/tasks/{fake_task}/labels"))
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
    server
        .post(&format!("/tasks/{fake_task}/labels/{fake_label}"))
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
    server
        .delete(&format!("/tasks/{fake_task}/labels/{fake_label}"))
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
}
