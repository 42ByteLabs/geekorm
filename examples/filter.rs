//! # Filter Example
//!
//! This example demonstrates how to use the `chrono` crate with `geekorm`.
use anyhow::Result;
use geekorm::{GEEKORM_BANNER, GEEKORM_VERSION, prelude::*};

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
    let connection = create_projects().await?;

    // Filter projects by name, multiple filters using `and`
    let results =
        Projects::filter(&connection, vec![("name", "serde"), ("name", "geekorm")]).await?;
    assert_eq!(results.len(), 2);
    println!("Results: {:?}", results);

    // Filter out projects by name
    let filter_out =
        Projects::filter(&connection, vec![("!name", "serde"), ("!name", "sqlx")]).await?;
    println!("Filtered out results:");
    assert_eq!(filter_out.len(), 5);
    for project in filter_out {
        println!("Project: {:?}", project);
    }

    Ok(())
}

async fn create_projects() -> Result<libsql::Connection> {
    // Initialize an in-memory database
    let db = libsql::Builder::new_local(":memory:").build().await?;
    let connection = db.connect()?;
    Projects::create_table(&connection).await?;

    let project_names = vec![
        "serde", "tokio", "actix", "rocket", "geekorm", "sqlx", "libsql",
    ];

    for pname in project_names {
        let mut prj = Projects::new(pname);
        prj.save(&connection).await?;
    }

    let total = Projects::total(&connection).await?;
    assert_eq!(total, 7);

    Ok(connection)
}
