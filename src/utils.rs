use anyhow::{Context, Result};
use semver::{Version, VersionReq};
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

            if target_path.exists() && should_skip_copy(entry.path(), &target_path)? {
                continue;
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
    let (name, version_req_str) = match name_with_version.split_once("@") {
        Some((name, version)) => (name, Some(version)),
        None => (name_with_version, None),
    };

    let index_dir = packages_dir.join("_Index");
    if !index_dir.exists() {
        return None;
    }

    let search_prefix = format!("{}_{}", scope, name);
    let mut best_match: Option<(Version, PathBuf)> = None;

    let entries = match fs::read_dir(&index_dir) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!(
                "Warning: Could not read Wally packages index at {:?}: {}",
                index_dir, e
            );
            return None;
        }
    };

    for entry in entries.flatten() {
        let entry_name = entry.file_name();
        let entry_name_str = entry_name.to_string_lossy();

        if !entry_name_str.starts_with(&search_prefix) {
            continue;
        }

        let version_part = match entry_name_str.split_once("@") {
            Some((_, part)) => part,
            None => continue,
        };

        if let Ok(installed_version) = Version::parse(version_part) {
            let req_matches = match version_req_str {
                Some(req_str) => VersionReq::parse(req_str)
                    .map(|req| req.matches(&installed_version))
                    .unwrap_or(false),
                None => true,
            };

            if req_matches {
                if let Some((ref best_version, _)) = best_match {
                    if installed_version > *best_version {
                        best_match = Some((installed_version, entry.path()));
                    }
                } else {
                    best_match = Some((installed_version, entry.path()));
                }
            }
        }
    }

    best_match.map(|(_, path)| path)
}

fn should_skip_copy(src: &Path, dst: &Path) -> Result<bool> {
    let src_meta = fs::metadata(src)?;
    let dst_meta = fs::metadata(dst)?;

    if src_meta.len() != dst_meta.len() {
        return Ok(false);
    }

    if let (Ok(src_mod), Ok(dst_mod)) = (src_meta.modified(), dst_meta.modified()) {
        if dst_mod >= src_mod {
            return Ok(true);
        }
    }

    Ok(false)
}
