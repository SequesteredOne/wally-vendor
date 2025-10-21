use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Default)]
pub struct Lockfile {
    #[serde(default)]
    pub package: Vec<Package>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Package {
    pub name: String,
    pub version: String,
}

impl Lockfile {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read lockfile from {:?}", path))?;
        toml::from_str(&content).with_context(|| format!("Failed to parse TOML at {:?}", path))
    }

    pub fn get_package_versions(&self) -> HashMap<String, String> {
        self.package
            .iter()
            .map(|p| (p.name.clone(), p.version.clone()))
            .collect()
    }
}
