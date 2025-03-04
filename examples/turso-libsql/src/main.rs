#![allow(dead_code, unused_variables, unused_imports)]
use std::ops::Deref;

use anyhow::Result;
use geekorm::prelude::*;
use geekorm::ConnectionManager;

mod models;

use models::{ProjectType, Projects, Repository, PROJECTS};

#[tokio::main]
async fn main() -> Result<()> {
    init();

    // Initialize an in-memory database
    // let db = libsql::Builder::new_local(":memory:").build().await?;
    // let connection = db.connect()?;

    // Use the ConnectionManager to create a new connection to the database
    let manager = ConnectionManager::connect(":memory:").await?;
    let conn = manager.acquire().await;

    println!("Connection: {:?}", conn);

    // Create a table
    println!("Creating table 'projects'...");
    Repository::create_table(&conn).await?;
    Projects::create_table(&conn).await?;

    println!("Inserting data into the table...");
    for (name, typ, url, repo) in PROJECTS.iter() {
        // Use the Repository::new() constructor to create a new repository and
        // insert it into the database.
        let mut repository = Repository::new(repo.to_string());
        repository.save(&conn).await?;

        // Use the Projects::new() constructor to create a new project.
        // This is provided by the Table derive macro when the `new` feature is enabled.
        let mut project = Projects::new(name.to_string(), url.to_string(), repository.id);

        // You can also set the values of the struct directly before saving it.
        if project.project_type != *typ {
            // ProjectType isn't set using `.new()` and set to Library by default
            project.project_type = typ.clone();
        }

        project.save(&conn).await?;

        println!(
            "Project: {} - {} (repo: {})",
            project.name, project.url, repository.url
        );
    }

    // Access the number of queries run on the connection
    println!("Queries run: {}", conn.count());
    // If you drop the connection, the connection will be returned to the pool
    drop(conn);

    // Re-acquire the connection
    let conn = manager.acquire().await;

    // Count the number of projects in the table
    let count = Projects::total(&conn).await?;
    println!("Number of projects: {}\n", count);

    // Query all projects
    let all_projects = Projects::all(&conn).await?;

    for project in all_projects {
        println!(
            "Project: {:<10} ({:<12}) - {}",
            project.name, project.project_type, project.url
        );
    }

    let mut project_serde = Projects::fetch_by_name(&conn, "serde").await?;

    println!(
        "Project Serde: {} - {}\n",
        project_serde.name, project_serde.url
    );

    // Update the Serde project struct (name and url)
    project_serde.name = "SerDe".to_string();
    project_serde.url = "https://www.youtube.com/watch?v=BI_bHCGRgMY".to_string();

    // Now lets update the project in the database
    project_serde.update(&conn).await?;

    // Fetch the project repository by the foreign key
    let project_repository = project_serde.fetch_repository(&conn).await?;
    println!("Project Repository: {}", project_repository.url);

    // Fetch the project with the same repository primary key
    let project_repo: Vec<Projects> = Projects::fetch_by_repository(&conn, 3).await?;
    println!("Project by Repository: {:?}", project_repo);
    assert_eq!(project_repo.len(), 1); // Only one project with the repository id of 3

    // Print the updated project
    println!(
        "Updated Project: {} - {}\n",
        project_serde.name, project_serde.url
    );
    assert_eq!(project_serde.name, "SerDe");

    // Delete the project
    let id = project_serde.id.clone();
    project_serde.delete(&conn).await?;

    // Check that the project has been deleted (by counting the number of projects)
    let new_count = Projects::total(&conn).await?;
    assert_eq!(count, new_count + 1);
    println!("Queries run: {}", conn.count());

    Ok(())
}

fn init() {
    println!(
        "{}  - v{}\n",
        geekorm::GEEKORM_BANNER,
        geekorm::GEEKORM_VERSION
    );
    println!("Turso LibSQL Example\n{:=<40}\n", "=");
    let debug_env: bool = std::env::var("DEBUG").is_ok();
    env_logger::builder()
        .filter_level(if debug_env {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Info
        })
        .init();
}
