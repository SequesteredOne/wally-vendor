use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct DepsConfig {
    pub packages: HashMap<String, String>,
}

impl DepsConfig {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read dependencies config from {:?}", path))?;

        toml::from_str(&content)
            .with_context(|| format!("Failed to parse wally-vendor.toml at {:?}", path))
    }
}
