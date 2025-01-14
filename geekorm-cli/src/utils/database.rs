#![allow(dead_code)]
use anyhow::Result;
use geekorm::prelude::BuilderTable;
use glob::glob;
use std::path::PathBuf;

use super::Config;

/// This struct represents a database and is based on the `internal`
/// module of the `geekorm_derive` crate.
#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct Database {
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub tables: Vec<BuilderTable>,
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
                let database = Self::load_database(entry).ok()?;
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

    /// Load the database from the file
    pub fn load_database(path: PathBuf) -> Result<Self> {
        let database = std::fs::read_to_string(path)?;
        let database: Database = serde_json::from_str(&database)?;
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

        for _passes in 0..=2 {
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
}
