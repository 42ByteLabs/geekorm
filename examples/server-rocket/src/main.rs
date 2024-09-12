#[macro_use]
extern crate rocket;

use anyhow::Result;
use geekorm::prelude::*;
use rocket::routes;

pub struct AppState {
    db: libsql::Database,
}

#[derive(Table, Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Users {
    #[geekorm(primary_key, auto_increment)]
    pub id: PrimaryKey<i32>,
    #[geekorm(unique)]
    pub username: String,
}

#[get("/?<name>")]
pub async fn index(config: &rocket::State<AppState>, name: Option<String>) -> String {
    if let Some(name) = &name {
        let connection: libsql::Connection = config.db.connect().unwrap();

        println!("Fetching user by username: {}", name);
        let user = match Users::fetch_by_username(&connection, name).await {
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
        .manage(AppState { db })
        .mount("/", routes![index])
        .launch()
        .await?;

    Ok(())
}
