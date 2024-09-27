use anyhow::Result;
use geekorm::{prelude::*, GEEKORM_BANNER, GEEKORM_VERSION};

#[derive(Debug, Clone, Default, Table, serde::Serialize, serde::Deserialize)]
struct Projects {
    #[geekorm(primary_key, auto_increment)]
    pub id: PrimaryKey<i32>,
    #[geekorm(unique)]
    pub name: String,

    #[geekorm(new = "chrono::Utc::now()")]
    pub published: chrono::DateTime<chrono::Utc>,

    #[geekorm(update = "chrono::Utc::now()")]
    pub updated: chrono::DateTime<chrono::Utc>,
}

#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(debug_assertions)]
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .init();

    println!("{}     v{}\n", GEEKORM_BANNER, GEEKORM_VERSION);

    // Initialize an in-memory database
    let db = libsql::Builder::new_local(":memory:").build().await?;
    let connection = db.connect()?;
    Projects::create_table(&connection).await?;

    let now = chrono::Utc::now();
    let mut gorm = Projects::new("geekorm", now);
    gorm.save(&connection).await?;

    println!("Project :: {:?}", gorm);

    Ok(())
}
