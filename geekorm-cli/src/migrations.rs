use anyhow::Result;
use std::path::PathBuf;

use crate::codegen;
use crate::utils::database::Database;
use crate::utils::{prompt_select_with_default, Config};

pub async fn create_migrations(config: &Config) -> Result<()> {
    log::info!("Initializing a version migration...");

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

    create_schema_migration(config, &path).await?;

    codegen::lib_generation(config).await?;
    codegen::create_mod(config, &mod_path).await?;

    log::debug!("Formatting the lib/mod file...");
    let fmtdir = if config.crate_mode() {
        config.migrations_path()?
    } else {
        config.working_dir.clone()
    };
    tokio::process::Command::new("cargo")
        .arg("fmt")
        .current_dir(fmtdir)
        .status()
        .await?;

    Ok(())
}

async fn create_schema_migration(config: &Config, path: &PathBuf) -> Result<()> {
    log::info!("Creating a schema migration...");

    let mut database = Database::find_database(config)?;
    database.sort_tables();

    // Creates a new database from scratch
    let create_path = path.join("create.sql");
    log::debug!("Create Path: {}", create_path.display());
    codegen::generate_create_sql(&database, &create_path).await?;

    // Update the schema
    // let upgrade_path = path.join("upgrade.sql");
    // tokio::fs::write(&upgrade_path, b"").await?;

    // Rollback the schema
    // let rollback_path = path.join("rollback.sql");
    // tokio::fs::write(&rollback_path, b"").await?;

    Ok(())
}
