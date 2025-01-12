use crate::migrations;
use crate::utils::{
    prompt_input_with_default, prompt_select, prompt_select_many, prompt_select_with_default,
    Config,
};
use anyhow::Result;

pub async fn init(config: &mut Config) -> Result<()> {
    log::info!("Initializing GeekORM...");
    // TODO: Check if the configuration file already exists

    let (selected, _) = prompt_select("Migration Mode:", &vec!["Crate", "Module"])?;

    match selected {
        "Crate" => {
            config.mode = "crate".to_string();
        }
        "Module" => {
            log::error!("Module mode is not yet supported");
            return Err(anyhow::anyhow!("Module mode is not yet supported"));
        }
        _ => {
            log::error!("Invalid mode selected");
            return Err(anyhow::anyhow!("Invalid mode selected"));
        }
    }

    let name = prompt_input_with_default("Name:", &config.name())?;
    config.name = Some(name);

    let (database, _) = prompt_select("Database:", &vec!["SQLite"])?;
    config.database = database.to_lowercase().to_string();

    config.drivers = prompt_select_many("Drivers:", &vec!["none", "libsql", "rustqlite"])?
        .iter()
        .map(|d| d.to_string())
        .collect();

    if config.mode == "crate" {
        init_crate(config).await?;
    }

    // Create new migration?
    let (new_migration, _) =
        prompt_select_with_default("Create new migration?", &vec!["Yes", "No"], 0)?;

    if new_migration == "Yes" {
        migrations::create_migrations(config).await?;
    }

    Ok(())
}

/// Initialize the crate mode
///
/// - Create the `migrations` directory
/// - Create the `migrations` rust project with the `migrations` directory as the workspace
pub async fn init_crate(config: &Config) -> Result<()> {
    log::info!("Initializing the crate mode...");

    let name = config.name();
    let migrations_dir = config.migrations_path()?;
    // let migrations_src_dir = migrations_dir.join("src");
    log::debug!("Migrations directory: {}", migrations_dir.display());

    // Setup the migrations project
    if !migrations_dir.exists() {
        log::info!("Creating the migrations project...");
        std::fs::create_dir_all(&migrations_dir)?;
        tokio::process::Command::new("cargo")
            .args(&["init", "--name", name.as_str(), "--lib", "--vcs", "none"])
            .current_dir(&migrations_dir)
            .status()
            .await?;
    }

    let mut features = vec!["migrations", "backends"];
    let mut crates = vec!["lazy_static@1"];

    config.drivers.iter().for_each(|d| match d.as_str() {
        "libsql" => {
            crates.push("libsql");
            features.push("libsql");
        }
        "rustqlite" => {
            crates.push("rustqlite");
            features.push("rustqlite")
        }
        _ => {}
    });
    log::debug!("Features: {:?}", features);

    let geekorm_lib: Vec<String> = if let Some(gpath) = &config.geekorm {
        vec!["--path".to_string(), gpath.to_string()]
    } else {
        vec!["geekorm".to_string()]
    };
    log::debug!("GeekORM Library: {:?}", geekorm_lib);

    // Add dependencies
    tokio::process::Command::new("cargo")
        .arg("add")
        .args(geekorm_lib)
        .arg("-F")
        .arg(features.join(","))
        .current_dir(&migrations_dir)
        .status()
        .await?;

    // Dependencies
    for crt in crates {
        log::debug!("Adding dependency: {}", crt);
        tokio::process::Command::new("cargo")
            .arg("add")
            .arg(crt)
            .current_dir(&migrations_dir)
            .status()
            .await?;
    }

    if config.mode == "crate" {
        // Add the migrations directory as a dependency
        tokio::process::Command::new("cargo")
            .args(&["add", "--path", name.as_str()])
            .status()
            .await?;
    }

    Ok(())
}
