use anyhow::Result;
use geekorm::{GEEKORM_BANNER, GEEKORM_VERSION, prelude::*};

#[derive(Table, Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Project {
    #[geekorm(primary_key, auto_increment)]
    pub id: PrimaryKey<i32>,

    #[geekorm(unique)]
    pub name: String,

    pub version: semver::Version,
}

#[tokio::main]
async fn main() -> Result<()> {
    println!("{}     v{}\n", GEEKORM_BANNER, GEEKORM_VERSION);

    let gorm = Project::new("geekorm", semver::Version::parse(GEEKORM_VERSION).unwrap());
    println!("Project :: {:?}", gorm);

    let req = semver::VersionReq::parse(">=0.4.0, <1.0.0").unwrap();

    assert!(req.matches(&gorm.version));

    Ok(())
}
