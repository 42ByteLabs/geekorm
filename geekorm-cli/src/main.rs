use anyhow::Result;

mod cli;
mod display;
mod init;
mod migrations;
mod utils;

use crate::cli::*;
use crate::utils::{prompt_select, Config};

#[tokio::main]
async fn main() -> Result<()> {
    let arguments = init();

    match arguments.commands {
        Some(ArgumentCommands::Init) => {
            init::init(&arguments.config).await?;
        }
        Some(ArgumentCommands::Migrate) => {
            println!("Migrating the database...");
            let config = Config::load(&arguments.config).await?;
            log::debug!("Config: {:#?}", config);

            migrations::create_migrations(&config).await?;
        }
        Some(ArgumentCommands::Display) => {
            let config = Config::load(&arguments.config).await?;
            display::display_database(&config)?
        }
        None => {
            let options = vec!["Migrate", "Init", "Display"];
            let (selected, _) = prompt_select("Select an option:", &options)?;

            println!("You selected: {}", selected);
            match selected {
                "Init" => {
                    println!("Initializing GeekORM...");
                    init::init(&arguments.config).await?;
                }
                _ => {
                    log::error!("Invalid command");
                }
            }
        }
    }

    Ok(())
}
