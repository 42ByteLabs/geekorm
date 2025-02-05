#![allow(dead_code)]
use anyhow::Result;
use geekorm::prelude::BuilderTable;
use geekorm::Column;
use glob::glob;
use std::collections::HashMap;
use std::path::PathBuf;

use super::Config;

/// This struct represents a database and is based on the `internal`
/// module of the `geekorm_derive` crate.
#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct Database {
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,

    /// The name of the database
    #[serde(skip)]
    pub(crate) name: String,
    /// The tables in the database
    pub(crate) tables: Vec<BuilderTable>,
}

impl Database {
    /// Finds the database file in the target directory
    ///
    /// During the build process, the `geekorm_derive` crate will generate
    /// a database file in the target directory. This function will find
    /// the latest database file based on the creation date.
    pub fn find_database(config: &Config) -> Result<Self> {
        let target_path = "target/*/build/geekorm-derive-*/out/geekorm-*.json";

        let path = config.working_dir.join(target_path);
        let path_str = path.to_str().ok_or_else(|| {
            anyhow::anyhow!("Failed to convert path to string: {:?}", config.working_dir)
        })?;

        // Find the latest database file based on the creation date
        glob(path_str)?
            .filter_map(|entry| entry.ok())
            .fold(None, |acc, entry| {
                log::trace!("Database Entry: {:#?}", entry);
                let database = match Self::load_database(entry) {
                    Ok(database) => database,
                    Err(err) => {
                        log::warn!("Failed to load database: {}", err);
                        return acc;
                    }
                };

                Some(acc.map_or(database.clone(), |ref acc: Database| {
                    if database.updated_at < acc.updated_at {
                        database
                    } else {
                        acc.clone()
                    }
                }))
            })
            .ok_or_else(|| anyhow::anyhow!("Database not found"))
    }

    /// Find the default database
    pub fn find_default_database(config: &Config) -> Result<Self> {
        let databases = Self::find_databases(config)?;
        Ok(databases
            .iter()
            .find(|db| db.name.as_str() == "Database")
            .ok_or_else(|| anyhow::anyhow!("Database not found"))?
            .clone())
    }

    /// Find all the databases in the target directory
    pub fn find_databases(config: &Config) -> Result<Vec<Self>> {
        let database = Self::find_database(config)?;
        let mut databases = HashMap::new();

        for table in database.tables {
            let database = table.database.clone().unwrap_or("Database".to_string());
            log::debug!("Found table: {}.{}", database, table.name);
            databases
                .entry(database.clone())
                .or_insert_with(|| Database {
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                    name: database,
                    tables: Vec::new(),
                })
                .tables
                .push(table.clone());
        }

        log::debug!("Found {} database(s)", databases.len());

        Ok(databases.values().cloned().collect())
    }

    /// Load the database from the file
    pub fn load_database(path: PathBuf) -> Result<Self> {
        let database = std::fs::read_to_string(path)?;
        let mut database: Database = serde_json::from_str(&database)?;

        // Remove skipped columns
        database.tables.iter_mut().for_each(|table| {
            table.columns.columns.retain(|col| !col.skip);
        });

        Ok(database)
    }

    /// Sorts the tables in the database from least to most dependent
    ///
    /// This is a simple sort algorithm that will sort the tables based on
    /// their dependencies. If a table has no dependencies, it will be added
    /// to the list of sorted tables. If a table has dependencies, it will be
    /// added to the list of sorted tables if all of its dependencies are
    /// already in the list of sorted tables.
    pub fn sort_tables(&mut self) {
        let mut tables = Vec::new();
        let mut dependencies = Vec::new();

        while tables.len() < self.tables.len() {
            for table in &self.tables {
                if dependencies.contains(&table.name) {
                    continue;
                }
                let table_deps = table.get_dependencies();
                if table_deps.is_empty() || table_deps.iter().all(|dep| dependencies.contains(dep))
                {
                    tables.push(table.clone());
                    dependencies.push(table.name.to_string());
                }
            }
        }

        debug_assert_eq!(tables.len(), self.tables.len());
        if tables.len() != self.tables.len() {
            log::error!("Failed to sort tables: {:#?}", self.tables);
            log::error!("Counts :: {} != {}", tables.len(), self.tables.len());
        }

        self.tables = tables;
    }

    /// Gets all the tables (default tables only)
    pub fn get_tables(&self) -> Vec<&BuilderTable> {
        self.get_database_tables("Database")
    }

    /// Get the tables for a specific database
    pub fn get_database_tables(&self, database: &str) -> Vec<&BuilderTable> {
        self.tables
            .iter()
            .filter(|table| table.database == Some(database.to_string()))
            .collect()
    }

    pub fn get_table(&self, name: &str) -> Option<&BuilderTable> {
        self.tables.iter().find(|table| table.name == name)
    }

    pub fn get_table_column(&self, table: &str, column: &str) -> Option<&Column> {
        self.get_table(table)
            .unwrap()
            .columns
            .columns
            .iter()
            .find(|col| col.name == column)
    }

    pub fn get_table_names(&self) -> Vec<&str> {
        self.tables.iter().map(|t| t.name.as_str()).collect()
    }

    pub fn get_table_columns(&self, table: &str) -> Vec<&str> {
        self.get_table(table)
            .unwrap()
            .columns
            .columns
            .iter()
            .map(|col| col.name.as_str())
            .collect()
    }
}
