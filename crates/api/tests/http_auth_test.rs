mod common;

use axum::http::StatusCode;
use axum_test::TestServer;
use serde_json::json;

async fn setup() -> TestServer {
    TestServer::new(common::setup_test_app().await)
}

#[tokio::test]
async fn register_returns_201_with_token_and_user() {
    let server = setup().await;

    let response = server
        .post("/auth/register")
        .json(&json!({"name": "Alice", "email": "alice@test.com", "password": "secret123"}))
        .await;

    response.assert_status(StatusCode::CREATED);
    let body: serde_json::Value = response.json();
    assert!(body["token"].as_str().is_some_and(|t| !t.is_empty()));
    assert_eq!(body["user"]["name"], "Alice");
    assert_eq!(body["user"]["email"], "alice@test.com");
    assert!(body["user"]["id"].as_str().is_some());
    assert!(body["user"].get("password_hash").is_none());
}

#[tokio::test]
async fn register_duplicate_email_returns_409() {
    let server = setup().await;
    common::register_user(&server, "Alice", "dup@test.com").await;

    let response = server
        .post("/auth/register")
        .json(&json!({"name": "Bob", "email": "dup@test.com", "password": "secret123"}))
        .await;

    response.assert_status(StatusCode::CONFLICT);
}

#[tokio::test]
async fn register_missing_fields_returns_422() {
    let server = setup().await;

    let response = server
        .post("/auth/register")
        .json(&json!({"name": "Alice", "email": "alice@test.com"}))
        .await;

    response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn registered_user_can_log_in() {
    let server = setup().await;
    let user = common::register_user(&server, "Alice", "alice@test.com").await;

    let response = server
        .post("/auth/login")
        .json(&json!({"email": "alice@test.com", "password": "test_password_123"}))
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert!(body["token"].as_str().is_some());
    assert_eq!(body["user"]["id"], user["id"]);
}

#[tokio::test]
async fn login_wrong_password_returns_401() {
    let server = setup().await;
    common::register_user(&server, "Alice", "alice@test.com").await;

    let response = server
        .post("/auth/login")
        .json(&json!({"email": "alice@test.com", "password": "wrong"}))
        .await;

    response.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn login_unknown_email_returns_401() {
    let server = setup().await;

    let response = server
        .post("/auth/login")
        .json(&json!({"email": "nobody@test.com", "password": "whatever"}))
        .await;

    response.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn me_returns_the_logged_in_user() {
    let server = setup().await;
    let token = common::auth_token(&server).await;

    let response = server
        .get("/auth/me")
        .authorization_bearer(token)
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["name"], "Test User");
}

#[tokio::test]
async fn me_without_token_returns_401() {
    let server = setup().await;

    server.get("/auth/me").await.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn post_users_is_no_longer_routed() {
    let server = common::fake_authed_server().await;

    let response = server
        .post("/users")
        .json(&json!({"name": "Alice", "email": "alice@test.com"}))
        .await;

    response.assert_status(StatusCode::METHOD_NOT_ALLOWED);
}
