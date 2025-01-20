use anyhow::Result;
use geekorm::QueryBuilder;
use std::path::PathBuf;

use crate::utils::database::Database;

/// Creates the `create.sql` file for the schema migration
pub async fn generate_create_sql(database: &Database, path: &PathBuf) -> Result<()> {
    log::debug!("Creating `create.sql` file at: {}", path.display());
    let mut data = String::new();
    data += "-- GeekORM Database Migrations\n\n";

    for table in &database.tables {
        let comment = format!("-- {} Table\n", table.name);
        let query = QueryBuilder::create().table(table.clone()).build()?.query;

        data.push_str(&comment);
        data.push_str(&query);
        data += "\n\n";
    }

    tokio::fs::write(path, data.as_bytes()).await?;
    Ok(())
}
