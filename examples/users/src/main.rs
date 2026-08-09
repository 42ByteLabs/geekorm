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

    // Create the tables
    Users::create_table(&connection).await?;
    UserSessions::create_table(&connection).await?;

    for (username, role, user_password) in users {
        // Use the auto-generated `::new()` function with the required fields
        let mut user = Users::new(username, user_password);
        println!("Creating user: {}", user.username);

        // You can set struct fields like any other struct
        user.role = role;

        // Save inserts and returns the new record
        // This is why `user` needs to be mutable
        user.save(&connection).await?;
    }

    let total = Users::total(&connection).await?;
    println!("Total Number of Users: {}", total);

    // Use a helper function to quickly fetch by a field value
    let geekmasher = Users::fetch_by_username(&connection, "geekmasher").await?;
    println!("User :: {:?}", geekmasher);

    Ok(())
}
