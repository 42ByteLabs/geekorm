//! # Pagination Example
//!
//! This example demonstrates how to use the `chrono` crate with `geekorm`.
use anyhow::Result;
use geekorm::{Connection, ConnectionManager, GEEKORM_BANNER, GEEKORM_VERSION, prelude::*};

#[derive(Table, Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
struct Projects {
    #[geekorm(primary_key, auto_increment)]
    pub id: PrimaryKey<i32>,
    #[geekorm(unique)]
    pub name: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(debug_assertions)]
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    println!("{}     v{}\n", GEEKORM_BANNER, GEEKORM_VERSION);
    // Initialize an in-memory database
    let db = ConnectionManager::in_memory().await?;

    create_projects(db.acquire().await).await?;

    let connection = db.acquire().await;

    let mut page = Projects::paginate(&connection).await?;

    // Get the first page of projects
    let mut projects = page.next(&connection).await?;
    assert_eq!(page.page(), 0);
    println!("Projects :: {:?}", projects);

    projects = page.next(&connection).await?;
    assert_eq!(projects.len(), 100);
    assert_eq!(page.page(), 1);

    Ok(())
}

// Helper function to create 1000 projects
async fn create_projects(connection: Connection<'_>) -> Result<()> {
    Projects::create_table(&connection).await?;

    for pname in 1..=1000 {
        let mut prj = Projects::new(format!("geekorm-{}", pname));
        prj.save(&connection).await?;
    }

    let total = Projects::total(&connection).await?;
    assert_eq!(total, 1000);

    Ok(())
}
