mod common;

use axum::http::StatusCode;
use axum_test::TestServer;
use serde_json::json;

async fn setup() -> TestServer {
    let app = common::setup_test_app().await;
    TestServer::new(app)
}

#[tokio::test]
async fn full_kanban_workflow() {
    let server = setup().await;

    // 1. Create a board
    let board: serde_json::Value = server
        .post("/boards")
        .json(&json!({"name": "Sprint 1"}))
        .await
        .json();
    let board_id = board["id"].as_str().unwrap();

    // 2. Create columns
    let col_todo: serde_json::Value = server
        .post(&format!("/boards/{board_id}/columns"))
        .json(&json!({"name": "To Do", "position": 0}))
        .await
        .json();
    let col_todo_id = col_todo["id"].as_str().unwrap();

    let col_progress: serde_json::Value = server
        .post(&format!("/boards/{board_id}/columns"))
        .json(&json!({"name": "In Progress", "position": 1}))
        .await
        .json();
    let col_progress_id = col_progress["id"].as_str().unwrap();

    let col_done: serde_json::Value = server
        .post(&format!("/boards/{board_id}/columns"))
        .json(&json!({"name": "Done", "position": 2}))
        .await
        .json();
    let col_done_id = col_done["id"].as_str().unwrap();

    // 3. Create a task in To Do
    let task: serde_json::Value = server
        .post("/tasks")
        .json(&json!({"title": "Implement login", "column_id": col_todo_id}))
        .await
        .json();
    let task_id = task["id"].as_str().unwrap();

    // 4. Verify task is in To Do
    let todo_tasks: Vec<serde_json::Value> = server
        .get(&format!("/boards/{board_id}/columns/{col_todo_id}/tasks"))
        .await
        .json();
    assert_eq!(todo_tasks.len(), 1);
    assert_eq!(todo_tasks[0]["title"], "Implement login");

    // 5. Move task to In Progress
    server
        .put(&format!("/tasks/{task_id}/move"))
        .json(&json!({"column_id": col_progress_id}))
        .await
        .assert_status_ok();

    // 6. Verify task moved
    let todo_tasks: Vec<serde_json::Value> = server
        .get(&format!("/boards/{board_id}/columns/{col_todo_id}/tasks"))
        .await
        .json();
    assert!(todo_tasks.is_empty());

    let progress_tasks: Vec<serde_json::Value> = server
        .get(&format!(
            "/boards/{board_id}/columns/{col_progress_id}/tasks"
        ))
        .await
        .json();
    assert_eq!(progress_tasks.len(), 1);
    assert_eq!(progress_tasks[0]["title"], "Implement login");

    // 7. Move task to Done
    server
        .put(&format!("/tasks/{task_id}/move"))
        .json(&json!({"column_id": col_done_id}))
        .await
        .assert_status_ok();

    let done_tasks: Vec<serde_json::Value> = server
        .get(&format!("/boards/{board_id}/columns/{col_done_id}/tasks"))
        .await
        .json();
    assert_eq!(done_tasks.len(), 1);

    // 8. Verify column ordering is maintained
    let columns: Vec<serde_json::Value> = server
        .get(&format!("/boards/{board_id}/columns"))
        .await
        .json();
    assert_eq!(columns.len(), 3);
    assert_eq!(columns[0]["name"], "To Do");
    assert_eq!(columns[1]["name"], "In Progress");
    assert_eq!(columns[2]["name"], "Done");
}

#[tokio::test]
async fn cascade_delete_board_removes_columns() {
    let server = setup().await;

    let board: serde_json::Value = server
        .post("/boards")
        .json(&json!({"name": "Temp Board"}))
        .await
        .json();
    let board_id = board["id"].as_str().unwrap();

    let col: serde_json::Value = server
        .post(&format!("/boards/{board_id}/columns"))
        .json(&json!({"name": "Col", "position": 0}))
        .await
        .json();
    let col_id = col["id"].as_str().unwrap();

    // Delete board
    server
        .delete(&format!("/boards/{board_id}"))
        .await
        .assert_status(StatusCode::NO_CONTENT);

    // Column should be gone (FK cascade)
    server
        .get(&format!("/boards/{board_id}/columns/{col_id}"))
        .await
        .assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn delete_user_nullifies_task_assignment() {
    let server = setup().await;

    let user: serde_json::Value = server
        .post("/users")
        .json(&json!({"name": "Alice", "email": "alice@test.com"}))
        .await
        .json();
    let user_id = user["id"].as_str().unwrap();

    let task: serde_json::Value = server
        .post("/tasks")
        .json(&json!({"title": "Task", "assigned_to": user_id}))
        .await
        .json();
    let task_id = task["id"].as_str().unwrap();

    // Delete user
    server
        .delete(&format!("/users/{user_id}"))
        .await
        .assert_status(StatusCode::NO_CONTENT);

    // Task should still exist but assigned_to should be null
    let task: serde_json::Value = server
        .get(&format!("/tasks/{task_id}"))
        .await
        .json();
    assert!(task["assigned_to"].is_null());
}

#[tokio::test]
async fn delete_project_nullifies_task_project_id() {
    let server = setup().await;

    let project: serde_json::Value = server
        .post("/projects")
        .json(&json!({"name": "Project"}))
        .await
        .json();
    let project_id = project["id"].as_str().unwrap();

    let task: serde_json::Value = server
        .post("/tasks")
        .json(&json!({"title": "Task", "project_id": project_id}))
        .await
        .json();
    let task_id = task["id"].as_str().unwrap();

    // Delete project
    server
        .delete(&format!("/projects/{project_id}"))
        .await
        .assert_status(StatusCode::NO_CONTENT);

    // Task should still exist but project_id should be null
    let task: serde_json::Value = server
        .get(&format!("/tasks/{task_id}"))
        .await
        .json();
    assert!(task["project_id"].is_null());
}

#[tokio::test]
async fn project_board_relationship() {
    let server = setup().await;

    // Create a board
    let board: serde_json::Value = server
        .post("/boards")
        .json(&json!({"name": "Board"}))
        .await
        .json();
    let board_id = board["id"].as_str().unwrap();

    // Create projects and associate with board via update
    let project: serde_json::Value = server
        .post("/projects")
        .json(&json!({"name": "Project 1"}))
        .await
        .json();
    let project_id = project["id"].as_str().unwrap();

    // Note: CreateProject doesn't have board_id, but we can check via the board's project list
    // The board_id association would need to be set via direct DB or an update
    // For now, verify the endpoint returns an empty list since projects aren't linked
    let board_projects: Vec<serde_json::Value> = server
        .get(&format!("/boards/{board_id}/projects"))
        .await
        .json();

    // Projects aren't linked to board via HTTP API yet (no board_id in CreateProject)
    // This documents the current behavior
    assert!(board_projects.is_empty());

    // Verify the project exists independently
    server
        .get(&format!("/projects/{project_id}"))
        .await
        .assert_status_ok();
}

#[tokio::test]
async fn task_with_full_associations() {
    let server = setup().await;

    // Create all associated entities
    let user: serde_json::Value = server
        .post("/users")
        .json(&json!({"name": "Alice", "email": "alice@test.com"}))
        .await
        .json();
    let user_id = user["id"].as_str().unwrap();

    let project: serde_json::Value = server
        .post("/projects")
        .json(&json!({"name": "Project"}))
        .await
        .json();
    let project_id = project["id"].as_str().unwrap();

    let board: serde_json::Value = server
        .post("/boards")
        .json(&json!({"name": "Board"}))
        .await
        .json();
    let board_id = board["id"].as_str().unwrap();

    let column: serde_json::Value = server
        .post(&format!("/boards/{board_id}/columns"))
        .json(&json!({"name": "To Do", "position": 0}))
        .await
        .json();
    let column_id = column["id"].as_str().unwrap();

    // Create task with all associations
    let task: serde_json::Value = server
        .post("/tasks")
        .json(&json!({
            "title": "Full task",
            "description": "Has everything",
            "project_id": project_id,
            "assigned_to": user_id,
            "column_id": column_id
        }))
        .await
        .json();
    let task_id = task["id"].as_str().unwrap();

    assert_eq!(task["project_id"], project_id);
    assert_eq!(task["assigned_to"], user_id);
    assert_eq!(task["column_id"], column_id);

    // Verify it shows up in all the right places
    let user_tasks: Vec<serde_json::Value> = server
        .get(&format!("/users/{user_id}/tasks"))
        .await
        .json();
    assert_eq!(user_tasks.len(), 1);
    assert_eq!(user_tasks[0]["id"], task_id);

    let project_tasks: Vec<serde_json::Value> = server
        .get(&format!("/projects/{project_id}/tasks"))
        .await
        .json();
    assert_eq!(project_tasks.len(), 1);
    assert_eq!(project_tasks[0]["id"], task_id);

    let column_tasks: Vec<serde_json::Value> = server
        .get(&format!("/boards/{board_id}/columns/{column_id}/tasks"))
        .await
        .json();
    assert_eq!(column_tasks.len(), 1);
    assert_eq!(column_tasks[0]["id"], task_id);
}
