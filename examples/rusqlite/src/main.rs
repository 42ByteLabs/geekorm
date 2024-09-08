#![allow(dead_code, unused_variables, unused_imports)]
use anyhow::Result;

use geekorm::prelude::*;

#[derive(Debug, Clone, Default, Table, serde::Serialize, serde::Deserialize)]
pub struct Repository {
    #[geekorm(primary_key, auto_increment)]
    pub id: PrimaryKeyInteger,
    pub url: String,
}

#[derive(Data, Debug, Clone, Default)]
pub enum ProjectType {
    #[default]
    Library,
    Application,
    Framework,
    Tool,
}

#[derive(Debug, Clone, Default, Table, serde::Serialize, serde::Deserialize)]
pub struct Projects {
    #[geekorm(primary_key, auto_increment)]
    pub id: PrimaryKeyInteger,

    #[geekorm(unique)]
    pub name: String,

    #[geekorm(new = "ProjectType::Library")]
    pub project_type: ProjectType,

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

    let conn = rusqlite::Connection::open_in_memory().expect("Failed to open database");

    // Create a table
    println!("Creating table 'projects'...");
    Repository::create_table(&conn).await?;
    Projects::create_table(&conn).await?;
    println!("Table created successfully!\n");

    println!("Inserting data into the table...");
    for (name, url, repo) in projects {
        // Use the Repository::new() constructor to create a new repository and
        // insert it into the database.
        let mut repository = Repository::new(repo.to_string());
        repository.save(&conn).await?;

        // Use the Projects::new() constructor to create a new project.
        // This is provided by the Table derive macro when the `new` feature is enabled.
        let mut project = Projects::new(name.to_string(), url.to_string(), repository.id);
        project.save(&conn).await?;

        println!(
            "Project: {} - {} (repo: {})",
            project.name, project.url, repository.url
        );
    }

    // Count the number of projects in the table
    let count = Projects::row_count(&conn, Projects::query_count().build().unwrap()).await?;
    println!("Number of projects: {}\n", count);

    // Query all projects
    let all_projects = Projects::fetch_all(&conn).await?;

    for project in all_projects {
        println!("Project: {:<10} - {}", project.name, project.url);
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
    println!("SQLite Example\n{:=<40}\n", "=");
    let debug_env: bool = std::env::var("DEBUG").is_ok();
    env_logger::builder()
        .filter_level(if debug_env {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Info
        })
        .init();
}
