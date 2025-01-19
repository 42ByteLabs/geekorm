use anyhow::Result;
use std::path::PathBuf;

/// Cargo configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Cargo {
    /// The main package configuration.
    pub package: Package,
    /// The workspace configuration.
    pub workspace: Option<Workspace>,
}

/// Workspace configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Workspace {
    pub members: Vec<String>,
    pub package: Package,
}

/// Package configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Package {
    pub name: Option<String>,
    pub version: Version,
}

/// Version can be a string or a workspace struct (version.workspace = true)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum Version {
    Version(String),
    Workspace { workspace: bool },
}

impl Cargo {
    pub async fn read(path: &PathBuf) -> Result<Cargo> {
        let data = tokio::fs::read_to_string(path).await?;
        let cargo: Cargo = toml::from_str(&data)?;
        Ok(cargo)
    }

    /// Gets the version from the package or the workspace.
    pub fn version(&self) -> Option<String> {
        if let Version::Version(version) = &self.package.version {
            Some(version.to_string())
        } else if let Version::Workspace { workspace } = &self.package.version {
            if *workspace {
                if let Some(workspace) = &self.workspace {
                    match &workspace.package.version {
                        Version::Version(version) => Some(version.clone()),
                        _ => None,
                    }
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        }
    }
}
