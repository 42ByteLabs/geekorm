use anyhow::Result;
use std::path::PathBuf;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Cargo {
    pub package: Package,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Package {
    pub name: String,
    pub version: String,
}

impl Cargo {
    pub async fn read(path: &PathBuf) -> Result<Cargo> {
        let data = tokio::fs::read_to_string(path).await?;
        let cargo: Cargo = toml::from_str(&data)?;
        Ok(cargo)
    }
}
