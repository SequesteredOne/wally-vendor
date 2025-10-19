use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct VendorConfig {
    pub packages: HashMap<String, String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct  WallyManifest {
    #[serde(default)]
    pub dependencies: HashMap<String, String>,
}

pub enum Config {
    Vendor(VendorConfig),
    Wally(WallyManifest),
}

impl Config {
    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config from {:?}", path))?;

        let toml_value: toml::Value = toml::from_str(&content)
            .with_context(|| format!("Failed to parse TOML at {:?}", path))?;

        if toml_value.get("packages").is_some() {
            let config: VendorConfig = toml::from_str(&content)
                .with_context(|| format!("Failed to parse as wally-vendor.toml at {:?}", path))?;

            return Ok(Config::Vendor(config));
        }

        if toml_value.get("dependencies").is_some() {
            let manifest: WallyManifest = toml::from_str(&content)
                .with_context(|| format!("Failed to parse as wally.toml at {:?}", path))?;

            return Ok(Config::Wally(manifest));
        }

        bail!("Could not determine config format for {:?}. Expected either a `[packages]` or `[dependencies]` table", path);
    }
}
