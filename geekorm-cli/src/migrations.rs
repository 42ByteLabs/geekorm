use anyhow::Result;
use quote::quote;
use std::path::PathBuf;

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

    create_mod(config, &mod_path, selected.contains(&"Data Migrations")).await?;

    super::init::lib_generation(config, &config.migrations_path()?).await?;

    Ok(())
}

async fn create_schema_migration(_config: &Config, path: &PathBuf) -> Result<()> {
    log::info!("Creating a schema migration...");

    // Update the schema
    let upgrade_path = path.join("upgrade.sql");
    tokio::fs::write(&upgrade_path, b"").await?;

    // Rollback the schema
    let downgrade_path = path.join("rollback.sql");
    tokio::fs::write(&downgrade_path, b"").await?;

    // Create database
    let create_path = path.join("create.sql");
    tokio::fs::write(&create_path, b"").await?;

    println!("Path: {}", path.display());

    Ok(())
}

async fn create_mod(config: &Config, path: &PathBuf, data: bool) -> Result<()> {
    log::info!("Creating a mod file...");

    let database = Database::find_database(config)?;
    log::debug!("Database: {:#?}", database);

    let tables = database.tables;

    let doctitle = format!("GeekORM Database Migrations - {}", chrono::Utc::now());
    let version = config.version.to_string();

    let data = if data {
        quote! {
            async fn migrate<'a, C>(connection: &'a C) -> Result<(), geekorm::Error>
            where
                C: geekorm::GeekConnection<Connection = C> + 'a,
            {
                todo!("Migrate data...");
            }
        }
    } else {
        quote! {}
    };

    let database_ast = quote! {
        pub static ref Database: geekorm::Database = geekorm::Database {
            tables: Vec::from([
                #(#tables),*
            ])
        };
    };

    let ast = quote! {
        #![doc = #doctitle]

        pub struct Migration;

        impl geekorm::Migration for Migration {
            fn version() -> &'static str {
                #version
            }

            #data

            fn create_query() -> &'static str {
                include_str!("create.sql")
            }
            fn upgrade_query() -> &'static str {
                include_str!("upgrade.sql")
            }
            fn rollback_query() -> &'static str {
                include_str!("rollback.sql")
            }
        }

        // Static Database Tables
        lazy_static::lazy_static! {
            #database_ast
        }

    };

    tokio::fs::write(path, ast.to_string().as_bytes()).await?;

    Ok(())
}
