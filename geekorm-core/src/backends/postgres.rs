//! # Postgres Backend

use super::GeekConnection;

mod de;

impl GeekConnection for tokio_postgres::Client {
    type Connection = tokio_postgres::Client;

    async fn batch(connection: &Self::Connection, query: crate::Query) -> Result<(), crate::Error> {
        #[cfg(feature = "log")]
        {
            log::debug!("Executing query: {}", query.query);
        }

        connection.batch_execute(&query.query).await?;

        Ok(())
    }

    async fn query<T>(
        connection: &Self::Connection,
        query: crate::Query,
    ) -> Result<Vec<T>, crate::Error>
    where
        T: serde::de::DeserializeOwned,
    {
        #[cfg(feature = "log")]
        {
            log::debug!("Executing query: {}", query.query);
        }

        let parameters: &[&(dyn tokio_postgres::types::ToSql + Sync)] = &query
            .parameters
            .values
            .iter()
            .map(|(_name, value)| value as &(dyn tokio_postgres::types::ToSql + Sync))
            .collect::<Vec<_>>();

        let rows = connection.query(query.query.as_str(), &parameters).await?;

        let mut results: Vec<T> = Vec::new();
        for row in rows {
            results.push(de::from_row::<T>(&row).map_err(|e| {
                #[cfg(feature = "log")]
                {
                    log::error!("Error deserializing row: `{}`", e);
                }
                crate::Error::SerdeError(e.to_string())
            })?);
        }

        Ok(results)
    }

    async fn query_first<T>(
        connection: &Self::Connection,
        query: crate::Query,
    ) -> Result<T, crate::Error>
    where
        T: serde::de::DeserializeOwned,
    {
        Err(crate::Error::NotImplemented)
    }

    async fn execute(
        connection: &Self::Connection,
        query: crate::Query,
    ) -> Result<(), crate::Error> {
        Err(crate::Error::NotImplemented)
    }
}
