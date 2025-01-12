use crate::backends::TableInfo;
use crate::{Database, GeekConnection};

use super::MigrationState;

pub(super) async fn validate_database<'a, C>(
    connection: &'a C,
    database: &Database,
) -> Result<MigrationState, crate::Error>
where
    C: GeekConnection<Connection = C> + 'a,
{
    println!("Validating database schema");
    let dbtables = C::table_names(connection).await?;

    // If there are no tables, the database is initialized
    if dbtables.is_empty() {
        return Ok(MigrationState::Initialized);
    }

    for table in dbtables {
        if let Some(mtable) = database.get_table(table.as_str()) {
            let dbcolumns = C::pragma_info(connection, table.as_str()).await?;

            for dbcolumn in dbcolumns {
                if let Some(mcolumn) = mtable.columns.get(dbcolumn.name.as_str()) {
                    match validate_column(&table, &dbcolumn, mcolumn) {
                        Ok(MigrationState::UpToDate) | Ok(MigrationState::Initialized) => {}
                        Ok(MigrationState::OutOfDate(reason)) => {
                            return Ok(MigrationState::OutOfDate(reason));
                        }
                        Err(err) => {
                            return Err(err);
                        }
                    }
                }
            }
        } else {
            // If a table is not found, the database is out of date
            return Ok(MigrationState::OutOfDate(format!(
                "Table not found: {}",
                table
            )));
        }
    }
    Ok(MigrationState::UpToDate)
}

/// Validate a column
///
/// `column` is the source of truth (the migration file)
fn validate_column(
    table: &String,
    dbcolumn: &TableInfo,
    column: &crate::Column,
) -> Result<MigrationState, crate::Error> {
    if dbcolumn.name != column.name {
        return Err(crate::Error::MigrationError(format!(
            "Column name mismatch: {} != {}",
            dbcolumn.name, column.name
        )));
    }

    // Primary key check
    if column.is_primary_key() && dbcolumn.pk != 1 {
        return Ok(MigrationState::OutOfDate(format!(
            "Primary key not set for column: {}",
            column.name
        )));
    }
    // Not null check
    if column.is_not_null() && dbcolumn.notnull == 0 {
        return Ok(MigrationState::OutOfDate(format!(
            "Not null constraint not set for column: {}.{}",
            table, column.name
        )));
    }

    Ok(MigrationState::UpToDate)
}
