//! # Users
//!
//! This is an example of how to use the GeekORM query builder

#![allow(dead_code, unused_imports)]

use anyhow::Result;
use geekorm::{Connection, ConnectionManager, prelude::*};

mod models;
use models::{UserRole, UserSessions, Users};

#[tokio::main]
async fn main() -> Result<()> {
    // Data
    let users = vec![("geekmasher", UserRole::Admin, "ThisIsMySecurePassword")];

    let database = ConnectionManager::in_memory().await?;
    let connection = database.acquire().await;

    println!("Creating database tables...");
    Users::create_table(&connection).await?;
    UserSessions::create_table(&connection).await?;

    println!("Inserting users...");
    for (username, role, user_password) in users {
        // Create a new user
        let user = Users::create(&connection, username, user_password, role).await?;
        println!("Creating user :: {:#?}", user);

        println!("User Session  :: {}", user.session);
    }

    let args: Vec<String> = std::env::args().collect();

    // Auth
    if args.len() == 3 {
        println!("\nRunning Authentication Mode...\n");

        let username = args.get(1).expect("Getting username failed");
        let user_password = args.get(2).expect("Getting passowrd failed");
        match Users::login(&connection, username, user_password).await {
            Ok(user) => {
                println!("Successfully authenticated as `{}` user", user.username);
            }
            Err(err) => {
                eprintln!("Failed to authenticate: {}", err);
            }
        }
    } else {
        println!("\nDisplaying data...\n");
        let total = Users::total(&connection).await?;
        println!("Total Users :: {}", total);
    }

    Ok(())
}
