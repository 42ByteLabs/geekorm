use anyhow::Result;
use console::style;
use log::debug;

use crate::utils::Config;
use crate::utils::database::Database;

pub fn display_database(config: &Config) -> Result<()> {
    println!("Displaying the database schema generated by GeekORM...\n");

    let database = Database::find_database(config)?;
    debug!("Database: {:#?}", database);

    for table in database.get_tables() {
        println!(" Table({}) {{", style(table.name.to_string()).green());

        for column in table.columns.clone() {
            if column.skip {
                continue;
            }

            println!(
                "    Column({}, {})",
                style(column.name).blue(),
                style(column.column_type).yellow()
            );
        }

        println!(" }}\n");
    }

    Ok(())
}
