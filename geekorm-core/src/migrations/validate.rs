//! # Validate
use crate::error::MigrationError;
use crate::{Database, backends::TableInfo};

use super::MigrationState;

/// Migration validator
#[derive(Debug)]
pub struct Validator {
    /// Errors
    pub errors: Vec<MigrationError>,
    /// Quick validation
    pub quick: bool,
}

/// Validate the database schema
///
/// Errors are returned if an issue has occured, not if the database is out of date
pub fn validate_database(
    database_tables: &super::DatabaseTables,
    migration_database: &Database,
    validator: &mut Validator,
) -> Result<MigrationState, crate::Error> {
    let mut state = MigrationState::UpToDate;
    #[cfg(feature = "log")]
    {
        log::debug!("Validating database schema");
    }

    // Simple checks first
    if database_tables.is_empty() || migration_database.tables.is_empty() {
        return Ok(MigrationState::Initialized);
    }
    // // Mismatched table count
    if database_tables.len() != migration_database.tables.len() {
        #[cfg(feature = "log")]
        {
            log::info!("Database Tables :: {:?}", database_tables);
            log::info!(
                "Migration Tables :: {:?}",
                migration_database.get_table_names()
            );
        }
        state = MigrationState::OutOfDate("Table count mismatch".to_string());
        if validator.quick {
            return Ok(state);
        }
    }

    // Validate each table
    for (name, table) in database_tables {
        if let Some(mtable) = migration_database.get_table(name.as_str()) {
            // Mismatched column count
            if mtable.columns.len() != table.len() {
                state = MigrationState::OutOfDate("Column count mismatch".to_string());
            }

            for dbcolumn in table {
                #[cfg(feature = "log")]
                {
                    log::debug!("Columns :: {:?}", dbcolumn);
                }
                if let Some(mcolumn) = mtable.columns.get(dbcolumn.name.as_str()) {
                    match validate_column(name, dbcolumn, mcolumn, &mut validator.errors) {
                        MigrationState::UpToDate | MigrationState::Initialized => {}
                        MigrationState::OutOfDate(reason) => {
                            state = MigrationState::OutOfDate(reason);
                            if validator.quick {
                                return Ok(state);
                            }
                        }
                    }
                } else {
                    validator.errors.push(MigrationError::MissingColumn {
                        table: name.to_string(),
                        column: dbcolumn.name.to_string(),
                    });

                    state = MigrationState::OutOfDate(format!(
                        "Column not found: {}.{}",
                        name, dbcolumn.name
                    ));
                    if validator.quick {
                        return Ok(state);
                    }
                }
            }

            // HACK: This is a little hacky, but we need to validate all columns
            for mcolumn in mtable.columns.columns.iter() {
                #[cfg(feature = "log")]
                {
                    log::debug!("Migration Columns :: {:?}", mcolumn);
                }
                if let Some(dbcolumn) = table.iter().find(|c| c.name == mcolumn.name) {
                    match validate_column(name, dbcolumn, mcolumn, &mut validator.errors) {
                        MigrationState::UpToDate | MigrationState::Initialized => {}
                        MigrationState::OutOfDate(reason) => {
                            state = MigrationState::OutOfDate(reason);
                            if validator.quick {
                                return Ok(state);
                            }
                        }
                    }
                } else {
                    validator.errors.push(MigrationError::MissingColumn {
                        table: name.to_string(),
                        column: mcolumn.name.to_string(),
                    });
                    state = MigrationState::OutOfDate(format!(
                        "Column not found: {}.{}",
                        name, mcolumn.name
                    ));
                    if validator.quick {
                        return Ok(state);
                    }
                }
            }
        } else {
            validator
                .errors
                .push(MigrationError::MissingTable(name.to_string()));
            // If a table is not found, the database is out of date
            state = MigrationState::OutOfDate(format!("Table not found: {}", name));
            if validator.quick {
                return Ok(state);
            }
        }
    }
    Ok(state)
}

/// Validate a column
///
/// `column` is the source of truth (the migration file)
fn validate_column(
    table: &String,
    dbcolumn: &TableInfo,
    column: &crate::Column,
    errors: &mut Vec<MigrationError>,
) -> MigrationState {
    let mut state = MigrationState::UpToDate;

    // Primary key check
    if column.is_primary_key() && dbcolumn.pk != 1 {
        errors.push(MigrationError::ColumnTypeMismatch {
            table: table.to_string(),
            column: column.name.clone(),
            feature: "primary-key".to_string(),
        });

        state = MigrationState::OutOfDate("Primary key constraint not set".to_string());
    }
    // Not null check
    if column.is_not_null() && dbcolumn.notnull == 0 {
        errors.push(MigrationError::ColumnTypeMismatch {
            table: table.to_string(),
            column: column.name.clone(),
            feature: "not-null".to_string(),
        });
    }

    state
}
