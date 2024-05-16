#![allow(dead_code)]
use anyhow::Result;
use geekorm::Table;
use glob::glob;
use std::path::PathBuf;

use crate::Arguments;

/// This struct represents a database and is based on the `internal`
/// module of the `geekorm_derive` crate.
#[derive(Debug, Clone, serde::Deserialize)]
pub(crate) struct Database {
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub tables: Vec<Table>,
}

impl Database {
    pub fn find_database(arguments: &Arguments) -> Result<Self> {
        let target_path = "target/*/build/geekorm-derive-*/out/geekorm-*.json";

        let path = arguments.working_dir.join(target_path);
        let path_str = path.to_str().ok_or_else(|| {
            anyhow::anyhow!(
                "Failed to convert path to string: {:?}",
                arguments.working_dir
            )
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

    pub fn load_database(path: PathBuf) -> Result<Self> {
        let database = std::fs::read_to_string(path)?;
        let database: Database = serde_json::from_str(&database)?;
        Ok(database)
    }
}
