use anyhow::Result;

use geekorm::prelude::*;
use geekorm::PrimaryKeyInteger;

#[derive(GeekTable, Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
struct Users {
    id: PrimaryKeyInteger,
    username: String,

    #[geekorm(foreign_key = "Sessions.id")]
    #[serde(skip)]
    sessions: Vec<Sessions>,
}

#[derive(GeekTable, Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
struct Sessions {
    id: PrimaryKeyInteger,

    #[geekorm(rand, rand_prefix = "session")]
    token: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let conn = init().await?;

    let mut user = match Users::query_first(&conn, Users::select_by_username("geekmasher")).await {
        Ok(user) => user,
        Err(_) => {
            let mut user = Users::new("geekmasher");

            user.execute_insert(&conn).await?;
            user
        }
    };

    // New session
    let mut session = Sessions::new();
    session.execute_insert(&conn).await?;

    // Add session to user
    user.sessions.push(session);
    // user.execute_update_session(&conn).await?;
    user.execute_update(&conn).await?;

    println!("{:?}", user);

    let mut query_user = Users::query_first(&conn, Users::select_by_primary_key(user.id)).await?;
    query_user.fetch_all(&conn).await?;

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
    // let db = libsql::Builder::new_local(":memory:").build().await?;
    let db = libsql::Builder::new_local("/tmp/turso-testing.sqlite")
        .build()
        .await?;
    let conn = db.connect()?;

    // TODO: Make this better
    let tables = tables!();

    tables.create_all(&conn).await?;

    Ok(conn)
}
