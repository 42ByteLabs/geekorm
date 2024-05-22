use anyhow::Result;

use geekorm::prelude::*;
use geekorm::PrimaryKey;

#[derive(GeekTable, Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
struct Users {
    id: PrimaryKey<i32>,
    username: String,

    #[geekorm(foreign_key = "Sessions.id")]
    sessions: Vec<Sessions>,
}

#[derive(GeekTable, Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
struct Sessions {
    id: PrimaryKey<i32>,

    #[geekorm(rand, rand_prefix = "session")]
    token: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    init();
    // Initialize an in-memory database
    let db = libsql::Builder::new_local(":memory:").build().await?;
    // let db = libsql::Builder::new_local("/tmp/turso-testing.sqlite").build().await?;
    let conn = db.connect()?;

    let tables = tables!();

    let mut user = Users::new("geekmasher");
    let session = Sessions::new();

    user.sessions.push(session);

    println!("{:?}", user);

    Ok(())
}

fn init() {
    println!(
        "{}  - v{}\n",
        geekorm::GEEKORM_BANNER,
        geekorm::GEEKORM_VERSION
    );
    println!("Turso LibSQL Example\n{:=<40}\n", "=");
    let debug_env: bool = std::env::var("DEBUG").is_ok();
    env_logger::builder()
        .filter_level(if debug_env {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Info
        })
        .init();
}
