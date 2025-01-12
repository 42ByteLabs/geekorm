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
                init::init_crate(&config).await?;
            }

            codegen::lib_generation(&config, &config.migrations_path()?).await?;
        }
        Some(ArgumentCommands::Display) => {
            display::display_database(&config)?;
        }
        None => {
            let options = if config.new {
                vec!["Init", "Display"]
            } else {
                vec!["Migrate", "Display"]
            };
            let (selected, _) = prompt_select("Select an option:", &options)?;

            println!("You selected: {}", selected);
            match selected {
                "Init" => {
                    println!("Initializing GeekORM...");
                    init::init(&mut config).await?;
                    config.save(&arguments.config)?;
                }
                "Migrate" => {
                    println!("Migrating the database...");
                    migrations::create_migrations(&config).await?;
                }
                _ => {
                    log::error!("Invalid command");
                }
            }
        }
    }

    Ok(())
}
