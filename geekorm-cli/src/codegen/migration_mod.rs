use anyhow::Result;
use quote::quote;
use std::path::PathBuf;

use crate::utils::database::Database;
use crate::utils::Config;

pub async fn create_mod(config: &Config, path: &PathBuf, data: bool) -> Result<()> {
    log::info!("Creating a mod file...");

    let database = Database::find_database(config)?;
    log::debug!("Database: {:#?}", database);

    let tables = database.tables;

    let doctitle = format!("GeekORM Database Migrations - {}", chrono::Utc::now());
    let version = config.version.to_string();

    let data = if data {
        quote! {
            async fn migrate<'a, C>(connection: &'a C) -> Result<(), geekorm::Error>
            where
                C: geekorm::GeekConnection<Connection = C> + 'a,
            {
                todo!("Migrate data...");
            }
        }
    } else {
        quote! {}
    };

    let ast = quote! {
        #![doc = #doctitle]

        pub struct Migration;

        impl geekorm::Migration for Migration {
            fn version() -> &'static str {
                #version
            }

            #data

            fn create_query() -> &'static str {
                include_str!("create.sql")
            }
            fn upgrade_query() -> &'static str {
                include_str!("upgrade.sql")
            }
            fn rollback_query() -> &'static str {
                include_str!("rollback.sql")
            }
        }

        // Static Database Tables
        lazy_static::lazy_static! {
            pub static ref Database: geekorm::Database = geekorm::Database {
                tables: Vec::from([
                    #(#tables),*
                ])
            };
        }

    };

    tokio::fs::write(path, ast.to_string().as_bytes()).await?;

    Ok(())
}
