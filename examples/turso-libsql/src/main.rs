#![allow(dead_code, unused_variables)]
use anyhow::Result;
use serde::{Deserialize, Serialize};

use geekorm::prelude::*;
use geekorm::{GeekTable, PrimaryKeyInteger};

#[derive(Debug, Clone, Default, GeekTable, Serialize, Deserialize)]
pub struct Projects {
    pub id: PrimaryKeyInteger,
    pub name: String,
    pub url: String,
    pub description: Option<String>,
    pub is_open_source: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    println!(
        "{}  - v{}\n",
        geekorm::GEEKORM_BANNER,
        geekorm::GEEKORM_VERSION
    );
    println!("Turso LibSQL Example\n{:=<40}\n", "=");

    let projects = vec![
        ("serde", "https://serde.rs/", "", true),
        ("tokio", "https://tokio.rs/", "", true),
        ("actix", "https://actix.rs/", "", true),
        ("rocket", "https://rocket.rs/", "", true),
        ("reqwest", "https://reqwest.rs/", "", true),
        ("hyper", "https://hyper.rs/", "", true),
        ("rust", "https://rust-lang.org/", "", true),
    ];
    // Initialize an in-memory database
    let db = libsql::Builder::new_local(":memory:").build().await?;
    let conn = db.connect()?;

    // Create a table
    Projects::create_table(&conn).await?;

    // Insert data into the table
    for (name, url, description, is_open_source) in projects {
        // Use the Projects::new() constructor to create a new project.
        // This is provided by the GeekTable derive macro when the `new` feature is enabled.
        let project = Projects::new(name.to_string(), url.to_string(), is_open_source);

        // Insert the project into the database
        Projects::query(
            // Pass in the connection
            &conn,
            // Build an INSERT query with the data
            Projects::insert(&project),
        )
        .await?;
    }

    // Count the number of projects in the table
    let count = Projects::row_count(&conn, Projects::count().build().unwrap()).await?;
    println!("Number of projects: {}\n", count);

    // Look for a project with the name "serde" (only one should exist)
    let mut project_serde = Projects::query_first(
        &conn,
        // Create a SELECT query with a WHERE clause
        Projects::select()
            .where_eq("name", "serde")
            .limit(1)
            .build()
            .unwrap(),
    )
    .await?;

    println!(
        "Project Serde: {} - {}\n",
        project_serde.name, project_serde.url
    );

    // Query all projects
    let all_projects = Projects::query(&conn, Projects::all()).await?;
    for project in all_projects {
        println!("Project: {:<10} - {}", project.name, project.url);
    }

    // Update the Serde project struct (name and url)
    project_serde.name = "SerDe".to_string();
    project_serde.url = "https://www.youtube.com/watch?v=BI_bHCGRgMY".to_string();
    // Now lets update the project in the database
    Projects::execute(&conn, Projects::update(&project_serde)).await?;

    // Select the updated project
    let sproject = Projects::query_first(
        &conn,
        Projects::select()
            .where_eq("name", "SerDe")
            .limit(1)
            .build()
            .expect("Failed to build SELECT query"),
    )
    .await?;

    println!("\n");

    // Print the updated project
    println!("Updated Project: {} - {}\n", sproject.name, sproject.url);
    assert_eq!(sproject.name, "SerDe");

    Ok(())
}
