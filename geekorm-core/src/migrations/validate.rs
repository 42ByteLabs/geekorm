use crate::{backends::TableInfo, Database};

use super::MigrationState;

pub(super) fn validate_database(
    database_tables: &super::DatabaseTables,
    migration_database: &Database,
) -> Result<MigrationState, crate::Error> {
    println!("Validating database schema");

    // Validate each table
    for (name, table) in database_tables {
        if let Some(mtable) = migration_database.get_table(name.as_str()) {
            for dbcolumn in table {
                if let Some(mcolumn) = mtable.columns.get(dbcolumn.name.as_str()) {
                    println!("Validating column: {}", dbcolumn.name);

                    match validate_column(name, dbcolumn, mcolumn) {
                        Ok(MigrationState::UpToDate) | Ok(MigrationState::Initialized) => {}
                        Ok(MigrationState::OutOfDate(reason)) => {
                            return Ok(MigrationState::OutOfDate(reason));
                        }
                        Err(err) => {
                            return Err(err);
                        }
                    }
                } else {
                    return Ok(MigrationState::OutOfDate(format!(
                        "Column not found: {}.{}",
                        name, dbcolumn.name
                    )));
                }
            }

            // HACK: This is a little hacky, but we need to validate all columns
            for mcolumn in mtable.columns.columns.iter() {
                if let Some(dbcolumn) = table.iter().find(|c| c.name == mcolumn.name) {
                    match validate_column(name, dbcolumn, mcolumn) {
                        Ok(MigrationState::UpToDate) | Ok(MigrationState::Initialized) => {}
                        Ok(MigrationState::OutOfDate(reason)) => {
                            return Ok(MigrationState::OutOfDate(reason));
                        }
                        Err(err) => {
                            return Err(err);
                        }
                    }
                } else {
                    return Ok(MigrationState::OutOfDate(format!(
                        "Column not found: {}.{}",
                        name, mcolumn.name
                    )));
                }
            }
        } else {
            // If a table is not found, the database is out of date
            return Ok(MigrationState::OutOfDate(format!(
                "Table not found: {}",
                name
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
