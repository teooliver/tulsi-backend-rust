mod common;

use axum::http::StatusCode;
use serde_json::json;

#[tokio::test]
async fn full_user_journey() {
    let server = common::authed_server().await;

    // 1. Create a project
    let project: serde_json::Value = server
        .post("/projects")
        .json(&json!({"name": "Launch"}))
        .await
        .json();
    let project_id = project["id"].as_str().unwrap().to_string();

    // 2. Create several tasks assigned to the project
    let task_titles = ["Spec", "Design", "Build", "Test", "Ship"];
    let mut task_ids: Vec<String> = Vec::new();
    for title in task_titles {
        let task: serde_json::Value = server
            .post("/tasks")
            .json(&json!({"title": title, "project_id": project_id}))
            .await
            .json();
        task_ids.push(task["id"].as_str().unwrap().to_string());
    }

    // Verify all 5 tasks are linked to the project
    let project_tasks: Vec<serde_json::Value> = server
        .get(&format!("/projects/{project_id}/tasks"))
        .await
        .json();
    assert_eq!(project_tasks.len(), 5);

    // 3. Create a label and attach it to the first 3 tasks
    let label: serde_json::Value = server
        .post("/labels")
        .json(&json!({"name": "urgent", "color": "#ff0000"}))
        .await
        .json();
    let label_id = label["id"].as_str().unwrap().to_string();

    let labeled_task_ids = &task_ids[..3];
    for task_id in labeled_task_ids {
        server
            .post(&format!("/tasks/{task_id}/labels/{label_id}"))
            .await
            .assert_status(StatusCode::NO_CONTENT);
    }

    // Sanity: first task has the label
    let first_task_labels: Vec<serde_json::Value> = server
        .get(&format!("/tasks/{}/labels", labeled_task_ids[0]))
        .await
        .json();
    assert_eq!(first_task_labels.len(), 1);
    assert_eq!(first_task_labels[0]["id"], label_id);

    // 4. Create a plan filtered by that label
    let plan: serde_json::Value = server
        .post("/plans")
        .json(&json!({
            "name": "Urgent work",
            "filters": {"label_id": [label_id]}
        }))
        .await
        .json();
    let plan_id = plan["id"].as_str().unwrap().to_string();

    let plan_tasks: Vec<serde_json::Value> = server
        .get(&format!("/plans/{plan_id}/tasks"))
        .await
        .json();
    assert_eq!(plan_tasks.len(), 3);
    let returned_ids: std::collections::HashSet<String> = plan_tasks
        .iter()
        .map(|t| t["id"].as_str().unwrap().to_string())
        .collect();
    let expected_ids: std::collections::HashSet<String> = labeled_task_ids.iter().cloned().collect();
    assert_eq!(returned_ids, expected_ids);

    // 5. Create a board with Todo / In Progress / Done columns
    let board: serde_json::Value = server
        .post("/boards")
        .json(&json!({"name": "Sprint"}))
        .await
        .json();
    let board_id = board["id"].as_str().unwrap().to_string();

    let todo: serde_json::Value = server
        .post(&format!("/boards/{board_id}/columns"))
        .json(&json!({"name": "Todo", "position": 0}))
        .await
        .json();
    let todo_id = todo["id"].as_str().unwrap().to_string();

    let in_progress: serde_json::Value = server
        .post(&format!("/boards/{board_id}/columns"))
        .json(&json!({"name": "In Progress", "position": 1}))
        .await
        .json();
    let in_progress_id = in_progress["id"].as_str().unwrap().to_string();

    let done: serde_json::Value = server
        .post(&format!("/boards/{board_id}/columns"))
        .json(&json!({"name": "Done", "position": 2}))
        .await
        .json();
    let done_id = done["id"].as_str().unwrap().to_string();

    // 6. Place the 3 labeled tasks in Todo
    for task_id in labeled_task_ids {
        server
            .put(&format!("/tasks/{task_id}/move"))
            .json(&json!({"column_id": todo_id}))
            .await
            .assert_status_ok();
    }

    let todo_tasks: Vec<serde_json::Value> = server
        .get(&format!("/boards/{board_id}/columns/{todo_id}/tasks"))
        .await
        .json();
    assert_eq!(todo_tasks.len(), 3);

    // 7. Move tasks along the columns
    // Task 0: Todo -> In Progress
    server
        .put(&format!("/tasks/{}/move", labeled_task_ids[0]))
        .json(&json!({"column_id": in_progress_id}))
        .await
        .assert_status_ok();

    let counts_after_first_move = column_counts(&server, &board_id, &[&todo_id, &in_progress_id, &done_id]).await;
    assert_eq!(counts_after_first_move, [2, 1, 0]);

    // Task 1: Todo -> In Progress -> Done
    server
        .put(&format!("/tasks/{}/move", labeled_task_ids[1]))
        .json(&json!({"column_id": in_progress_id}))
        .await
        .assert_status_ok();
    server
        .put(&format!("/tasks/{}/move", labeled_task_ids[1]))
        .json(&json!({"column_id": done_id}))
        .await
        .assert_status_ok();

    // 8. Final state: 1 in Todo, 1 in In Progress, 1 in Done
    let final_counts = column_counts(&server, &board_id, &[&todo_id, &in_progress_id, &done_id]).await;
    assert_eq!(final_counts, [1, 1, 1]);

    let done_tasks: Vec<serde_json::Value> = server
        .get(&format!("/boards/{board_id}/columns/{done_id}/tasks"))
        .await
        .json();
    assert_eq!(done_tasks[0]["id"], labeled_task_ids[1]);
}

async fn column_counts(
    server: &axum_test::TestServer,
    board_id: &str,
    column_ids: &[&str],
) -> Vec<usize> {
    let mut counts = Vec::with_capacity(column_ids.len());
    for col_id in column_ids {
        let tasks: Vec<serde_json::Value> = server
            .get(&format!("/boards/{board_id}/columns/{col_id}/tasks"))
            .await
            .json();
        counts.push(tasks.len());
    }
    counts
}
