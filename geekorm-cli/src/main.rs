use anyhow::Result;

mod cli;
mod utils;
mod workflows;

use crate::cli::*;
use crate::workflows::*;

fn main() -> Result<()> {
    let arguments = init();

    match arguments.commands {
        Some(ArgumentCommands::Display) => display_database(&arguments)?,
        None => {
            println!("No subcommand selected...");
            println!("Use --help for more information.\n");

            println!("Thank you for trying out / using GeekORM!");
        }
    }

    Ok(())
}
