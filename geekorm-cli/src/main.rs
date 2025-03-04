#![doc = include_str!("../README.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/42ByteLabs/geekorm/main/assets/geekorm.png"
)]
#![deny(unsafe_code)]
use anyhow::Result;

mod cli;
mod codegen;
mod display;
mod init;
mod migrations;
mod utils;

use crate::cli::*;
use crate::utils::{Config, prompt_select};

#[tokio::main]
async fn main() -> Result<()> {
    let arguments = init();

    let mut config = match Config::load(&arguments.config).await {
        Ok(config) => config,
        Err(err) => {
            log::error!("Failed to load configuration: {}", err);
            return Err(err);
        }
    };

    if let Some(geekorm_path) = &arguments.geekorm_path {
        config.geekorm = Some(geekorm_path.display().to_string());
    }
    log::debug!("Config: {:#?}", config);

    match arguments.commands {
        Some(ArgumentCommands::Init) => {
            init::init(&mut config).await?;
            config.save(&arguments.config)?;
        }
        Some(ArgumentCommands::Migrate { data }) => {
            config.data_migrations = data;
            migrations::create_migrations(&mut config).await?;
        }
        Some(ArgumentCommands::Update) => {
            if config.mode == "crate" {
                init::initalise(&config).await?;
            }

            codegen::regenerate_mods(&config).await?;
            codegen::lib_generation(&config).await?;

            config.code_format().await?;
        }
        Some(ArgumentCommands::Test) => {
            let results = migrations::test_migrations(&config).await?;
            if results.errors.is_empty() {
                log::info!("All migrations passed");
            } else {
                log::error!("The following migrations failed:");
                for error in results.errors {
                    log::error!(" > {}", error);
                }
            }
        }
        Some(ArgumentCommands::Display) => {
            display::display_database(&config)?;
        }
        None => {
            let options = if config.new {
                vec!["Init", "Display"]
            } else {
                vec!["Migrate", "Test/Validate", "Update", "Display"]
            };
            let (selected, _) = prompt_select("Select an option:", &options)?;

            log::info!("You selected: {}", selected);
            match selected {
                "Init" => {
                    log::info!("Initializing GeekORM...");
                    init::init(&mut config).await?;
                    config.save(&arguments.config)?;
                }
                "Migrate" => {
                    log::info!("Migrating the database...");
                    migrations::create_migrations(&mut config).await?;
                }
                "Test/Validate" => {
                    log::info!("Testing the migrations...");
                    if !config.is_initial_version() {
                        migrations::test_migrations(&config).await?;
                    } else {
                        log::info!("No migrations to test");
                    }
                }
                "Update" => {
                    log::info!("Updating the migrations...");
                    if config.mode == "crate" {
                        init::initalise(&config).await?;
                    }

                    codegen::lib_generation(&config).await?;
                }
                "Display" => {
                    log::info!("Displaying the database...");
                    display::display_database(&config)?;
                }
                _ => {
                    log::error!("Invalid command");
                    return Err(anyhow::anyhow!("Invalid command"));
                }
            }
        }
    }

    Ok(())
}
