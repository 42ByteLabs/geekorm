use anyhow::Result;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use std::path::PathBuf;

use crate::utils::Config;

pub async fn lib_generation(config: &Config, path: &PathBuf) -> Result<()> {
    log::info!("Generating the lib file...");
    let src_dir = path.join("src");

    let lib_file = if config.mode == "crate" {
        src_dir.join("lib.rs")
    } else {
        src_dir.join("mod.rs")
    };

    let mut latest = format_ident!("v0_0_0");
    let mut imports = vec![];

    // Get a list of dirs that start with "v"
    let mut dirs = tokio::fs::read_dir(&src_dir).await?;

    while let Some(dir) = dirs.next_entry().await? {
        if dir.file_type().await?.is_dir() {
            let name = dir.file_name();
            if let Some(name) = name.to_str() {
                if name.starts_with("v") {
                    // Latest == last version
                    latest = format_ident!("{}", name);

                    let impstmt = quote! {
                        mod #latest;
                    };
                    imports.push(impstmt);
                }
            }
        }
    }

    let drivers = drivers_gen(config)?;

    let ast = quote! {
        //! GeekORM Database Migrations
        #[allow(unused_imports, unused_variables)]
        use geekorm::prelude::*;

        #( #imports )*

        pub use #latest::{Database, Migration as LatestMigration};

        pub async fn init<'a, T>(connection: &'a T) -> Result<(), geekorm::Error>
        where
            T: geekorm::GeekConnection<Connection = T> + 'a,
        {
            let database = &Database;

            match LatestMigration::validate(connection, database).await {
                Ok(MigrationState::Initialized) => {
                    LatestMigration::create(connection, database).await?;
                }
                Ok(MigrationState::OutOfDate(reason)) => {
                    eprintln!("Database is out of date: {}", reason);
                    // LatestMigration::upgrade(connection).await?;
                    Err(geekorm::Error::MigrationError(reason))?;
                }
                Ok(MigrationState::UpToDate) => {}
                Err(err) => {
                    eprintln!("Error validating database: {}", err);
                    return Err(err);
                }
            }

            Ok(())
        }


        pub async fn connect<'a, T>(connection: impl Into<String>) -> Result<T, geekorm::Error>
        where
            T: geekorm::GeekConnection<Connection = T> + 'a,
        {
            let connection = connection.into();

            match connection.as_str() {
                #drivers
            }
        }
    };

    log::debug!("Updating the src/lib.rs file...");

    tokio::fs::write(&lib_file, ast.to_string().as_bytes()).await?;

    tokio::process::Command::new("cargo")
        .arg("fmt")
        .current_dir(&src_dir)
        .status()
        .await?;

    Ok(())
}

fn drivers_gen(config: &Config) -> Result<TokenStream> {
    let mut drivers = quote! {};
    if config.drivers.contains(&"libsql".to_string()) {
        drivers.extend(quote! {
            ":memory:" => {
                let db = libsql::Builder::new_local(":memory:").build().await?;
                Ok(db.connect())
            }
            // libsql database
            path if path.starts_with("libsql:") => {
                let token = self.token.clone().ok_or_else(|| {
                    Error::UnknownError("libsql database requires a token".to_string())
                })?;

                let db = libsql::Builder::new_remote(path.to_string(), token)
                    .build()
                    .await?;
                Ok(db.connect())
            }

        });
    } else if config.drivers.contains(&"rustqlite".to_string()) {
        drivers.extend(quote! {
            ":memory:" => {
                Ok(Connection::open_in_memory()?)
            }
            path if path.starts_with("rusqlite") => {
                let connection = T::connect("sqlite://:memory:").await?;
                Ok(connection)
            }
        });
    }

    Ok(drivers)
}
