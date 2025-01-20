use anyhow::Result;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::path::PathBuf;

use crate::utils::database::Database;
use crate::utils::Config;

pub async fn create_mod(config: &Config, path: &PathBuf) -> Result<()> {
    log::info!("Creating migration files");

    let database = Database::find_database(config)?;
    log::trace!("Database: {:#?}", database);

    // Create SQL files
    if let Some(parent) = path.parent() {
        let create_path = parent.join("create.sql");
        crate::codegen::generate_create_sql(&database, &create_path).await?;
    }

    let tables = database.tables;

    let doctitle = format!("GeekORM Database Migrations - {}", chrono::Utc::now());
    let version = config.version.to_string();

    let parent = path.parent().ok_or(anyhow::anyhow!("Invalid path"))?;

    let mut imports = TokenStream::new();
    let mut body = TokenStream::new();
    let mut data_migrations = TokenStream::new();

    if let Some(pname) = config.previous_version() {
        log::debug!("Previous Version: {}", pname);
        let ident = format_ident!("{}", pname);
        body.extend(quote! {
            fn previous() -> Option<Box<dyn geekorm::Migration>>
            where
                Self: Sized,
            {
                Some(Box::new(previous::Migration))
            }
        });
        imports.extend(quote! {
            use super::#ident as previous;
        });
    }

    // Data migrations
    if config.data_migrations {
        log::debug!("Data Migrations: true");
        imports.extend(quote! {
            mod data;
        });
        body.extend(quote! {
            #[doc = "Applies migrations to the database."]
            async fn migrate<'a, C>(connection: &'a C) -> Result<(), geekorm::Error>
            where
                C: geekorm::GeekConnection<Connection = C> + 'a,
            {
                data::migrate(connection).await
            }
        });
        // Create data migrations
        let doctitle = format!("Migrations for {}", config.version);
        data_migrations.extend(quote! {
            use super::Migration;

            #[doc = #doctitle]
            pub(super) async fn migrate<'a, C>(connection: &'a C) -> Result<(), geekorm::Error>
            where
                C: geekorm::GeekConnection<Connection = C> + 'a,
            {
                todo!("Migrate the database to version ")
            }
        });
    }

    // Create query
    if parent.join("create.sql").exists() {
        body.extend(quote! {
            fn create_query() -> &'static str {
                include_str!("create.sql")
            }
        });
    }
    // Upgrade query
    if parent.join("upgrade.sql").exists() {
        body.extend(quote! {
            fn upgrade_query() -> &'static str {
                include_str!("upgrade.sql")
            }
        });
    }
    // Rollback query
    if parent.join("rollback.sql").exists() {
        body.extend(quote! {
            fn rollback_query() -> &'static str {
                include_str!("rollback.sql")
            }
        });
    }

    let ast = quote! {
        #![doc = #doctitle]
        #![allow(unused_variables, non_upper_case_globals, missing_docs)]

        #imports

        pub struct Migration;

        impl geekorm::Migration for Migration {
            fn version() -> &'static str {
                #version
            }

            #body

            fn database(&self) -> &geekorm::Database {
                &Database
            }
        }

        // Static Database Tables
        geekorm::lazy_static! {
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

    if config.data_migrations {
        let data_path = config.migrations_data_path()?;
        if !data_path.exists() {
            log::debug!("Creating data migrations: {}", data_path.display());

            tokio::fs::write(data_path, data_migrations.to_string().as_bytes()).await?;
        } else {
            log::warn!("Data migrations already exist: {}", data_path.display());
        }
    }

    Ok(())
}
