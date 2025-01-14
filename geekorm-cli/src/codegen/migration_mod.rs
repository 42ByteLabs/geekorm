use anyhow::Result;
use quote::{format_ident, quote};
use std::path::PathBuf;

use crate::utils::database::Database;
use crate::utils::Config;

pub async fn create_mod(config: &Config, path: &PathBuf) -> Result<()> {
    log::info!("Creating a mod file...");

    let database = Database::find_database(config)?;
    log::debug!("Database: {:#?}", database);

    let tables = database.tables;

    let doctitle = format!("GeekORM Database Migrations - {}", chrono::Utc::now());
    let version = config.version.to_string();

    let parent = path.parent().ok_or(anyhow::anyhow!("Invalid path"))?;

    let mut previous_block = quote! {};
    let previous = if let Some(pname) = config.previous_version() {
        log::debug!("Previous Version: {}", pname);
        let ident = format_ident!("{}", pname);
        previous_block = quote! {
            fn previous() -> Option<Box<dyn geekorm::Migration>>
            where
                Self: Sized,
            {
                Some(Box::new(previous::Migration))
            }
        };
        quote! {
            use super::#ident as previous;
        }
    } else {
        quote! {}
    };

    let create_query = if parent.join("create.sql").exists() {
        quote! {
            fn create_query() -> &'static str {
                include_str!("create.sql")
            }
        }
    } else {
        quote! {}
    };
    let upgrade_query = if parent.join("upgrade.sql").exists() {
        quote! {
            fn upgrade_query() -> &'static str {
                include_str!("upgrade.sql")
            }
        }
    } else {
        quote! {}
    };
    let rollback_query = if parent.join("rollback.sql").exists() {
        quote! {
            fn rollback_query() -> &'static str {
                include_str!("rollback.sql")
            }
        }
    } else {
        quote! {}
    };

    let ast = quote! {
        #![doc = #doctitle]
        #![allow(unused_variables, non_upper_case_globals)]

        #previous

        pub struct Migration;

        impl geekorm::Migration for Migration {
            fn version(&self) -> &'static str {
                #version
            }

            /// The migrate function is used to apply the migration to the database.
            async fn migrate<'a, C>(connection: &'a C) -> Result<(), geekorm::Error>
            where
                C: geekorm::GeekConnection<Connection = C> + 'a,
            {
                Ok(())
            }

            #create_query
            #upgrade_query
            #rollback_query

            #previous_block

            fn database(&self) -> &geekorm::Database {
                &Database
            }
        }

        // Static Database Tables
        lazy_static::lazy_static! {
            pub static ref Database: Box<geekorm::Database> = Box::new(
                geekorm::Database {
                    tables: Vec::from([
                        #(#tables),*
                    ])
                }
            );
        }

    };

    tokio::fs::write(path, ast.to_string().as_bytes()).await?;

    Ok(())
}
