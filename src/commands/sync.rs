use crate::cli::{Realm, SyncArgs};
use crate::config::Config;
use crate::lockfile::Lockfile;
use crate::utils;
use anyhow::{Context, Result, bail};
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

pub fn execute(args: SyncArgs) -> Result<()> {
    if let Some(jobs) = args.jobs {
        if jobs > 0 {
            rayon::ThreadPoolBuilder::new()
                .num_threads(jobs)
                .build_global()
                .with_context(|| "Failed to initialize global thread pool")?;
        }
    }

    let start = Instant::now();

    let config_path = find_config_path(&args.deps)?;
    let config = Config::load(&config_path)
        .with_context(|| format!("Failed to load config from {:?}", config_path))?;
    let manifest = config.manifest;

    let mut realms = args.realms.clone();
    if realms.is_empty() {
        if !manifest.dependencies.is_empty() {
            realms.push(Realm::Shared);
        }
        if !manifest.server_dependencies.is_empty() {
            realms.push(Realm::Server);
        }
        if !manifest.dev_dependencies.is_empty() {
            realms.push(Realm::Dev);
        }
        println!("Detected realms to vendor: {:?}", realms);
    }

    let lockfile_path = PathBuf::from("wally.lock");
    if args.locked && !lockfile_path.exists() {
        bail!("--locked flag was specified, but wally.lock was not found.");
    }

    let lockfile = if lockfile_path.exists() {
        Some(Lockfile::load(&lockfile_path)?)
    } else {
        println!(
            "wally.lock not found, proceeding without it. Vendored packages may not be deterministic"
        );
        None
    };
    let package_versions = lockfile.as_ref().map(|l| l.get_package_versions());

    let vendor_base = &args.vendor_dir;

    let shared_dest = args
        .shared_dir
        .clone()
        .or_else(|| config.wally_vendor.shared_dir.clone())
        .unwrap_or_else(|| vendor_base.clone());

    let server_dest = args
        .server_dir
        .clone()
        .or_else(|| config.wally_vendor.server_dir.clone())
        .unwrap_or_else(|| vendor_base.clone());

    let dev_dest = args
        .dev_dir
        .clone()
        .or_else(|| config.wally_vendor.dev_dir.clone())
        .unwrap_or_else(|| vendor_base.clone());

    let packages_root = args.packages_dir.parent().unwrap_or_else(|| Path::new("."));
    let server_packages_dir = packages_root.join("ServerPackages");
    let dev_packages_dir = packages_root.join("DevPackages");

    if args.clean {
        let mut dirs_to_clean = HashSet::new();
        if realms.contains(&Realm::Shared) {
            dirs_to_clean.insert(shared_dest.clone());
        }
        if realms.contains(&Realm::Server) {
            dirs_to_clean.insert(server_dest.clone());
        }
        if realms.contains(&Realm::Dev) {
            dirs_to_clean.insert(dev_dest.clone());
        }

        for dir in dirs_to_clean {
            if dir.exists() {
                fs::remove_dir_all(&dir)
                    .with_context(|| format!("Failed to remove vendor directory {:?}", &dir))?;
            }
        }
    }

    let mut total_vendored = 0;
    let mut all_missing = Vec::new();
    let mut total_dependencies = 0;

    if realms.contains(&Realm::Shared) {
        fs::create_dir_all(&shared_dest)
            .with_context(|| format!("Failed to create vendor directory {:?}", &shared_dest))?;
        total_dependencies += manifest.dependencies.len();
        let (vendored, missing) = vendor_packages(
            &manifest.dependencies,
            &args.packages_dir,
            &shared_dest,
            package_versions.as_ref(),
        )?;
        total_vendored += vendored;
        all_missing.extend(missing);
    }

    if realms.contains(&Realm::Server) {
        fs::create_dir_all(&server_dest)
            .with_context(|| format!("Failed to create vendor directory {:?}", &server_dest))?;
        total_dependencies += manifest.server_dependencies.len();
        let (vendored, missing) = vendor_packages(
            &manifest.server_dependencies,
            &server_packages_dir,
            &server_dest,
            package_versions.as_ref(),
        )?;
        total_vendored += vendored;
        all_missing.extend(missing);
    }

    if realms.contains(&Realm::Dev) {
        fs::create_dir_all(&dev_dest)
            .with_context(|| format!("Failed to create vendor directory {:?}", &dev_dest))?;
        total_dependencies += manifest.dev_dependencies.len();
        let (vendored, missing) = vendor_packages(
            &manifest.dev_dependencies,
            &dev_packages_dir,
            &dev_dest,
            package_versions.as_ref(),
        )?;
        total_vendored += vendored;
        all_missing.extend(missing);
    }

    if total_dependencies == 0 {
        println!("No packages to vendor.");
        return Ok(());
    }

    all_missing.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)));

    let duration = start.elapsed();

    if all_missing.is_empty() {
        println!(
            "Successfully vendored {}/{} packages in {}ms",
            total_vendored,
            total_dependencies,
            duration.as_millis()
        );
    } else {
        eprintln!("Missing {} package(s):", all_missing.len());
        for (alias, spec) in &all_missing {
            eprintln!("    {} ({})", alias, spec);
        }
        eprintln!();

        eprintln!(
            "Vendored {}/{} packages in {}ms",
            total_vendored,
            total_dependencies,
            duration.as_millis()
        );

        eprintln!();
        eprintln!("Hint: Try running `wally install` to fetch the missing dependnecies");

        if args.strict {
            bail!(
                "Strict mode enabled: {} package(s) missing",
                all_missing.len()
            );
        }
    }

    Ok(())
}

fn vendor_packages(
    dependencies: &HashMap<String, String>,
    source_base_dir: &Path,
    destination_dir: &Path,
    package_versions: Option<&HashMap<String, String>>,
) -> Result<(usize, Vec<(String, String)>)> {
    let mut missing_packages = Vec::new();
    let mut copy_operations = HashMap::new(); // Use HashMap to deduplicate by source path

    for (alias, package_spec) in dependencies {
        let package_name = package_spec.split("@").next().unwrap_or(package_spec);

        let final_spec = if let Some(versions) = package_versions {
            versions
                .get(package_name)
                .map(|version| format!("{}@{}", package_name, version))
                .unwrap_or_else(|| package_spec.clone())
        } else {
            package_spec.clone()
        };

        match utils::find_wally_package(source_base_dir, &final_spec) {
            Some(source_path) => {
                copy_operations
                    .entry(source_path)
                    .or_insert_with(Vec::new)
                    .push(alias.clone());
            }
            None => {
                missing_packages.push((alias.clone(), package_spec.clone()));
            }
        }
    }

    let copy_results: Vec<Result<()>> = copy_operations
        .par_iter()
        .map(|(source_path, aliases)| {
            copy_package(source_base_dir, source_path, destination_dir, aliases)?;
            Ok(())
        })
        .collect();

    for result in copy_results {
        if let Err(e) = result {
            eprintln!("A package copy operation failed: {:?}", e);
        }
    }

    let packages_vendored = dependencies.len() - missing_packages.len();

    Ok((packages_vendored, missing_packages))
}

fn find_config_path(cli_path: &Option<PathBuf>) -> Result<PathBuf> {
    if let Some(path) = cli_path {
        if !path.exists() {
            bail!("Specified config file does not exist: {:?}", path);
        }
        return Ok(path.clone());
    }

    let vendor_config = PathBuf::from("wally-vendor.toml");
    if vendor_config.exists() {
        return Ok(vendor_config);
    }

    let wally_config = PathBuf::from("wally.toml");
    if wally_config.exists() {
        return Ok(wally_config);
    }

    bail!(
        "Could not find a config file. Please specify one with `--deps` or create a `wally-vendor.toml` or `wally.toml` file"
    );
}

fn copy_package(
    packages_dir: &Path,
    source_path: &Path,
    vendor_dir: &Path,
    aliases: &[String],
) -> Result<()> {
    let relative_path = source_path
        .strip_prefix(packages_dir)
        .with_context(|| "Failed to create relative path for mirrored package")?;

    let vendor_target = vendor_dir.join(relative_path);

    if !vendor_target.exists() {
        if let Some(parent) = vendor_target.parent() {
            fs::create_dir_all(parent)?;
        }
        utils::copy_dir_recursive(source_path, &vendor_target)?;
    }

    for alias in aliases {
        let redirector_lua = packages_dir.join(format!("{}.lua", alias));
        let redirector_luau = packages_dir.join(format!("{}.luau", alias));

        if redirector_lua.exists() && redirector_lua.is_file() {
            fs::copy(
                &redirector_lua,
                vendor_dir.join(redirector_lua.file_name().unwrap()),
            )?;
        } else if redirector_luau.exists() && redirector_luau.is_file() {
            fs::copy(
                &redirector_luau,
                vendor_dir.join(redirector_luau.file_name().unwrap()),
            )?;
        }
    }

    Ok(())
}
