//! # Transaction Example
//!
//! This is an example of how to use the GeekORM query builder

#![allow(dead_code, unused_imports)]
use anyhow::Result;
use geekorm::{
    Connection, ConnectionManager, GEEKORM_BANNER, GEEKORM_VERSION, TransactionConnector,
    prelude::*,
};

use geekorm::prelude::*;

#[derive(Table, Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
struct Projects {
    #[geekorm(primary_key, auto_increment)]
    pub id: PrimaryKey<i32>,
    #[geekorm(unique)]
    pub name: String,

    pub url: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(debug_assertions)]
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    println!("{}     v{}\n", GEEKORM_BANNER, GEEKORM_VERSION);
    // Initialize an in-memory database
    let db = ConnectionManager::in_memory().await?;
    // Create Projects table
    Projects::create_table(&db.acquire().await).await?;

    // Create a transaction connection
    let transactions = db.transations().await;

    // Create 10 projects
    for pname in 1..=10 {
        let mut prj = Projects::new(
            format!("geekorm-{}", pname),
            String::from("https://42bytelabs.com"),
        );
        prj.save(&transactions).await?;
    }

    // Execute the transation
    transactions.execute().await?;

    let total = Projects::total(&db.acquire().await).await?;
    println!("Total Projects :: {}", total);

    Ok(())
}
