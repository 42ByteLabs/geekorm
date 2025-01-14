use anyhow::Result;

mod cli;
mod codegen;
mod display;
mod init;
mod migrations;
mod utils;

use crate::cli::*;
use crate::utils::{prompt_select, Config};

#[tokio::main]
async fn main() -> Result<()> {
    let arguments = init();

    let mut config = match Config::load(&arguments.config).await {
        Ok(config) => config,
        Err(err) => {
            log::warn!("Failed to load configuration: {}", err);
            Config::default()
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
        Some(ArgumentCommands::Migrate) => {
            migrations::create_migrations(&config).await?;
        }
        Some(ArgumentCommands::Update) => {
            if config.mode == "crate" {
                init::initalise(&config).await?;
            }

            codegen::lib_generation(&config).await?;
        }
        Some(ArgumentCommands::Test) => {
            migrations::test_migrations(&config).await?;
        }
        Some(ArgumentCommands::Display) => {
            display::display_database(&config)?;
        }
        None => {
            let options = if config.new {
                vec!["Init", "Display"]
            } else {
                vec!["Migrate", "Update", "Display"]
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
                    migrations::create_migrations(&config).await?;
                }
                "Update" => {
                    log::info!("Updating the database...");
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
