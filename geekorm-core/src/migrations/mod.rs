//! # Migrations
//!
//! This module contains the migration logic for the database.

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
    OutOfDate,
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
    async fn validate<'a, C>(connection: &'a C) -> Result<MigrationState, crate::Error>
    where
        C: GeekConnection<Connection = C> + 'a,
    {
        Ok(MigrationState::Initialized)
    }

    /// Create the database if it does not exist
    ///
    /// Assumes the database is already created but the tables are not
    #[allow(async_fn_in_trait)]
    async fn create<'a, C>(connection: &'a C, database: &Database) -> Result<(), crate::Error>
    where
        C: GeekConnection<Connection = C> + 'a,
    {
        let query_str = Self::create_query().to_string();
        let query = Query::new(
            QueryType::Create,
            query_str,
            Values::new(),
            Values::new(),
            Vec::new(),
            Table::default(),
        );
        C::execute(connection, query).await
    }

    /// Migrate the database to the latest version
    #[allow(async_fn_in_trait)]
    async fn upgrade<'a, C>(connection: &'a C) -> Result<(), crate::Error>
    where
        C: GeekConnection<Connection = C> + 'a,
    {
        todo!("Migrate database");
    }

    /// Downgrade the database to the previous version
    #[allow(async_fn_in_trait)]
    async fn rollback<'a, C>(connection: &'a C) -> Result<(), crate::Error>
    where
        C: GeekConnection<Connection = C> + 'a,
    {
        todo!("Downgrade database");
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
