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
async fn create_project_returns_201_with_body() {
    let server = setup().await;

    let response = server
        .post("/projects")
        .json(&json!({"name": "My Project"}))
        .await;

    response.assert_status(StatusCode::CREATED);
    let body: serde_json::Value = response.json();
    assert_eq!(body["name"], "My Project");
    assert_eq!(body["description"], "");
    assert!(body["id"].as_str().is_some());
}

#[tokio::test]
async fn create_project_with_description() {
    let server = setup().await;

    let response = server
        .post("/projects")
        .json(&json!({"name": "Project", "description": "Details here"}))
        .await;

    response.assert_status(StatusCode::CREATED);
    let body: serde_json::Value = response.json();
    assert_eq!(body["description"], "Details here");
}

#[tokio::test]
async fn list_projects_returns_200_with_empty_array() {
    let server = setup().await;

    let response = server.get("/projects").await;

    response.assert_status_ok();
    let body: Vec<serde_json::Value> = response.json();
    assert!(body.is_empty());
}

#[tokio::test]
async fn list_projects_returns_created_projects() {
    let server = setup().await;

    server
        .post("/projects")
        .json(&json!({"name": "Project 1"}))
        .await;
    server
        .post("/projects")
        .json(&json!({"name": "Project 2"}))
        .await;

    let response = server.get("/projects").await;

    response.assert_status_ok();
    let body: Vec<serde_json::Value> = response.json();
    assert_eq!(body.len(), 2);
}

#[tokio::test]
async fn get_project_by_id_returns_200() {
    let server = setup().await;

    let created: serde_json::Value = server
        .post("/projects")
        .json(&json!({"name": "Find me"}))
        .await
        .json();
    let id = created["id"].as_str().unwrap();

    let response = server.get(&format!("/projects/{id}")).await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["name"], "Find me");
}

#[tokio::test]
async fn get_project_not_found_returns_404() {
    let server = setup().await;
    let id = Uuid::new_v4();

    let response = server.get(&format!("/projects/{id}")).await;

    response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn get_project_invalid_uuid_returns_400() {
    let server = setup().await;

    let response = server.get("/projects/not-a-uuid").await;

    response.assert_status(StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn update_project_returns_200() {
    let server = setup().await;

    let created: serde_json::Value = server
        .post("/projects")
        .json(&json!({"name": "Original"}))
        .await
        .json();
    let id = created["id"].as_str().unwrap();

    let response = server
        .put(&format!("/projects/{id}"))
        .json(&json!({"name": "Updated"}))
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["name"], "Updated");
}

#[tokio::test]
async fn update_project_partial_fields() {
    let server = setup().await;

    let created: serde_json::Value = server
        .post("/projects")
        .json(&json!({"name": "Project", "description": "Keep me"}))
        .await
        .json();
    let id = created["id"].as_str().unwrap();

    let response = server
        .put(&format!("/projects/{id}"))
        .json(&json!({"name": "New Name"}))
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["name"], "New Name");
    assert_eq!(body["description"], "Keep me");
}

#[tokio::test]
async fn update_project_not_found_returns_404() {
    let server = setup().await;
    let id = Uuid::new_v4();

    let response = server
        .put(&format!("/projects/{id}"))
        .json(&json!({"name": "Nope"}))
        .await;

    response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_project_returns_204() {
    let server = setup().await;

    let created: serde_json::Value = server
        .post("/projects")
        .json(&json!({"name": "Delete me"}))
        .await
        .json();
    let id = created["id"].as_str().unwrap();

    let response = server.delete(&format!("/projects/{id}")).await;

    response.assert_status(StatusCode::NO_CONTENT);

    server
        .get(&format!("/projects/{id}"))
        .await
        .assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_project_not_found_returns_404() {
    let server = setup().await;
    let id = Uuid::new_v4();

    let response = server.delete(&format!("/projects/{id}")).await;

    response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn create_project_missing_name_returns_422() {
    let server = setup().await;

    let response = server.post("/projects").json(&json!({})).await;

    response.assert_status(StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn list_project_tasks_returns_empty_for_new_project() {
    let server = setup().await;

    let project: serde_json::Value = server
        .post("/projects")
        .json(&json!({"name": "Project"}))
        .await
        .json();
    let project_id = project["id"].as_str().unwrap();

    let response = server
        .get(&format!("/projects/{project_id}/tasks"))
        .await;

    response.assert_status_ok();
    let body: Vec<serde_json::Value> = response.json();
    assert!(body.is_empty());
}

#[tokio::test]
async fn list_project_tasks_not_found_returns_404() {
    let server = setup().await;
    let id = Uuid::new_v4();

    let response = server.get(&format!("/projects/{id}/tasks")).await;

    response.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn list_project_tasks_returns_assigned_tasks() {
    let server = setup().await;

    let project: serde_json::Value = server
        .post("/projects")
        .json(&json!({"name": "Project"}))
        .await
        .json();
    let project_id = project["id"].as_str().unwrap();

    server
        .post("/tasks")
        .json(&json!({"title": "Task in project", "project_id": project_id}))
        .await;
    // Create a task NOT in this project
    server
        .post("/tasks")
        .json(&json!({"title": "Unrelated task"}))
        .await;

    let response = server
        .get(&format!("/projects/{project_id}/tasks"))
        .await;

    response.assert_status_ok();
    let body: Vec<serde_json::Value> = response.json();
    assert_eq!(body.len(), 1);
    assert_eq!(body[0]["title"], "Task in project");
}
