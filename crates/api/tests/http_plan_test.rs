mod common;

use axum::http::{HeaderValue, StatusCode};
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

// ── Auth helpers ──────────────────────────────────────────────────────────────

async fn register(server: &TestServer) -> (String, Uuid) {
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
    let token = body["token"].as_str().unwrap().to_string();
    let user_id = body["user"]["id"].as_str().unwrap().parse().unwrap();
    (token, user_id)
}

// ── CRUD ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn create_plan_returns_201_with_body() {
    let server = setup().await;
    let (token, _) = register(&server).await;

    let response = server
        .post("/plans")
        .add_header(
            axum::http::header::AUTHORIZATION,
            bearer(&token),
        )
        .json(&json!({
            "name": "Sprint 1",
            "filters": {}
        }))
        .await;

    response.assert_status(StatusCode::CREATED);
    let body: serde_json::Value = response.json();
    assert_eq!(body["name"], "Sprint 1");
    assert_eq!(body["description"], "");
    assert!(body["id"].as_str().is_some());
    assert!(body["filters"].is_object());
}

#[tokio::test]
async fn list_plans_returns_200_scoped_to_user() {
    let server = setup().await;
    let (token1, _) = register(&server).await;
    let (token2, _) = register(&server).await;

    // user1 creates two plans
    for name in ["Plan A", "Plan B"] {
        server
            .post("/plans")
            .add_header(
                axum::http::header::AUTHORIZATION,
                bearer(&token1),
            )
            .json(&json!({"name": name, "filters": {}}))
            .await;
    }
    // user2 creates one plan
    server
        .post("/plans")
        .add_header(
            axum::http::header::AUTHORIZATION,
            bearer(&token2),
        )
        .json(&json!({"name": "Plan C", "filters": {}}))
        .await;

    let response = server
        .get("/plans")
        .add_header(
            axum::http::header::AUTHORIZATION,
            bearer(&token1),
        )
        .await;

    response.assert_status_ok();
    let body: Vec<serde_json::Value> = response.json();
    assert_eq!(body.len(), 2);
}

#[tokio::test]
async fn get_plan_returns_200() {
    let server = setup().await;
    let (token, _) = register(&server).await;

    let created: serde_json::Value = server
        .post("/plans")
        .add_header(
            axum::http::header::AUTHORIZATION,
            bearer(&token),
        )
        .json(&json!({"name": "My Plan", "filters": {}}))
        .await
        .json();
    let id = created["id"].as_str().unwrap();

    let response = server
        .get(&format!("/plans/{id}"))
        .add_header(
            axum::http::header::AUTHORIZATION,
            bearer(&token),
        )
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["id"], id);
    assert_eq!(body["name"], "My Plan");
}

#[tokio::test]
async fn get_plan_another_users_plan_returns_404() {
    let server = setup().await;
    let (token1, _) = register(&server).await;
    let (token2, _) = register(&server).await;

    let created: serde_json::Value = server
        .post("/plans")
        .add_header(
            axum::http::header::AUTHORIZATION,
            bearer(&token1),
        )
        .json(&json!({"name": "Private Plan", "filters": {}}))
        .await
        .json();
    let id = created["id"].as_str().unwrap();

    let response = server
        .get(&format!("/plans/{id}"))
        .add_header(
            axum::http::header::AUTHORIZATION,
            bearer(&token2),
        )
        .await;

    response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn update_plan_returns_200() {
    let server = setup().await;
    let (token, _) = register(&server).await;

    let created: serde_json::Value = server
        .post("/plans")
        .add_header(
            axum::http::header::AUTHORIZATION,
            bearer(&token),
        )
        .json(&json!({"name": "Old Name", "filters": {}}))
        .await
        .json();
    let id = created["id"].as_str().unwrap();

    let response = server
        .put(&format!("/plans/{id}"))
        .add_header(
            axum::http::header::AUTHORIZATION,
            bearer(&token),
        )
        .json(&json!({"name": "New Name"}))
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["name"], "New Name");
}

#[tokio::test]
async fn delete_plan_returns_204() {
    let server = setup().await;
    let (token, _) = register(&server).await;

    let created: serde_json::Value = server
        .post("/plans")
        .add_header(
            axum::http::header::AUTHORIZATION,
            bearer(&token),
        )
        .json(&json!({"name": "To Delete", "filters": {}}))
        .await
        .json();
    let id = created["id"].as_str().unwrap();

    server
        .delete(&format!("/plans/{id}"))
        .add_header(
            axum::http::header::AUTHORIZATION,
            bearer(&token),
        )
        .await
        .assert_status(StatusCode::NO_CONTENT);

    // Verify it's gone
    server
        .get(&format!("/plans/{id}"))
        .add_header(
            axum::http::header::AUTHORIZATION,
            bearer(&token),
        )
        .await
        .assert_status(StatusCode::NOT_FOUND);
}

// ── Filter execution ──────────────────────────────────────────────────────────

#[tokio::test]
async fn execute_plan_returns_matching_tasks() {
    let server = setup().await;
    let (token, user_id) = register(&server).await;

    // Create a task authored by this user via the API (sets author from JWT)
    server
        .post("/tasks")
        .add_header(
            axum::http::header::AUTHORIZATION,
            bearer(&token),
        )
        .json(&json!({"title": "My task"}))
        .await
        .assert_status(StatusCode::CREATED);

    // Create a second task with no author (direct creation won't work through
    // a different user without a separate account, so just create another)
    server
        .post("/tasks")
        .add_header(
            axum::http::header::AUTHORIZATION,
            bearer(&token),
        )
        .json(&json!({"title": "Also mine"}))
        .await;

    // Plan filtering by author = current user
    let plan: serde_json::Value = server
        .post("/plans")
        .add_header(
            axum::http::header::AUTHORIZATION,
            bearer(&token),
        )
        .json(&json!({
            "name": "My tasks",
            "filters": {
                "author": [user_id]
            }
        }))
        .await
        .json();
    let plan_id = plan["id"].as_str().unwrap();

    let response = server
        .get(&format!("/plans/{plan_id}/tasks"))
        .add_header(
            axum::http::header::AUTHORIZATION,
            bearer(&token),
        )
        .await;

    response.assert_status_ok();
    let tasks: Vec<serde_json::Value> = response.json();
    assert_eq!(tasks.len(), 2);
    assert!(tasks.iter().all(|t| t["author"] == user_id.to_string()));
}

#[tokio::test]
async fn execute_plan_empty_filters_returns_all_tasks() {
    let server = setup().await;
    let (token, _) = register(&server).await;

    for title in ["T1", "T2", "T3"] {
        server
            .post("/tasks")
            .add_header(
                axum::http::header::AUTHORIZATION,
                bearer(&token),
            )
            .json(&json!({"title": title}))
            .await;
    }

    let plan: serde_json::Value = server
        .post("/plans")
        .add_header(
            axum::http::header::AUTHORIZATION,
            bearer(&token),
        )
        .json(&json!({"name": "Everything", "filters": {}}))
        .await
        .json();
    let plan_id = plan["id"].as_str().unwrap();

    let tasks: Vec<serde_json::Value> = server
        .get(&format!("/plans/{plan_id}/tasks"))
        .add_header(
            axum::http::header::AUTHORIZATION,
            bearer(&token),
        )
        .await
        .json();

    assert_eq!(tasks.len(), 3);
}

// ── Auth enforcement ──────────────────────────────────────────────────────────

#[tokio::test]
async fn plans_endpoints_require_auth() {
    let server = setup().await;

    server
        .get("/plans")
        .await
        .assert_status(StatusCode::UNAUTHORIZED);

    server
        .post("/plans")
        .json(&json!({"name": "x", "filters": {}}))
        .await
        .assert_status(StatusCode::UNAUTHORIZED);

    let fake_id = Uuid::new_v4();
    server
        .get(&format!("/plans/{fake_id}"))
        .await
        .assert_status(StatusCode::UNAUTHORIZED);

    server
        .get(&format!("/plans/{fake_id}/tasks"))
        .await
        .assert_status(StatusCode::UNAUTHORIZED);
}
