//! # Pagination

use super::pages::Page;
use crate::{GeekConnection, QueryBuilderTrait, TableBuilder};

/// A struct for paginating results
///
/// # Example
///
/// ```no_run
/// # #[cfg(feature = "libsql")] {
/// # use geekorm::prelude::*;
///
/// #[derive(Table, Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
/// pub struct Users {
///     pub id: PrimaryKeyInteger,
///     pub username: String,
///     pub age: i32,
///     pub postcode: Option<String>,
/// }
///
/// pub type UserPage = Pagination<Users>;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let database = libsql::Builder::new_local(":memory:").build().await?;
///     let connection = database.connect().unwrap();
///     
///     // Create a new Page instance
///     let mut page = UserPage::new();
///
///     // Update the page to the next page
///     let results = page.next(&connection).await?;
///     # assert_eq!(page.limit(), 100);
///     # assert_eq!(page.page(), 0);
///
///    # Ok(())
/// }
/// # }
/// ```
pub struct Pagination<T>
where
    T: TableBuilder
        + QueryBuilderTrait
        + serde::Serialize
        + for<'de> serde::Deserialize<'de>
        + Sized,
{
    phantom: std::marker::PhantomData<T>,
    page: Page,
}

impl<T> Pagination<T>
where
    T: TableBuilder
        + QueryBuilderTrait
        + serde::Serialize
        + for<'de> serde::Deserialize<'de>
        + Sized,
{
    /// Create a new Pagination instance
    pub fn new() -> Self {
        Self::default()
    }
    /// Get the current page
    pub fn page(&self) -> u32 {
        self.page.page()
    }
    /// Get the limit
    pub fn limit(&self) -> u32 {
        self.page.limit()
    }
    /// Get the total number of items
    pub fn total(&self) -> u32 {
        self.page.total
    }
    /// Set the total number of items
    pub fn set_total(&mut self, total: u32) {
        self.page.set_total(total);
    }

    /// Get the current page results
    pub async fn get<'a, C>(&mut self, connection: &'a C) -> Result<Vec<T>, crate::Error>
    where
        C: GeekConnection<Connection = C> + 'a,
    {
        // Gets the total number of rows if it hasn't been set
        if self.page.total == 0 {
            self.page
                .set_total(C::row_count(connection, T::query_count().build()?).await? as u32);
        }
        C::query(connection, T::query_select().page(&self.page).build()?).await
    }

    /// Get the next page of results
    pub async fn next<'a, C>(&mut self, connection: &'a C) -> Result<Vec<T>, crate::Error>
    where
        C: GeekConnection<Connection = C> + 'a,
    {
        self.page.next();
        if self.page.max() < self.page.page {
            return Err(crate::Error::PaginationError(
                "Cannot go to next page".to_string(),
            ));
        }
        self.get(connection).await
    }

    /// Get the previous page of results
    pub async fn prev<'a, C>(&mut self, connection: &'a C) -> Result<Vec<T>, crate::Error>
    where
        C: GeekConnection<Connection = C> + 'a,
    {
        if self.page.page == u32::MAX || self.page.page == 0 {
            return Err(crate::Error::PaginationError(
                "Cannot go to previous page".to_string(),
            ));
        }
        self.page.prev();
        self.get(connection).await
    }
}

impl<T> Default for Pagination<T>
where
    T: TableBuilder
        + QueryBuilderTrait
        + serde::Serialize
        + for<'de> serde::Deserialize<'de>
        + Sized,
{
    fn default() -> Self {
        Self {
            phantom: std::marker::PhantomData,
            page: Page {
                page: u32::MAX,
                ..Default::default()
            },
        }
    }
}
