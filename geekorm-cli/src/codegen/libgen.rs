use anyhow::Result;
use quote::{format_ident, quote};

use crate::utils::Config;

pub async fn lib_generation(config: &Config) -> Result<()> {
    let path = config.migrations_path()?;
    let src_dir = if config.crate_mode() {
        log::info!("Generating the lib file...");
        path.join("src")
    } else {
        log::info!("Generating the module file...");
        path.clone()
    };

    let lib_file = if config.mode == "crate" {
        src_dir.join("lib.rs")
    } else {
        path.join("mod.rs")
    };
    log::debug!("Lib File: {}", lib_file.display());

    let mut latest = format_ident!("v0_0_0");
    let mut imports = vec![];

    // Generate the imports for each version
    config.versions.iter().for_each(|v| {
        latest = format_ident!("{}", v);
        imports.push(quote! {
            mod #latest;
        });
    });

    let ast = if !imports.is_empty() {
        quote! {
            //! GeekORM Database Migrations
            #![allow(unused_imports, unused_variables)]
            use geekorm::prelude::*;

            #( #imports )*

            pub use #latest::{Database, Migration as LatestMigration};

            #[doc = "Initializes and automatically migrates database."]
            pub async fn init<'a, T>(connection: &'a T) -> Result<(), geekorm::Error>
            where
                T: geekorm::GeekConnection<Connection = T> + 'a,
            {
                let latest = &LatestMigration;

                match latest.validate_database(connection, &Database).await {
                    Ok(MigrationState::Initialized) => {
                        LatestMigration::create(connection).await?;
                    }
                    Ok(MigrationState::UpToDate) => {}
                    Ok(MigrationState::OutOfDate(_)) => {
                        return Err(geekorm::Error::Unknown);
                    }
                    Err(err) => {
                        return Err(err);
                    }
                }

                Ok(())
            }
        }
    } else {
        quote! {
            //! # GeekORM Database Migrations
            #[allow(unused_imports, unused_variables)]
            use geekorm::prelude::*;
            pub async fn init<'a, T>(connection: &'a T) -> Result<(), geekorm::Error>
            where
                T: geekorm::GeekConnection<Connection = T> + 'a,
            {
                Ok(())
            }
        }
    };

    log::debug!("Writing the lib/mod file...");
    tokio::fs::write(&lib_file, ast.to_string().as_bytes()).await?;

    if config.module_mode() {
        log::warn!("The module must be manually added to our lib.rs/main.rs file.");
    }

    log::debug!("Updated {}", lib_file.display());

    Ok(())
}
