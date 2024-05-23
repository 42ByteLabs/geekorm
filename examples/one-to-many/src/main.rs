use anyhow::Result;

use geekorm::prelude::*;
use geekorm::PrimaryKey;

#[derive(GeekTable, Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
struct Users {
    id: PrimaryKey<i32>,
    username: String,

    #[geekorm(foreign_key = "Sessions.id")]
    #[serde(skip)]
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
    let conn = init().await?;

    let mut user = Users::new("geekmasher");
    user.execute_insert(&conn).await?;

    let session = Sessions::new();

    user.sessions.push(session);
    user.execute_update(&conn).await?;

    println!("{:?}", user);

    let query_user = Users::query_first(&conn, Users::select_by_primary_key(user.id)).await?;
    println!("{:?}", query_user);

    Ok(())
}

async fn init() -> Result<libsql::Connection> {
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

    // Initialize an in-memory database
    let db = libsql::Builder::new_local(":memory:").build().await?;
    // let db = libsql::Builder::new_local("/tmp/turso-testing.sqlite").build().await?;
    let conn = db.connect()?;

    let tables = tables!();
    for table in tables {
        let query = table.create()?;
        conn.execute(query.to_str(), ()).await?;
    }

    Ok(conn)
}
