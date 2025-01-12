use anyhow::Result;
use geekorm::QueryBuilder;
use std::path::PathBuf;

use crate::codegen;
use crate::utils::database::Database;
use crate::utils::{prompt_select_many, prompt_select_with_default, Config};

pub async fn create_migrations(config: &Config) -> Result<()> {
    log::info!("Initializing a version migration...");

    let selected =
        prompt_select_many("Migration Mode:", &vec!["Schema Update", "Data Migrations"])?;

    if selected.is_empty() {
        log::error!("No migration mode selected");
        return Err(anyhow::anyhow!("No migration mode selected"));
    }

    let path = config.new_migration_path()?;

    if path.exists() {
        let (overwrite, _) = prompt_select_with_default(
            "The migrations directory already exists. Overwrite?",
            &vec!["No", "Yes"],
            0,
        )?;
        if overwrite == "No" {
            log::info!("The migrations directory will not be overwritten");
            return Ok(());
        }
        // TODO: Maybe append a integer to the end of the directory name?
    }

    std::fs::create_dir_all(&path)?;
    log::debug!("New Migration Path: {}", path.display());

    let mod_path = path.join("mod.rs");

    if selected.contains(&"Schema Update") {
        create_schema_migration(config, &path).await?;
    }

    codegen::create_mod(config, &mod_path, selected.contains(&"Data Migrations")).await?;

    codegen::lib_generation(config, &config.migrations_path()?).await?;

    Ok(())
}

async fn create_schema_migration(config: &Config, path: &PathBuf) -> Result<()> {
    log::info!("Creating a schema migration...");

    let database = Database::find_database(config)?;

    // Update the schema
    let upgrade_path = path.join("upgrade.sql");
    tokio::fs::write(&upgrade_path, b"").await?;

    // Rollback the schema
    let downgrade_path = path.join("rollback.sql");
    tokio::fs::write(&downgrade_path, b"").await?;

    // Create database
    let create_path = path.join("create.sql");
    generate_create_sql(&database, &create_path).await?;

    Ok(())
}

/// Creates the `create.sql` file for the schema migration
async fn generate_create_sql(database: &Database, path: &PathBuf) -> Result<()> {
    let mut data = String::new();
    data += "-- GeekORM Database Migrations\n\n";

    for table in &database.tables {
        let query = QueryBuilder::create().table(table.clone()).build()?;
        data += query.to_str();
        data += "\n\n";
    }

    tokio::fs::write(path, data.as_bytes()).await?;
    Ok(())
}
