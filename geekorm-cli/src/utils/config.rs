//! # Utils Configuration
use anyhow::Result;
use std::path::PathBuf;

use crate::utils::cargo::Cargo;

/// Configuration struct
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Config {
    #[serde(skip)]
    pub new: bool,
    #[serde(skip)]
    pub working_dir: PathBuf,

    /// GeekORM mode
    #[serde(skip_serializing_if = "String::is_empty")]
    pub mode: String,
    /// Crate/Module name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) name: Option<String>,

    /// Database
    #[serde(skip_serializing_if = "String::is_empty")]
    pub(crate) database: String,
    /// Database Driver
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) drivers: Vec<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) geekorm: Option<String>,

    #[serde(skip)]
    pub(crate) version: String,
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
            Config::load_file(path).await?
        };

        // Set default working directory
        config.working_dir = path.parent().unwrap().to_path_buf();

        if config.working_dir.join("Cargo.toml").exists() {
            let cargo = Cargo::read(&config.working_dir.join("Cargo.toml")).await?;
            config.version = cargo.package.version;
            log::debug!("Set version to `{}`", config.version);
        } else {
            log::warn!("Cargo.toml not found in working directory");
        }

        Ok(config)
    }

    async fn load_file(path: &PathBuf) -> Result<Self> {
        log::debug!("Loading configuration from `{}`", path.display());
        let data = tokio::fs::read_to_string(path).await?;
        // Based off extension, we can determine the format of the configuration file
        let config: Self = if path
            .extension()
            .map_or(false, |ext| ext == "yml" || ext == "yaml")
        {
            serde_yaml::from_str(&data)?
        } else if path.extension().map_or(false, |ext| ext == "json") {
            serde_json::from_str(&data)?
        } else if path.extension().map_or(false, |ext| ext == "toml") {
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
            .map_or(false, |ext| ext == "yml" || ext == "yaml")
        {
            serde_yaml::to_string(self)?
        } else if path.extension().map_or(false, |ext| ext == "json") {
            serde_json::to_string(self)?
        } else if path.extension().map_or(false, |ext| ext == "toml") {
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

    /// Returns the migrations directory
    pub fn migrations_path(&self) -> Result<PathBuf> {
        if self.crate_mode() {
            Ok(self.working_dir.join(self.name()))
        } else if self.module_mode() {
            todo!("module mode")
        } else {
            Err(anyhow::anyhow!("No mode selected"))
        }
    }

    pub fn new_migration_path(&self) -> Result<PathBuf> {
        if self.crate_mode() {
            let migrations_dir = self.migrations_path()?.join("src");
            // Get the current Cargo package version
            Ok(migrations_dir.join(format!("v{}", self.version())))
        } else {
            Err(anyhow::anyhow!("No mode selected"))
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
            geekorm: None,
            version: "0.1.0".to_string(),
        }
    }
}
