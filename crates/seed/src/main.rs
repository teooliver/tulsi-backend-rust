use fake::faker::company::en::{BsAdj, BsNoun, CatchPhrase, CompanyName};
use fake::faker::lorem::en::{Paragraph, Sentence};
use fake::Fake;
use rand::prelude::IndexedRandom;
use sqlx::postgres::PgPoolOptions;
use uuid::Uuid;

// The seed user is the Author of every seeded task. Reused across runs because
// emails are unique. It has no password, so it can't log in.
async fn create_seed_user(pool: &sqlx::PgPool) -> Uuid {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO users (name, email) VALUES ('Seed User', 'seed@example.com')
         ON CONFLICT (email) DO UPDATE SET name = EXCLUDED.name
         RETURNING id",
    )
    .fetch_one(pool)
    .await
    .expect("Failed to create seed user");

    println!("Using seed user: seed@example.com ({})", row.0);
    row.0
}

async fn create_board(pool: &sqlx::PgPool) -> Uuid {
    let name: String = CompanyName().fake();
    let description: String = CatchPhrase().fake();

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO boards (name, description) VALUES ($1, $2) RETURNING id",
    )
    .bind(&name)
    .bind(&description)
    .fetch_one(pool)
    .await
    .expect("Failed to create board");

    println!("Created board: {} ({})", name, row.0);
    row.0
}

async fn create_column(pool: &sqlx::PgPool, board_id: Uuid, name: &str, position: i32) -> Uuid {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO columns (name, position, board_id) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(name)
    .bind(position)
    .bind(board_id)
    .fetch_one(pool)
    .await
    .expect("Failed to create column");

    println!("  Created column: {} ({})", name, row.0);
    row.0
}

async fn create_project(pool: &sqlx::PgPool) -> Uuid {
    let adj: String = BsAdj().fake();
    let noun: String = BsNoun().fake();
    let name = format!("{} {}", adj, noun);
    let description: String = Sentence(3..6).fake();

    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO projects (name, description) VALUES ($1, $2) RETURNING id",
    )
    .bind(&name)
    .bind(&description)
    .fetch_one(pool)
    .await
    .expect("Failed to create project");

    println!("  Created project: {} ({})", name, row.0);
    row.0
}

async fn create_task(pool: &sqlx::PgPool, author: Uuid, project_id: Uuid, column_id: Uuid) {
    let title: String = Sentence(2..5).fake();
    let description: String = Paragraph(1..3).fake();

    sqlx::query(
        "INSERT INTO tasks (title, description, author, project_id, column_id) VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(&title)
    .bind(&description)
    .bind(author)
    .bind(project_id)
    .bind(column_id)
    .execute(pool)
    .await
    .expect("Failed to create task");

    println!("    Created task: {}", title);
}

const DEFAULT_COLUMNS: &[&str] = &["To Do", "In Progress", "Review", "Done"];

#[tokio::main]
async fn main() {
    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://db_user_test:12345@localhost:5432/tulsi_test_db".to_string()
    });

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");

    println!("Connected to database. Seeding data...\n");

    let mut rng = rand::rng();

    let author = create_seed_user(&pool).await;

    // Create 2 boards with columns
    let board_1 = create_board(&pool).await;
    let board_2 = create_board(&pool).await;

    println!("\nColumns for Board 1:");
    let mut columns_1 = Vec::new();
    for (i, name) in DEFAULT_COLUMNS.iter().enumerate() {
        let col_id = create_column(&pool, board_1, name, i as i32).await;
        columns_1.push(col_id);
    }

    println!("\nColumns for Board 2:");
    let mut columns_2 = Vec::new();
    for (i, name) in DEFAULT_COLUMNS.iter().enumerate() {
        let col_id = create_column(&pool, board_2, name, i as i32).await;
        columns_2.push(col_id);
    }

    // Create 6 projects: tasks of the first 4 go on board_1's columns, the last 2 on board_2's
    let mut projects_1 = Vec::new();
    let mut projects_2 = Vec::new();

    println!("\nProjects with tasks on Board 1:");
    for _ in 0..4 {
        let project_id = create_project(&pool).await;
        projects_1.push(project_id);
    }

    println!("\nProjects with tasks on Board 2:");
    for _ in 0..2 {
        let project_id = create_project(&pool).await;
        projects_2.push(project_id);
    }

    // Create 10 tasks per project, assigned to random columns of their board
    println!("\nCreating tasks...");
    for project_id in &projects_1 {
        for _ in 0..10 {
            let column_id = *columns_1.choose(&mut rng).unwrap();
            create_task(&pool, author, *project_id, column_id).await;
        }
    }
    for project_id in &projects_2 {
        for _ in 0..10 {
            let column_id = *columns_2.choose(&mut rng).unwrap();
            create_task(&pool, author, *project_id, column_id).await;
        }
    }

    println!("\nSeeding complete!");
    println!("  Boards:   2");
    println!("  Columns:  8 (4 per board)");
    println!("  Projects: 6 (4 + 2)");
    println!("  Tasks:    60 (10 per project)");
}
