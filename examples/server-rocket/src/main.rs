#[macro_use]
extern crate rocket;

use anyhow::Result;
use geekorm::prelude::*;
use rocket::{State, routes};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct AppState {
    connection: Arc<Mutex<libsql::Connection>>,
}

#[derive(Table, Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Users {
    #[geekorm(primary_key, auto_increment)]
    pub id: PrimaryKey<i32>,
    #[geekorm(unique)]
    pub username: String,
}

#[get("/?<name>")]
pub async fn index(config: &State<AppState>, name: Option<String>) -> String {
    if let Some(name) = &name {
        println!("Fetching user by username: {}", name);

        let user = match Users::fetch_by_username(&config.connection, name).await {
            Ok(user) => user,
            Err(err) => {
                println!("Error fetching user: {:?}", err);
                return format!("User not found: {}", name);
            }
        };

        return format!("Hello, {}!", user.username);
    }

    format!("Hello, world!")
}

#[rocket::main]
async fn main() -> Result<()> {
    let db = libsql::Builder::new_local(":memory:").build().await?;
    let connection = db.connect()?;

    println!("Creating table Users...");
    Users::create_table(&connection).await?;

    println!("Inserting users...");
    let users = vec!["Alice", "Bob", "Charlie", "David", "Eve"];
    for name in users.iter() {
        let mut user = Users::new(name.to_string());
        user.save(&connection).await?;
        println!("Inserted user: {:?}", user);
    }
    println!("Users inserted: {}", Users::total(&connection).await?);

    // Skip launching the server if running in CI
    if std::env::var("CI").is_ok() {
        return Ok(());
    }

    println!("Starting Rocket server...");
    rocket::build()
        .manage(AppState {
            connection: Arc::new(Mutex::new(connection)),
        })
        .mount("/", routes![index])
        .launch()
        .await?;

    Ok(())
}
