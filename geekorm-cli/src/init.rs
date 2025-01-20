use crate::migrations;
use crate::utils::{
    prompt_confirm, prompt_input_with_default, prompt_select, prompt_select_many,
    prompt_select_with_default, Config,
};
use anyhow::Result;

pub async fn init(config: &mut Config) -> Result<()> {
    let (selected, _) = prompt_select_with_default("Migration Mode:", &vec!["Module", "Crate"], 0)?;

    match selected {
        "Crate" => {
            config.mode = "crate".to_string();
        }
        "Module" => {
            config.mode = "module".to_string();
        }
        _ => {
            log::error!("Invalid mode selected");
            return Err(anyhow::anyhow!("Invalid mode selected"));
        }
    }
    log::debug!("Migration Mode: {}", config.mode);

    let name = prompt_input_with_default("Name:", &config.name())?;
    // Add basic validation
    if name.contains("/") || name.contains("\\") {
        log::error!("Invalid name: {}", name);
        return Err(anyhow::anyhow!("Invalid name"));
    }
    config.name = Some(name);
    log::debug!("Name: {}", config.name());
    let path = config.migrations_path()?;
    log::info!("Migration Directory: {}", path.display());

    let (database, _) = prompt_select("Database:", &vec!["SQLite"])?;
    config.database = database.to_lowercase().to_string();
    log::debug!("Database: {}", config.database);

    config.drivers = prompt_select_many("Drivers:", &vec!["none", "libsql", "rustqlite"])?
        .iter()
        .map(|d| d.to_string())
        .collect();

    initalise(config).await?;

    // Create the lib/mod generation
    crate::codegen::lib_generation(config).await?;

    // Create new migration?
    let (new_migration, _) =
        prompt_select_with_default("Create initial migration?", &vec!["Yes", "No"], 0)?;

    if new_migration == "Yes" {
        log::debug!("Creating initial migration...");
        migrations::create_migrations(config).await?;
    }

    Ok(())
}

/// Initialize the crate mode
///
/// - Create the `migrations` directory
/// - Create the `migrations` rust project with the `migrations` directory as the workspace
pub async fn initalise(config: &Config) -> Result<()> {
    log::info!("Initializing the crate mode...");

    let name = config.name();
    let migrations_dir = config.migrations_path()?;
    log::debug!("Migrations directory: {}", migrations_dir.display());

    // Setup the migrations project
    if config.crate_mode() {
        if !migrations_dir.exists() {
            log::info!("Creating the migrations project...");
            std::fs::create_dir_all(&migrations_dir)?;
            tokio::process::Command::new("cargo")
                .args(&["init", "--name", name.as_str(), "--lib", "--vcs", "none"])
                .current_dir(&migrations_dir)
                .status()
                .await?;
        }
    } else if config.module_mode() {
        log::info!("Creating the migrations module...");
        if !migrations_dir.exists() {
            std::fs::create_dir_all(&migrations_dir)?;
        } else {
            log::warn!("The {} module already exists", config.name());
            if !prompt_confirm("Overwrite the module?")? {
                log::info!("The module will not be overwritten");
                return Ok(());
            }
        }
    } else {
        log::error!("Invalid mode");
        return Err(anyhow::anyhow!("Invalid mode"));
    }

    let mut features = vec!["migrations", "backends"];
    let mut crates = vec![];

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
    if config.crate_mode() {
        tokio::process::Command::new("cargo")
            .arg("add")
            .args(geekorm_lib)
            .arg("-F")
            .arg(features.join(","))
            .current_dir(&migrations_dir)
            .status()
            .await?;
    } else if config.module_mode() {
        tokio::process::Command::new("cargo")
            .arg("add")
            .args(geekorm_lib)
            .arg("-F")
            .arg(features.join(","))
            .current_dir(&config.working_dir)
            .status()
            .await?;
    }

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
        log::info!("Adding the migrations project to the workspace...");
        tokio::process::Command::new("cargo")
            .args(&["add", "--path", name.as_str()])
            .status()
            .await?;
    }

    Ok(())
}
