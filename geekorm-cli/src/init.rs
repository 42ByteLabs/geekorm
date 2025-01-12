use std::path::PathBuf;

use crate::utils::{prompt_input_with_default, prompt_select, Config};
use anyhow::Result;
use quote::{format_ident, quote};

pub async fn init(path: &PathBuf) -> Result<Config> {
    log::info!("Initializing GeekORM...");
    // TODO: Check if the configuration file already exists

    let mut config = Config::default();
    let (selected, _) = prompt_select("Migration Mode:", &vec!["Crate", "Module"])?;

    match selected {
        "Crate" => {
            config.mode = "crate".to_string();
        }
        "Module" => {
            log::error!("Module mode is not yet supported");
            return Err(anyhow::anyhow!("Module mode is not yet supported"));
        }
        _ => {
            log::error!("Invalid mode selected");
            return Err(anyhow::anyhow!("Invalid mode selected"));
        }
    }

    let name = prompt_input_with_default("Name:", &config.name())?;
    config.name = Some(name);

    let (database, _) = prompt_select("Database:", &vec!["SQLite"])?;
    config.database = database.to_lowercase().to_string();

    let (driver, _) = prompt_select("Driver:", &vec!["libsql", "rustqlite", "none"])?;
    config.driver = driver.to_lowercase().to_string();

    config.save(path)?;

    if config.mode == "crate" {
        init_crate(&config).await?;
    }

    Ok(config)
}

/// Initialize the crate mode
///
/// - Create the `migrations` directory
/// - Create the `migrations` rust project with the `migrations` directory as the workspace
pub async fn init_crate(config: &Config) -> Result<()> {
    log::info!("Initializing the crate mode...");

    let name = config.name();
    let migrations_dir = config.migrations_path()?;
    // let migrations_src_dir = migrations_dir.join("src");
    log::debug!("Migrations directory: {}", migrations_dir.display());

    log::info!("Creating the migrations directory...");
    if !migrations_dir.exists() {
        std::fs::create_dir_all(&migrations_dir)?;
        log::debug!("The migrations directory has been created");
    }

    // Setup the migrations project
    tokio::process::Command::new("cargo")
        .args(&["init", "--name", name.as_str(), "--lib", "--vcs", "none"])
        .current_dir(&migrations_dir)
        .status()
        .await?;

    let mut features = vec!["migrations", "backends"];
    match config.driver.as_str() {
        "libsql" => features.push("libsql"),
        "rustqlite" => features.push("rustqlite"),
        _ => {}
    }

    // Add dependencies
    tokio::process::Command::new("cargo")
        .arg("add")
        .arg("geekorm")
        .arg("-F")
        .arg(features.join(","))
        .current_dir(&migrations_dir)
        .status()
        .await?;
    //
    tokio::process::Command::new("cargo")
        .arg("add")
        .arg("lazy_static@1")
        .current_dir(&migrations_dir)
        .status()
        .await?;

    // Add the migrations directory as a dependency
    tokio::process::Command::new("cargo")
        .args(&["add", "--path", "./migrations"])
        .status()
        .await?;

    // Rust files

    // Save the database schema
    let database_path = migrations_dir.join("src/database.json");
    log::debug!(
        "Saving the database schema to `{}`",
        database_path.display()
    );
    // let database_json = serde_json::to_string_pretty(&database.tables)?;
    // std::fs::write(&database_path, database_json)?;

    Ok(())
}

pub async fn lib_generation(_config: &Config, path: &PathBuf) -> Result<()> {
    log::info!("Generating the lib file...");
    let src_dir = path.join("src");
    let lib_file = src_dir.join("lib.rs");

    let mut latest = format_ident!("v0_0_0");
    let mut imports = vec![];

    // Get a list of dirs that start with "v"
    let mut dirs = tokio::fs::read_dir(&src_dir).await?;

    while let Some(dir) = dirs.next_entry().await? {
        if dir.file_type().await?.is_dir() {
            let name = dir.file_name();
            if let Some(name) = name.to_str() {
                if name.starts_with("v") {
                    // Latest == last version
                    latest = format_ident!("{}", name);

                    let impstmt = quote! {
                        mod #latest;
                    };
                    imports.push(impstmt);
                }
            }
        }
    }

    let ast = quote! {
        //! GeekORM Database Migrations
        #[allow(unused_imports, unused_variables)]
        use geekorm::prelude::*;

        #( #imports )*

        pub use #latest::{Database, Migration as LatestMigration};

        pub async fn init<'a, T>(connection: &'a T) -> Result<(), geekorm::Error>
        where
            T: geekorm::GeekConnection<Connection = T> + 'a,
        {
            let database = &Database;

            match LatestMigration::validate(connection).await? {
                MigrationState::Initialized => {
                    LatestMigration::create(connection, database).await?;
                }
                MigrationState::OutOfDate => {
                    LatestMigration::upgrade(connection).await?;
                }
                _ => {
                    return Err(geekorm::Error::Unknown);
                }
            }

            Ok(())
        }
    };

    log::debug!("Updating the src/lib.rs file...");

    tokio::fs::write(&lib_file, ast.to_string().as_bytes()).await?;

    tokio::process::Command::new("cargo")
        .arg("fmt")
        .current_dir(&src_dir)
        .status()
        .await?;

    Ok(())
}
