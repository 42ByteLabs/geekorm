//! # Chrono Example
//!
//! This example demonstrates how to use the `chrono` crate with `geekorm`.
use anyhow::Result;
use geekorm::{ConnectionManager, GEEKORM_BANNER, GEEKORM_VERSION, prelude::*};

#[derive(Debug, Clone, Default, Table, serde::Serialize, serde::Deserialize)]
struct Projects {
    #[geekorm(primary_key, auto_increment)]
    pub id: PrimaryKey<i32>,
    #[geekorm(unique)]
    pub name: String,

    pub release: String,

    pub published: chrono::DateTime<chrono::Utc>,

    #[geekorm(new = "chrono::Utc::now()", on_update = "chrono::Utc::now()")]
    pub updated: chrono::DateTime<chrono::Utc>,
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
    let connection = db.acquire().await;

    Projects::create_table(&connection).await?;

    // Three weeks ago
    let three_weeks_ago = chrono::Utc::now() - chrono::Duration::weeks(3);
    let mut gorm = Projects::new("geekorm", "0.1.0", three_weeks_ago);
    gorm.save(&connection).await?;

    assert_eq!(gorm.release, "0.1.0".to_string());
    println!(
        "Project :: {} v{} ({}, {})",
        gorm.name, gorm.release, gorm.published, gorm.updated
    );

    // Get the project updated time
    let updated = gorm.updated.clone();

    // On update we will change the release version and
    // automatically update the updated time (`update` attribute)
    gorm.release = "0.2.0".to_string();
    gorm.update(&connection).await?;

    assert_eq!(gorm.release, "0.2.0".to_string());
    // Confirm that the updated time has changed
    assert_ne!(gorm.updated, updated);

    println!(
        "Project :: {} v{} ({}, {})",
        gorm.name, gorm.release, gorm.published, gorm.updated
    );

    Ok(())
}
