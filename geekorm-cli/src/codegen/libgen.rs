use anyhow::Result;
use quote::{format_ident, quote};

use crate::utils::Config;

pub async fn lib_generation(config: &Config) -> Result<()> {
    let path = config.migrations_path()?;
    log::info!("Generating the lib file...");
    let src_dir = if config.crate_mode() {
        path.join("src")
    } else {
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

    let ast = if !imports.is_empty() {
        quote! {
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
                let latest = &LatestMigration;

                latest.validate_database(connection, database).await?;

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
