#![allow(dead_code, unused_variables, unused_imports)]
use anyhow::Result;

use geekorm::{prelude::*, ForeignKey, PrimaryKeyInteger};

#[derive(Debug, Clone, Default, GeekTable, serde::Serialize, serde::Deserialize)]
pub struct Repository {
    pub id: PrimaryKeyInteger,
    pub url: String,
}

#[derive(Debug, Clone, Default, GeekTable, serde::Serialize, serde::Deserialize)]
pub struct Projects {
    pub id: PrimaryKeyInteger,
    pub name: String,
    pub url: String,

    #[geekorm(foreign_key = "Repository.id")]
    pub repository: ForeignKey<i32, Repository>,
}

#[tokio::main]
async fn main() -> Result<()> {
    init();

    let projects = vec![
        (
            "serde",
            "https://serde.rs/",
            "https://github.com/serde-rs/serde",
        ),
        (
            "tokio",
            "https://tokio.rs/",
            "https://github.com/tokio-rs/tokio",
        ),
        (
            "actix",
            "https://actix.rs/",
            "https://github.com/actix/actix-web",
        ),
        (
            "rocket",
            "https://rocket.rs/",
            "https://github.com/rwf2/Rocket",
        ),
        (
            "reqwest",
            "https://docs.rs/reqwest/latest/reqwest/",
            "https://github.com/seanmonstar/reqwest",
        ),
        (
            "hyper",
            "https://hyper.rs/",
            "https://github.com/hyperium/hyper",
        ),
        (
            "rust",
            "https://rust-lang.org/",
            "https://github.com/rust-lang/rust/",
        ),
    ];
    // Initialize an in-memory database
    let db = libsql::Builder::new_local(":memory:").build().await?;
    // let db = libsql::Builder::new_local("/tmp/turso-testing.sqlite").build().await?;
    let conn = db.connect()?;

    // Create a table
    println!("Creating table 'projects'...");
    Repository::create_table(&conn).await?;
    Projects::create_table(&conn).await?;

    println!("Inserting data into the table...");
    for (name, url, repo) in projects {
        // Use the Repository::new() constructor to create a new repository and 
        // insert it into the database.
        let mut repository = Repository::new(repo.to_string());
        repository.execute_insert(&conn).await?;

        // Use the Projects::new() constructor to create a new project.
        // This is provided by the GeekTable derive macro when the `new` feature is enabled.
        let mut project = Projects::new(name.to_string(), url.to_string(), repository.id);
        project.execute_insert(&conn).await?;

        println!("Project: {} - {} (repo: {})", project.name, project.url, repository.url);
    }

    // Count the number of projects in the table
    let count = Projects::row_count(&conn, Projects::count().build().unwrap()).await?;
    println!("Number of projects: {}\n", count);

    // Look for a project with the name "serde" (only one should exist)
    println!("Querying for project with name 'serde'...");
    let query = Projects::select()
        .where_eq("name", "serde")
        .limit(1)
        .build()
        .unwrap();

    println!("Query: {}", query);
    let mut project_serde = Projects::query_first(
        &conn,
        // Create a SELECT query with a WHERE clause
        Projects::select()
            .where_eq("name", "serde")
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
    project_serde.execute_update(&conn).await?;

    // Fetch the project repository by the foreign key
    let project_repository = project_serde.fetch_repository(&conn).await?;
    println!("\nProject Repository: {}", project_repository.url);

    println!("\n");

    // Print the updated project
    println!(
        "Updated Project: {} - {}\n",
        project_serde.name, project_serde.url
    );
    assert_eq!(project_serde.name, "SerDe");

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
