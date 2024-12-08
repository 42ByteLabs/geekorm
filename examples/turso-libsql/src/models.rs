//! # Models
use geekorm::prelude::*;

#[derive(Data, Debug, Clone, Default, PartialEq)]
pub enum ProjectType {
    #[default]
    Library,
    Application,
    Framework,
    Tool,
    #[geekorm(key = "Programming Language")]
    Language,
}

/// Repository model
#[derive(Table, Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Repository {
    #[geekorm(primary_key, auto_increment)]
    pub id: PrimaryKeyInteger,
    pub url: String,
}

/// Projects model
#[derive(Debug, Clone, Default, Table, serde::Serialize, serde::Deserialize)]
pub struct Projects {
    #[geekorm(primary_key, auto_increment)]
    pub id: PrimaryKey<i32>,

    #[geekorm(unique)]
    pub name: String,

    #[geekorm(new = "ProjectType::Library")]
    pub project_type: ProjectType,

    #[geekorm(search)]
    pub url: String,

    #[geekorm(foreign_key = "Repository.id")]
    pub repository: ForeignKey<i32, Repository>,
}

/// List of default projects to insert into the database
pub const PROJECTS: [(&str, ProjectType, &str, &str); 7] = [
    (
        "serde",
        ProjectType::Library,
        "https://serde.rs/",
        "https://github.com/serde-rs/serde",
    ),
    (
        "tokio",
        ProjectType::Library,
        "https://tokio.rs/",
        "https://github.com/tokio-rs/tokio",
    ),
    (
        "actix",
        ProjectType::Framework,
        "https://actix.rs/",
        "https://github.com/actix/actix-web",
    ),
    (
        "rocket",
        ProjectType::Framework,
        "https://rocket.rs/",
        "https://github.com/rwf2/Rocket",
    ),
    (
        "reqwest",
        ProjectType::Library,
        "https://docs.rs/reqwest/latest/reqwest/",
        "https://github.com/seanmonstar/reqwest",
    ),
    (
        "hyper",
        ProjectType::Library,
        "https://hyper.rs/",
        "https://github.com/hyperium/hyper",
    ),
    (
        "rust",
        ProjectType::Language,
        "https://rust-lang.org/",
        "https://github.com/rust-lang/rust/",
    ),
];
