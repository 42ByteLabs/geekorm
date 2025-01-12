//! # Migrations
//!
//! This module contains the migration logic for the database.

mod validate;

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

/// Migration trait
pub trait Migration {
    /// Get the version of the migration
    fn version() -> &'static str;
    /// Get the create query
    fn create_query() -> &'static str;
    /// Get the upgrade query
    fn upgrade_query() -> &'static str;
    /// Get the rollback query
    fn rollback_query() -> &'static str;

    /// Validate the database schema is correct
    #[allow(async_fn_in_trait)]
    async fn validate<'a, C>(
        connection: &'a C,
        database: &Database,
    ) -> Result<MigrationState, crate::Error>
    where
        C: GeekConnection<Connection = C> + 'a,
    {
        validate::validate_database(connection, database).await
    }

    /// Create the database if it does not exist
    ///
    /// Assumes the database is already created but the tables are not
    #[allow(async_fn_in_trait)]
    async fn create<'a, C>(connection: &'a C, _database: &Database) -> Result<(), crate::Error>
    where
        C: GeekConnection<Connection = C> + 'a,
    {
        // TODO: Create the database if it does not exist
        C::execute(
            connection,
            Query::new(
                QueryType::Update,
                Self::create_query().to_string(),
                Values::new(),
                Values::new(),
                Vec::new(),
                Table::default(),
            ),
        )
        .await
    }

    /// Migrate the database to the latest version
    #[allow(async_fn_in_trait)]
    async fn upgrade<'a, C>(connection: &'a C) -> Result<(), crate::Error>
    where
        C: GeekConnection<Connection = C> + 'a,
    {
        C::execute(
            connection,
            Query::new(
                QueryType::Update,
                Self::upgrade_query().to_string(),
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
        C: GeekConnection<Connection = C> + 'a,
    {
        C::execute(
            connection,
            Query::new(
                QueryType::Update,
                Self::rollback_query().to_string(),
                Values::new(),
                Values::new(),
                Vec::new(),
                Table::default(),
            ),
        )
        .await
    }

    /// Migrating data from one version to another
    #[allow(async_fn_in_trait)]
    async fn migrate<'a, C>(connection: &'a C) -> Result<(), crate::Error>
    where
        C: GeekConnection<Connection = C> + 'a,
    {
        Ok(())
    }
}
