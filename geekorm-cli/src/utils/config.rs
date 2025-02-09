//! # Utils Configuration
use anyhow::Result;
use std::path::PathBuf;

use crate::utils::cargo::Cargo;
use crate::utils::prompt_input;

/// Configuration struct
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Config {
    /// If the configuration is newly created
    #[serde(skip)]
    pub new: bool,
    /// Working directory
    #[serde(skip)]
    pub working_dir: PathBuf,

    /// GeekORM mode
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub mode: String,
    /// Crate/Module name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) name: Option<String>,

    /// Database
    #[serde(skip_serializing_if = "String::is_empty")]
    pub(crate) database: String,
    /// Database Driver
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) drivers: Vec<String>,

    /// Build command (if any)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) build: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) geekorm: Option<String>,

    #[serde(skip)]
    pub(crate) version: String,

    #[serde(skip)]
    pub(crate) versions: Vec<String>,

    /// Data migrations
    #[serde(skip)]
    pub(crate) data_migrations: bool,
}

impl Config {
    /// Load the configuration from the specified path
    pub async fn load(path: &PathBuf) -> Result<Self> {
        let mut config = if path.is_dir() {
            log::warn!("Configuration file is a directory");
            Config::default()
        } else if !path.exists() {
            log::warn!("Configuration file does not exist");
            Config::default()
        } else {
            log::debug!("Loading configuration from `{}`", path.display());
            Config::load_file(path).await?
        };

        // Set default working directory
        config.working_dir = if let Some(parent) = path.parent() {
            parent.to_path_buf()
        } else {
            PathBuf::from(".")
        };
        log::debug!("Working directory: {}", config.working_dir.display());

        let cargo_path = config.working_dir.join("Cargo.toml");
        if cargo_path.exists() {
            log::debug!("Cargo.toml found in working directory");
            let cargo = Cargo::read(&cargo_path).await?;
            let version = if let Some(version) = cargo.version() {
                version
            } else {
                log::warn!("Cargo.toml does not contain a version");
                prompt_input("Enter the version of the crate (e.g. 0.1.0): ")?
            };
            config.version = version;
        } else {
            log::warn!("Cargo.toml not found in working directory");
            let version = prompt_input("Enter the version of the crate (e.g. 0.1.0): ")?;
            config.version = version;
        }

        if config.version.is_empty() {
            log::error!("Version cannot be empty");
            return Err(anyhow::anyhow!("Version cannot be empty"));
        } else {
            log::debug!("Set version to `{}`", config.version);
        }

        config.versions = config.get_versions().await?;
        log::debug!("Versions: {:?}", config.versions);

        Ok(config)
    }

    async fn load_file(path: &PathBuf) -> Result<Self> {
        log::debug!("Loading configuration from `{}`", path.display());
        let data = tokio::fs::read_to_string(path).await?;
        // Based off extension, we can determine the format of the configuration file
        let config: Self = if path
            .extension()
            .is_some_and(|ext| ext == "yml" || ext == "yaml")
        {
            serde_yaml::from_str(&data)?
        } else if path.extension().is_some_and(|ext| ext == "json") {
            serde_json::from_str(&data)?
        } else if path.extension().is_some_and(|ext| ext == "toml") {
            toml::from_str(&data)?
        } else {
            return Err(anyhow::anyhow!("Configuration file is not valid"));
        };
        Ok(config)
    }

    /// Save the configuration to the specified path
    pub fn save(&self, path: &PathBuf) -> Result<()> {
        log::debug!("Saving configuration to `{}`", path.display());
        let data = if path
            .extension()
            .is_some_and(|ext| ext == "yml" || ext == "yaml")
        {
            serde_yaml::to_string(self)?
        } else if path.extension().is_some_and(|ext| ext == "json") {
            serde_json::to_string(self)?
        } else if path.extension().is_some_and(|ext| ext == "toml") {
            toml::to_string(self)?
        } else {
            return Err(anyhow::anyhow!("Configuration file is not valid"));
        };

        std::fs::write(path, data)?;
        Ok(())
    }

    /// Returns if crate mode is selected
    pub fn crate_mode(&self) -> bool {
        self.mode == "crate"
    }
    /// Returns if module mode is selected
    pub fn module_mode(&self) -> bool {
        self.mode == "module"
    }
    pub fn name(&self) -> String {
        self.name.clone().unwrap_or("db".to_string())
    }

    pub fn version(&self) -> String {
        self.version.replace(".", "_")
    }

    /// Build Command for the Rust Project
    pub fn build_command(&self) -> Result<Vec<String>> {
        if self.build.is_empty() {
            Ok(vec![
                "cargo".to_string(),
                "build".to_string(),
                "--workspace".to_string(),
            ])
        } else if let Some(cmd) = self.build.first() {
            if cmd == "cargo" || cmd == "cross" {
                Ok(self.build.clone())
            } else {
                log::error!("Only `cargo` or `cross` commands are supported");
                Err(anyhow::anyhow!("Invalid build command"))
            }
        } else {
            Err(anyhow::anyhow!("No build command specified"))
        }
    }

    /// Run the code format command
    pub async fn code_format(&self) -> Result<()> {
        log::debug!("Running cargo fmt...");
        tokio::process::Command::new("cargo")
            .args(["fmt", "--all"])
            .status()
            .await?;
        Ok(())
    }

    /// Returns the migrations directory
    pub fn migrations_path(&self) -> Result<PathBuf> {
        if self.crate_mode() {
            Ok(self.working_dir.join(self.name()))
        } else if self.module_mode() {
            // TODO: What if the migrations directory is not in the src directory?
            Ok(self.working_dir.join("src").join(self.name()))
        } else {
            Err(anyhow::anyhow!("No mode selected"))
        }
    }

    pub fn migrations_src_path(&self) -> Result<PathBuf> {
        if self.crate_mode() {
            Ok(self.migrations_path()?.join("src"))
        } else if self.module_mode() {
            self.migrations_path()
        } else {
            Err(anyhow::anyhow!("No mode selected"))
        }
    }

    pub fn new_migration_path(&self) -> Result<PathBuf> {
        if self.crate_mode() {
            let migrations_dir = self.migrations_src_path()?;
            // Get the current Cargo package version
            Ok(migrations_dir.join(format!("v{}", self.version())))
        } else if self.module_mode() {
            Ok(self.migrations_path()?.join(format!("v{}", self.version())))
        } else {
            Err(anyhow::anyhow!("No mode selected"))
        }
    }

    pub fn migrations_data_path(&self) -> Result<PathBuf> {
        Ok(self.new_migration_path()?.join("data.rs"))
    }

    pub fn is_initial_version(&self) -> bool {
        self.versions.len() < 2
    }

    async fn get_versions(&self) -> Result<Vec<String>> {
        if self.mode.is_empty() {
            // No mode means we haven't initialised geekorm yet
            return Ok(vec![]);
        }

        let mut results = vec![];
        let src_dir = if self.crate_mode() {
            self.migrations_path()?.join("src")
        } else {
            self.migrations_path()?
        };

        let mut dirs = tokio::fs::read_dir(&src_dir).await?;

        while let Some(dir) = dirs.next_entry().await? {
            if dir.file_type().await?.is_dir() {
                let name = dir.file_name();
                if let Some(name) = name.to_str() {
                    if name.starts_with("v") {
                        results.push(name.to_string());
                    }
                }
            }
        }
        results.sort();
        Ok(results)
    }

    pub fn previous_version(&self) -> Option<String> {
        if self.versions.len() > 1 {
            Some(self.versions[self.versions.len() - 2].clone())
        } else {
            None
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            new: true,
            working_dir: PathBuf::from("."),
            mode: "".to_string(),
            name: None,
            database: "".to_string(),
            drivers: Vec::new(),
            build: Vec::new(),
            geekorm: None,
            version: String::new(),
            versions: Vec::new(),
            data_migrations: false,
        }
    }
}
