use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Manifest {
    #[serde(default)]
    pub dependencies: HashMap<String, String>,

    #[serde(default)]
    pub server_dependencies: HashMap<String, String>,

    #[serde(default)]
    pub dev_dependencies: HashMap<String, String>,
}

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct VendorConfig {
    pub shared_dir: Option<PathBuf>,
    pub server_dir: Option<PathBuf>,
    pub dev_dir: Option<PathBuf>,
}

#[derive(Debug, Deserialize, Default, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
    #[serde(flatten)]
    pub manifest: Manifest,

    #[serde(default)]
    pub wally_vendor: VendorConfig,
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config from {:?}", path))?;

        toml::from_str(&content).with_context(|| format!("Failed to parse TOML at {:?}", path))
    }
}
