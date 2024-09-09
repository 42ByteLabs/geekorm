#![allow(dead_code, unused_variables, unused_imports)]
use anyhow::Result;

use geekorm::prelude::*;

#[derive(Debug, Clone, Default, Table, serde::Serialize, serde::Deserialize)]
pub struct Projects {
    #[geekorm(primary_key, auto_increment)]
    id: PrimaryKeyInteger,

    #[geekorm(unique)]
    name: String,

    #[geekorm(search)]
    url: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    init();

    let projects = vec![
        (
            "serde",
            "https://serde.rs/",
        ),
        (
            "tokio",
            "https://tokio.rs/",
        ),
        (
            "actix",
            "https://actix.rs/",
        ),
        (
            "rocket",
            "https://rocket.rs/",
        ),
    ];

    let conn = rusqlite::Connection::open_in_memory().expect("Failed to open database");

    // Create a table
    println!("Creating table 'projects'...");
    Projects::create_table(&conn).await?;
    println!("Table created successfully!\n");

    println!("Inserting data into the table...");
    for (name, url) in projects {
        // Use the Projects::new() constructor to create a new project.
        // This is provided by the Table derive macro when the `new` feature is enabled.
        let mut project = Projects::new(name.to_string(), url.to_string());
        project.fetch_or_create(&conn).await?;

        println!(
            "Project: {} - {}",
            project.name, project.url
        );
    }

    // Query all projects
    let all_projects = Projects::fetch_all(&conn).await?;
    assert_eq!(all_projects.len(), 4);

    // Fetch the project by name (exact match)
    let mut project_serde = Projects::fetch_by_name(&conn, "serde").await?;
    println!(
        "Project Serde: {} - {}\n",
        project_serde.name, project_serde.url
    );
    assert_eq!(project_serde.name, "serde");
    
    // Update the project name (serde -> SerDe)
    project_serde.name = "SerDe".to_string();
    project_serde.update(&conn).await?;
    assert_eq!(project_serde.name, "SerDe");
    
    // Fetch or create a project
    let mut serde = Projects::new("SerDe", "https://serde.rs/");
    serde.fetch_or_create(&conn).await?;
    assert_eq!(serde.name, "SerDe");
    assert_eq!(serde.id, 1.into());

    // First and Last projects
    let first = Projects::first(&conn).await?;
    println!("First Project: {:?}\n", first);
    assert_eq!(first.name, "SerDe");

    let last = Projects::last(&conn).await?;
    println!("Last Project: {:?}\n", last);
    assert_eq!(last.name, "rocket");

    // Search for a project (partial match)
    let result = Projects::search(&conn, "e").await?;
    println!("Search Result: {:#?}\n", result);
    assert_eq!(result.len(), 2);


    Ok(())
}

fn init() {
    println!(
        "{}  - v{}\n",
        geekorm::GEEKORM_BANNER,
        geekorm::GEEKORM_VERSION
    );
    println!("RuSQLite Example\n{:=<40}\n", "=");
    let debug_env: bool = std::env::var("DEBUG").is_ok();
    env_logger::builder()
        .filter_level(if debug_env {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Info
        })
        .init();
}
