//! # Migrations
//!
//! This module contains the migration logic for the database.

mod validate;

use crate::backends::TableInfo;
use crate::builder::models::QueryType;
use crate::{Database, GeekConnection, Query, Table, Values};

/// Migration state
///
/// Represents the state of the database schema
pub enum MigrationState {
    /// The database is initialized but no tables have been created
    Initialized,
    /// The database is up to date
    UpToDate,
    /// The database is out of date
    OutOfDate(String),
}

pub(crate) type DatabaseTables = Vec<(String, Vec<TableInfo>)>;

/// Migration trait
pub trait Migration
where
    Self: Sync + Send,
{
    /// Get the version of the migration
    fn version<'a>(&self) -> &'static str;
    /// Get the create query
    fn create_query() -> &'static str
    where
        Self: Sized;
    /// Get the upgrade query
    fn upgrade_query() -> &'static str
    where
        Self: Sized,
    {
        ""
    }
    /// Get the rollback query
    fn rollback_query() -> &'static str
    where
        Self: Sized,
    {
        ""
    }

    /// Get the previous database if it exists
    fn previous() -> Option<Box<dyn Migration>>
    where
        Self: Sized,
    {
        None
    }

    /// Get the database schema
    fn database(&self) -> &Database;

    /// This function is called to validate the database schema
    /// by comparing the live database to the migration database
    #[allow(async_fn_in_trait)]
    async fn validate_database<'a, C>(
        &self,
        connection: &'a C,
        database: &Database,
    ) -> Result<MigrationState, crate::Error>
    where
        Self: Sized,
        C: GeekConnection<Connection = C> + 'a,
    {
        println!("Validating database schema...");
        // Get all the data from live database
        let database_tables = C::table_names(connection).await?;
        let mut database_table_columns: DatabaseTables = Vec::new();
        for table in database_tables {
            let dbcolumns = C::pragma_info(connection, table.as_str()).await?;
            database_table_columns.push((table, dbcolumns));
        }
        // TODO: Validation
        let mut migrations: Vec<Box<dyn Migration>> = Vec::new();

        Self::validate(&mut migrations, database, &database_table_columns)?;

        for migration in migrations {
            let v = migration.version();
            println!("Running migration: {}", v);
            Self::upgrade(connection).await?;
            // migration.upgrade(connection).await?;
        }

        Ok(MigrationState::UpToDate)
    }

    /// Validate the database schema is correct
    fn validate(
        migrations: &mut Vec<Box<dyn Migration>>,
        migration_database: &Database,
        live_database: &DatabaseTables,
    ) -> Result<MigrationState, crate::Error>
    where
        Self: Sized,
    {
        let result = validate::validate_database(live_database, migration_database)?;

        match result {
            MigrationState::OutOfDate(reason) => {
                println!("Database is out of date: {}", reason);
                if let Some(prev) = Self::previous() {
                    migrations.push(prev);
                }

                Ok(MigrationState::UpToDate)
            }
            _ => Ok(MigrationState::UpToDate),
        }
    }

    /// Create the database if it does not exist
    ///
    /// Assumes the database is already created but the tables are not
    #[allow(async_fn_in_trait)]
    async fn create<'a, C>(connection: &'a C) -> Result<(), crate::Error>
    where
        Self: Sized,
        C: GeekConnection<Connection = C> + 'a,
    {
        let query = Self::create_query().to_string();

        C::batch(
            connection,
            Query::new(
                QueryType::Create,
                query,
                Values::new(),
                Values::new(),
                Vec::new(),
                Table::default(),
            ),
        )
        .await
    }

    /// Migrate the previos database to the current version
    #[allow(async_fn_in_trait)]
    async fn upgrade<'a, C>(connection: &'a C) -> Result<(), crate::Error>
    where
        Self: Sized,
        C: GeekConnection<Connection = C> + 'a,
    {
        let query = Self::upgrade_query().to_string();
        if query.is_empty() {
            #[cfg(feature = "log")]
            {
                log::debug!("No upgrade query found");
            }
            return Ok(());
        }
        #[cfg(feature = "log")]
        {
            log::debug!("Executing upgrade query: {}", query);
        }
        C::execute(
            connection,
            Query::new(
                QueryType::Update,
                query,
                Values::new(),
                Values::new(),
                Vec::new(),
                Table::default(),
            ),
        )
        .await
    }

    /// Downgrade the database to the previous version
    #[allow(async_fn_in_trait)]
    async fn rollback<'a, C>(connection: &'a C) -> Result<(), crate::Error>
    where
        Self: Sized,
        C: GeekConnection<Connection = C> + 'a,
    {
        let query = Self::rollback_query().to_string();
        if query.is_empty() {
            #[cfg(feature = "log")]
            {
                log::debug!("No rollback query found");
            }
            return Ok(());
        }
        #[cfg(feature = "log")]
        {
            log::debug!("Executing rollback query: {}", query);
        }
        C::execute(
            connection,
            Query::new(
                QueryType::Update,
                query,
                Values::new(),
                Values::new(),
                Vec::new(),
                Table::default(),
            ),
        )
        .await
    }

    /// Migrating data from one version to another
    #[allow(async_fn_in_trait, unused_variables)]
    async fn migrate<'a, C>(connection: &'a C) -> Result<(), crate::Error>
    where
        Self: Sized,
        C: GeekConnection<Connection = C> + 'a,
    {
        Ok(())
    }
}
