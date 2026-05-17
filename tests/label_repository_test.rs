mod common;

use tulsi_rust_backend::models::label::{CreateLabel, UpdateLabel};
use tulsi_rust_backend::models::task::CreateTask;
use tulsi_rust_backend::repositories::label_repository::LabelRepository;
use tulsi_rust_backend::repositories::task_repository::TaskRepository;
use uuid::Uuid;

struct Repos {
    labels: LabelRepository,
    tasks: TaskRepository,
}

async fn setup() -> Repos {
    let pool = common::setup_test_db().await;
    Repos {
        labels: LabelRepository::new(pool.clone(), None),
        tasks: TaskRepository::new(pool.clone(), None),
    }
}

fn label(name: &str) -> CreateLabel {
    CreateLabel { name: name.to_string(), color: None }
}

async fn make_task(tasks: &TaskRepository, title: &str) -> Uuid {
    tasks
        .create(CreateTask {
            title: title.to_string(),
            description: None,
            project_id: None,
            author: None,
            assigned_to: None,
            column_id: None,
        })
        .await
        .unwrap()
        .id
}

// ── CRUD ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_create_label() {
    let r = setup().await;

    let label = r.labels.create(label("urgent")).await.unwrap();

    assert_eq!(label.name, "urgent");
    assert!(label.color.is_none());
}

#[tokio::test]
async fn test_create_label_with_color() {
    let r = setup().await;

    let label = r
        .labels
        .create(CreateLabel {
            name: "bug".to_string(),
            color: Some("#ff0000".to_string()),
        })
        .await
        .unwrap();

    assert_eq!(label.color.as_deref(), Some("#ff0000"));
}

#[tokio::test]
async fn test_create_duplicate_name_returns_unique_violation() {
    let r = setup().await;
    r.labels.create(label("dup")).await.unwrap();

    let err = r.labels.create(label("dup")).await.unwrap_err();

    match err {
        sqlx::Error::Database(db) => {
            assert_eq!(db.code().as_deref(), Some("23505"));
        }
        other => panic!("expected database error, got {other:?}"),
    }
}

#[tokio::test]
async fn test_find_all_ordered_by_name() {
    let r = setup().await;
    r.labels.create(label("zeta")).await.unwrap();
    r.labels.create(label("alpha")).await.unwrap();
    r.labels.create(label("mu")).await.unwrap();

    let all = r.labels.find_all().await.unwrap();

    assert_eq!(all.len(), 3);
    assert_eq!(all[0].name, "alpha");
    assert_eq!(all[1].name, "mu");
    assert_eq!(all[2].name, "zeta");
}

#[tokio::test]
async fn test_find_by_id_returns_label() {
    let r = setup().await;
    let created = r.labels.create(label("findme")).await.unwrap();

    let found = r.labels.find_by_id(created.id).await.unwrap();

    assert!(found.is_some());
    assert_eq!(found.unwrap().name, "findme");
}

#[tokio::test]
async fn test_find_by_id_not_found() {
    let r = setup().await;

    let found = r.labels.find_by_id(Uuid::new_v4()).await.unwrap();

    assert!(found.is_none());
}

#[tokio::test]
async fn test_update_label_name() {
    let r = setup().await;
    let created = r.labels.create(label("old")).await.unwrap();

    let updated = r
        .labels
        .update(
            created.id,
            UpdateLabel { name: Some("new".to_string()), color: None },
        )
        .await
        .unwrap()
        .unwrap();

    assert_eq!(updated.name, "new");
    assert!(updated.color.is_none());
}

#[tokio::test]
async fn test_update_label_color() {
    let r = setup().await;
    let created = r.labels.create(label("colorless")).await.unwrap();

    let updated = r
        .labels
        .update(
            created.id,
            UpdateLabel {
                name: None,
                color: Some("blue".to_string()),
            },
        )
        .await
        .unwrap()
        .unwrap();

    assert_eq!(updated.name, "colorless");
    assert_eq!(updated.color.as_deref(), Some("blue"));
}

#[tokio::test]
async fn test_update_label_not_found() {
    let r = setup().await;

    let result = r
        .labels
        .update(
            Uuid::new_v4(),
            UpdateLabel { name: Some("x".to_string()), color: None },
        )
        .await
        .unwrap();

    assert!(result.is_none());
}

#[tokio::test]
async fn test_delete_label() {
    let r = setup().await;
    let created = r.labels.create(label("gone")).await.unwrap();

    let deleted = r.labels.delete(created.id).await.unwrap();
    assert!(deleted);

    let found = r.labels.find_by_id(created.id).await.unwrap();
    assert!(found.is_none());
}

#[tokio::test]
async fn test_delete_label_not_found() {
    let r = setup().await;

    let deleted = r.labels.delete(Uuid::new_v4()).await.unwrap();

    assert!(!deleted);
}

// ── Task associations ────────────────────────────────────────────────────────

#[tokio::test]
async fn test_attach_label_to_task() {
    let r = setup().await;
    let task_id = make_task(&r.tasks, "Do it").await;
    let label_id = r.labels.create(label("urgent")).await.unwrap().id;

    let inserted = r.labels.attach_to_task(task_id, label_id).await.unwrap();
    assert!(inserted);

    let labels = r.labels.find_labels_for_task(task_id).await.unwrap();
    assert_eq!(labels.len(), 1);
    assert_eq!(labels[0].id, label_id);
}

#[tokio::test]
async fn test_attach_label_idempotent() {
    let r = setup().await;
    let task_id = make_task(&r.tasks, "Do it").await;
    let label_id = r.labels.create(label("urgent")).await.unwrap().id;

    let first = r.labels.attach_to_task(task_id, label_id).await.unwrap();
    let second = r.labels.attach_to_task(task_id, label_id).await.unwrap();

    assert!(first);
    assert!(!second); // no row inserted the second time
    let labels = r.labels.find_labels_for_task(task_id).await.unwrap();
    assert_eq!(labels.len(), 1);
}

#[tokio::test]
async fn test_detach_label_from_task() {
    let r = setup().await;
    let task_id = make_task(&r.tasks, "Do it").await;
    let label_id = r.labels.create(label("urgent")).await.unwrap().id;
    r.labels.attach_to_task(task_id, label_id).await.unwrap();

    let detached = r.labels.detach_from_task(task_id, label_id).await.unwrap();
    assert!(detached);

    let labels = r.labels.find_labels_for_task(task_id).await.unwrap();
    assert!(labels.is_empty());
}

#[tokio::test]
async fn test_detach_label_not_attached_returns_false() {
    let r = setup().await;
    let task_id = make_task(&r.tasks, "Do it").await;
    let label_id = r.labels.create(label("urgent")).await.unwrap().id;

    let detached = r.labels.detach_from_task(task_id, label_id).await.unwrap();

    assert!(!detached);
}

#[tokio::test]
async fn test_find_labels_for_task_empty() {
    let r = setup().await;
    let task_id = make_task(&r.tasks, "Lone task").await;

    let labels = r.labels.find_labels_for_task(task_id).await.unwrap();

    assert!(labels.is_empty());
}

#[tokio::test]
async fn test_find_labels_for_task_ordered_by_name() {
    let r = setup().await;
    let task_id = make_task(&r.tasks, "T").await;
    let zeta = r.labels.create(label("zeta")).await.unwrap().id;
    let alpha = r.labels.create(label("alpha")).await.unwrap().id;
    let mu = r.labels.create(label("mu")).await.unwrap().id;

    for id in [zeta, alpha, mu] {
        r.labels.attach_to_task(task_id, id).await.unwrap();
    }

    let labels = r.labels.find_labels_for_task(task_id).await.unwrap();

    assert_eq!(labels.len(), 3);
    assert_eq!(labels[0].name, "alpha");
    assert_eq!(labels[1].name, "mu");
    assert_eq!(labels[2].name, "zeta");
}

#[tokio::test]
async fn test_attach_nonexistent_task_returns_fk_error() {
    let r = setup().await;
    let label_id = r.labels.create(label("urgent")).await.unwrap().id;

    let err = r
        .labels
        .attach_to_task(Uuid::new_v4(), label_id)
        .await
        .unwrap_err();

    match err {
        sqlx::Error::Database(db) => {
            assert_eq!(db.code().as_deref(), Some("23503"));
        }
        other => panic!("expected FK violation, got {other:?}"),
    }
}

#[tokio::test]
async fn test_attach_nonexistent_label_returns_fk_error() {
    let r = setup().await;
    let task_id = make_task(&r.tasks, "T").await;

    let err = r
        .labels
        .attach_to_task(task_id, Uuid::new_v4())
        .await
        .unwrap_err();

    match err {
        sqlx::Error::Database(db) => {
            assert_eq!(db.code().as_deref(), Some("23503"));
        }
        other => panic!("expected FK violation, got {other:?}"),
    }
}

#[tokio::test]
async fn test_deleting_task_cascades_attachments() {
    let r = setup().await;
    let task_id = make_task(&r.tasks, "T").await;
    let label_id = r.labels.create(label("urgent")).await.unwrap().id;
    r.labels.attach_to_task(task_id, label_id).await.unwrap();

    r.tasks.delete(task_id).await.unwrap();

    // Label still exists, but the association is gone.
    assert!(r.labels.find_by_id(label_id).await.unwrap().is_some());
    let attached = r.labels.find_labels_for_task(task_id).await.unwrap();
    assert!(attached.is_empty());
}

#[tokio::test]
async fn test_deleting_label_cascades_attachments() {
    let r = setup().await;
    let task_id = make_task(&r.tasks, "T").await;
    let label_id = r.labels.create(label("urgent")).await.unwrap().id;
    r.labels.attach_to_task(task_id, label_id).await.unwrap();

    r.labels.delete(label_id).await.unwrap();

    let attached = r.labels.find_labels_for_task(task_id).await.unwrap();
    assert!(attached.is_empty());
}
