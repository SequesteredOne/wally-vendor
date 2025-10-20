use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst).with_context(|| format!("Failed to create directory {:?}", dst))?;

    for entry in WalkDir::new(src).min_depth(1) {
        let entry = entry?;
        let relative_path = entry.path().strip_prefix(src)?;
        let target_path = dst.join(relative_path);

        if entry.file_type().is_dir() {
            fs::create_dir_all(&target_path)
                .with_context(|| format!("Failed to create directory {:?}", target_path))?;
        } else {
            if let Some(parent) = target_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(entry.path(), &target_path).with_context(|| {
                format!("Failed to copy {:?} to {:?}", entry.path(), target_path)
            })?;
        }
    }

    Ok(())
}

pub fn find_wally_package(packages_dir: &Path, package_spec: &str) -> Option<PathBuf> {
    let parts: Vec<&str> = package_spec.split("/").collect();
    if parts.len() != 2 {
        return None;
    }

    let scope = parts[0];
    let name_with_version = parts[1];
    let (name, version) = match name_with_version.split_once("@") {
        Some((name, version)) => (name, Some(version)),
        None => (name_with_version, None),
    };

    let index_dir = packages_dir.join("_Index");
    if !index_dir.exists() {
        return None;
    }

    let search_prefix = format!("{}_{}", scope, name);

    let entries = fs::read_dir(index_dir).ok()?;
    for entry in entries.flatten() {
        let entry_name = entry.file_name();
        let entry_name_str = entry_name.to_string_lossy();

        let matches = if let Some(version) = version {
            let search_pattern = format!("{}@{}", search_prefix, version);
            entry_name_str.starts_with(&search_pattern)
        } else {
            entry_name_str.starts_with(&search_prefix)
        };

        if matches {
            let pkg_dir = entry.path().join(name);
            if pkg_dir.is_dir() {
                return Some(pkg_dir);
            }
        }
    }

    None
}
