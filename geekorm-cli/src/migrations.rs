use anyhow::Result;
use geekorm::{GeekConnection, MigrationState};
use geekorm_core::builder::alter::AlterMode;
use geekorm_core::error::MigrationError;
use geekorm_core::migrations::validate::Validator;
use geekorm_core::{AlterQuery, ToSqlite};
use std::path::PathBuf;

use crate::codegen;
use crate::utils::database::Database;
use crate::utils::{prompt_select, prompt_select_with_default, Config};

pub async fn create_migrations(config: &mut Config) -> Result<()> {
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

    // Data migrations
    if !config.data_migrations && !config.migrations_data_path()?.exists() {
        log::debug!("Prompting for data migrations...");
        let (data_migrations, _) = prompt_select_with_default(
            "Would you like to create data migrations?",
            &vec!["No", "Yes"],
            0,
        )?;
        config.data_migrations = data_migrations == "Yes";
    }

    log::info!("Running a build...");
    let build_cmd = config.build_command()?;

    if let Some(cmd) = config.build_command()?.first() {
        let rest = &build_cmd[1..];
        log::debug!("Running build command: {} [ {} ]", cmd, rest.join(", "));

        tokio::process::Command::new(cmd)
            .args(rest)
            .current_dir(&config.working_dir)
            .output()
            .await?;
        log::debug!("Building complete");
    } else {
        log::warn!("No build command specified, skipping build");
    }

    if create_schema_migration(config, &path).await? {
        let mod_path = path.join("mod.rs");

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
    } else {
        log::info!("No schema migration created");
        log::info!("If this is incorrect, please run `geekorm test` to validate");
    }

    Ok(())
}

/// Creates a schema migration if the database is out of date
///
/// If the database is up to date, this function does nothing and returns false
async fn create_schema_migration(config: &Config, path: &PathBuf) -> Result<bool> {
    log::debug!("Creating a schema migration...");

    let mut database = Database::find_database(config)?;
    database.sort_tables();

    // Update the schema
    let upgrade_path = path.join("upgrade.sql");
    if upgrade_path.exists() {
        log::debug!("Removing the upgrade.sql file...");
        tokio::fs::remove_file(&upgrade_path).await?;
    }

    let validator = test_migrations(config).await?;

    if !validator.errors.is_empty() {
        log::info!("Errors found, creating a schema migration...");

        let mut data = "-- This migration will update the schema\n\n".to_string();

        for verror in validator.errors.iter() {
            log::info!("Error: {}", verror);

            let query = prompt_table_alter(&database, verror)?;

            data.push_str(query.as_str());
            data.push_str("\n\n");
        }

        tokio::fs::write(&upgrade_path, data.as_bytes()).await?;

        // Creates a new database from scratch
        let create_path = path.join("create.sql");
        log::debug!("Create Path: {}", create_path.display());
        codegen::generate_create_sql(&database, &create_path).await?;

        // Rollback the schema
        // let rollback_path = path.join("rollback.sql");
        // tokio::fs::write(&rollback_path, b"").await?;

        log::info!("Schema migration created");
        Ok(true)
    } else {
        Ok(false)
    }
}

fn prompt_table_alter(database: &Database, migrations: &MigrationError) -> Result<String> {
    match migrations {
        MigrationError::MissingTable(table) => {
            if let Some(dbtable) = database.get_table(table) {
                log::info!("Prompting for missing table: `{:?}`", migrations);
                let (choice, _) = prompt_select_with_default(
                    "Alter Column:",
                    &vec!["Create", "Rename", "Skip"],
                    0,
                )?;

                if choice == "Rename" {
                    let tables = database.get_table_names();

                    let (new_table, _) = prompt_select("New Table Name:", &tables)?;

                    let mut alt = AlterQuery::new(AlterMode::RenameTable, table, "");
                    alt.rename(new_table);
                    Ok(dbtable.on_alter(&alt)?)
                } else if choice == "Create" {
                    let alt = AlterQuery::new(AlterMode::AddTable, table, "");
                    Ok(dbtable.on_alter(&alt)?)
                } else {
                    Ok("".to_string())
                }
            } else {
                Err(anyhow::anyhow!(
                    "Table not found (this should never happen): {}",
                    table
                ))
            }
        }
        MigrationError::MissingColumn { table, column } => {
            if let Some(dbcolumn) = database.get_table_column(table, column) {
                log::info!("Prompting for missing column: `{:?}`", migrations);
                // Table exists, only the column is missing
                let (choice, _) = prompt_select_with_default(
                    "Alter Column:",
                    &vec!["Create", "Rename", "Skip"],
                    0,
                )?;

                if choice == "Rename" {
                    let columns = database.get_table_columns(table);

                    let (new_column, _) = prompt_select("New Column Name:", &columns)?;

                    let mut alt = AlterQuery::new(AlterMode::RenameColumn, table, column);
                    alt.rename(new_column);
                    Ok(dbcolumn.on_alter(&alt)?)
                } else if choice == "Create" {
                    let alt = AlterQuery::new(AlterMode::AddColumn, table, column);
                    Ok(dbcolumn.on_alter(&alt)?)
                } else {
                    Ok("".to_string())
                }
            } else {
                Err(anyhow::anyhow!(
                    "Column not found (this should never happen): {}.{}",
                    table,
                    column
                ))
            }
        }
        _ => {
            todo!("Prompt for other types of migrations")
        }
    }
}

pub async fn test_migrations(config: &Config) -> Result<Validator> {
    log::info!("Testing the migrations...");

    let connection = rusqlite::Connection::open_in_memory()?;
    log::info!("Created an in-memory database to test the migrations against");

    let path = config.migrations_src_path()?;
    let migrations: Vec<PathBuf> = config.versions.iter().map(|v| path.join(v)).collect();

    for (index, migration) in migrations.iter().enumerate() {
        let query_path = if index == 0 {
            migration.join("create.sql")
        } else {
            migration.join("upgrade.sql")
        };

        if query_path.exists() {
            let query = tokio::fs::read_to_string(&query_path).await?;

            log::info!("Running migration: {:?}", query_path);
            connection.execute_batch(&query)?;
            log::info!("Migration complete");
        } else {
            log::warn!("Migration does not exist: {:?}", query_path);
        }
    }

    let current_database = Database::find_database(config)?;
    let database = geekorm::Database {
        tables: current_database.tables.clone(),
    };
    let validator = test_database(&connection, &database).await?;

    Ok(validator)
}

async fn test_database<'a, C>(connection: &'a C, database: &geekorm::Database) -> Result<Validator>
where
    C: GeekConnection<Connection = C> + 'a,
{
    let mut tables = Vec::new();
    for table_name in C::table_names(connection).await? {
        let table = C::pragma_info(connection, &table_name).await?;
        tables.push((table_name, table));
    }

    let mut validator = Validator {
        errors: Vec::new(),
        quick: false,
    };

    match geekorm_core::migrations::validate::validate_database(&tables, database, &mut validator) {
        Ok(MigrationState::UpToDate) | Ok(MigrationState::Initialized) => {
            log::info!("Database is up to date");
        }
        Ok(MigrationState::OutOfDate(_)) => {
            log::error!("Database is out of date!");
        }
        Err(err) => {
            log::error!("Error validating the database: {}", err);
            return Err(err.into());
        }
    }

    Ok(validator)
}
