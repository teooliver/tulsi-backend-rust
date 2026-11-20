mod common;

use axum::http::StatusCode;
use axum_test::TestServer;
use serde_json::json;
use uuid::Uuid;

async fn setup() -> TestServer {
    common::authed_server().await
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

async fn current_user_id(server: &TestServer) -> String {
    let me: serde_json::Value = server.get("/auth/me").await.json();
    me["id"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn create_task_sets_author_to_current_user() {
    let server = setup().await;
    let me = current_user_id(&server).await;

    // A client-supplied author is ignored.
    let response = server
        .post("/tasks")
        .json(&json!({"title": "My task", "author": Uuid::new_v4()}))
        .await;

    response.assert_status(StatusCode::CREATED);
    let body: serde_json::Value = response.json();
    assert_eq!(body["author"], me);
}

#[tokio::test]
async fn update_task_cannot_change_author() {
    let server = setup().await;
    let me = current_user_id(&server).await;

    let created: serde_json::Value = server
        .post("/tasks")
        .json(&json!({"title": "My task"}))
        .await
        .json();
    let id = created["id"].as_str().unwrap();

    let response = server
        .put(&format!("/tasks/{id}"))
        .json(&json!({"title": "Renamed", "author": Uuid::new_v4()}))
        .await;

    response.assert_status_ok();
    let body: serde_json::Value = response.json();
    assert_eq!(body["title"], "Renamed");
    assert_eq!(body["author"], me);
}

async fn create_and_delete_task(server: &TestServer, body: serde_json::Value) -> String {
    let created: serde_json::Value = server.post("/tasks").json(&body).await.json();
    let id = created["id"].as_str().unwrap().to_string();
    server
        .delete(&format!("/tasks/{id}"))
        .await
        .assert_status(StatusCode::NO_CONTENT);
    id
}

#[tokio::test]
async fn deleted_task_is_not_found_and_cannot_change() {
    let server = setup().await;
    let id = create_and_delete_task(&server, json!({"title": "Gone"})).await;

    server.get(&format!("/tasks/{id}")).await.assert_status(StatusCode::NOT_FOUND);
    server
        .put(&format!("/tasks/{id}"))
        .json(&json!({"title": "Back"}))
        .await
        .assert_status(StatusCode::NOT_FOUND);
    server.delete(&format!("/tasks/{id}")).await.assert_status(StatusCode::NOT_FOUND);

    let tasks: Vec<serde_json::Value> = server.get("/tasks").await.json();
    assert!(tasks.is_empty());
}

#[tokio::test]
async fn deleted_task_disappears_from_every_list() {
    let server = setup().await;
    let me = current_user_id(&server).await;

    let board: serde_json::Value = server.post("/boards").json(&json!({"name": "B"})).await.json();
    let board_id = board["id"].as_str().unwrap();
    let column: serde_json::Value = server
        .post(&format!("/boards/{board_id}/columns"))
        .json(&json!({"name": "To do"}))
        .await
        .json();
    let column_id = column["id"].as_str().unwrap();
    let project: serde_json::Value = server.post("/projects").json(&json!({"name": "P"})).await.json();
    let project_id = project["id"].as_str().unwrap();
    let view: serde_json::Value = server
        .post("/saved-views")
        .json(&json!({"name": "All", "filters": {}}))
        .await
        .json();
    let view_id = view["id"].as_str().unwrap();

    create_and_delete_task(
        &server,
        json!({
            "title": "Gone",
            "column_id": column_id,
            "project_id": project_id,
            "assigned_to": me,
        }),
    )
    .await;

    for path in [
        format!("/boards/{board_id}/columns/{column_id}/tasks"),
        format!("/projects/{project_id}/tasks"),
        format!("/users/{me}/tasks"),
        format!("/saved-views/{view_id}/tasks"),
    ] {
        let response = server.get(&path).await;
        response.assert_status_ok();
        let tasks: Vec<serde_json::Value> = response.json();
        assert!(tasks.is_empty(), "{path} still lists the deleted task");
    }
}

#[tokio::test]
async fn deleted_task_cannot_be_moved() {
    let server = setup().await;

    let board: serde_json::Value = server.post("/boards").json(&json!({"name": "B"})).await.json();
    let board_id = board["id"].as_str().unwrap();
    let column: serde_json::Value = server
        .post(&format!("/boards/{board_id}/columns"))
        .json(&json!({"name": "Done"}))
        .await
        .json();
    let id = create_and_delete_task(&server, json!({"title": "Gone"})).await;

    server
        .put(&format!("/tasks/{id}/move"))
        .json(&json!({"column_id": column["id"]}))
        .await
        .assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn deleted_task_keeps_its_history() {
    let server = setup().await;

    let created: serde_json::Value = server
        .post("/tasks")
        .json(&json!({"title": "Before"}))
        .await
        .json();
    let id = created["id"].as_str().unwrap();
    server
        .put(&format!("/tasks/{id}"))
        .json(&json!({"title": "After"}))
        .await
        .assert_status_ok();
    server
        .delete(&format!("/tasks/{id}"))
        .await
        .assert_status(StatusCode::NO_CONTENT);

    let response = server.get(&format!("/tasks/{id}/history")).await;
    response.assert_status_ok();
    let history: Vec<serde_json::Value> = response.json();
    assert!(history.iter().any(|e| e["event_type"] == "TitleChanged"
        && e["old_value"] == "Before"
        && e["new_value"] == "After"));
}

#[tokio::test]
async fn deleted_task_labels_are_hidden_and_frozen() {
    let server = setup().await;

    let label: serde_json::Value = server.post("/labels").json(&json!({"name": "urgent"})).await.json();
    let label_id = label["id"].as_str().unwrap();
    let created: serde_json::Value = server.post("/tasks").json(&json!({"title": "T"})).await.json();
    let id = created["id"].as_str().unwrap();
    server
        .post(&format!("/tasks/{id}/labels/{label_id}"))
        .await
        .assert_status(StatusCode::NO_CONTENT);
    server
        .delete(&format!("/tasks/{id}"))
        .await
        .assert_status(StatusCode::NO_CONTENT);

    // Same as for a task that never existed: an empty list, and 404 on changes.
    let labels: Vec<serde_json::Value> = server.get(&format!("/tasks/{id}/labels")).await.json();
    assert!(labels.is_empty());
    server
        .post(&format!("/tasks/{id}/labels/{label_id}"))
        .await
        .assert_status(StatusCode::NOT_FOUND);
    server
        .delete(&format!("/tasks/{id}/labels/{label_id}"))
        .await
        .assert_status(StatusCode::NOT_FOUND);
}
