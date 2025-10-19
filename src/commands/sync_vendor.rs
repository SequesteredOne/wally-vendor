use crate::cli::SyncVendorArgs;
use crate::config::DepsConfig;
use crate::utils;
use anyhow::{Context, Result, bail};
use std::fs;
use std::path::Path;

pub fn execute(args: SyncVendorArgs) -> Result<()> {
    let config = DepsConfig::load(&args.deps)?;

    if config.packages.is_empty() {
        println!("No packages defined in wally-vendor.toml");
        return Ok(());
    }

    if args.clean && args.vendor_dir.exists() {
        fs::remove_dir_all(&args.vendor_dir)
            .with_context(|| format!("Failed to remove vendor directory {:?}", args.vendor_dir))?;
    }

    fs::create_dir_all(&args.vendor_dir)
        .with_context(|| format!("Failed to create vendor directory {:?}", args.vendor_dir))?;

    let mut packages_vendored = 0;
    let mut missing_packages = Vec::new();

    for (alias, package_spec) in &config.packages {
        match utils::find_wally_package(&args.packages_dir, package_spec) {
            Some(source_path) => {
                if args.mirror {
                    copy_mirrored(&args.packages_dir, &source_path, &args.vendor_dir, alias)?;
                } else {
                    copy_flattened(&source_path, &args.vendor_dir, alias)?;
                }

                packages_vendored += 1;
            }
            None => {
                missing_packages.push((alias.clone(), package_spec.clone()));
            }
        }
    }

    println!(
        "Successfully vendored {}/{} packages",
        packages_vendored,
        config.packages.len()
    );

    if !missing_packages.is_empty() {
        eprintln!("Missing {} package(s):", missing_packages.len());
        for (alias, spec) in &missing_packages {
            eprintln!("    {} ({})", alias, spec);
        }
        eprintln!();
        eprintln!("Hint: Try running `wally install` to fetch the missing dependnecies");

        if args.strict {
            bail!(
                "Strict mode enabled: {} package(s) missing",
                missing_packages.len()
            );
        }
    }

    Ok(())
}

fn copy_flattened(source_path: &Path, vendor_dir: &Path, alias: &str) -> Result<()> {
    let vendor_target = vendor_dir.join(alias);

    utils::copy_dir_recursive(source_path, &vendor_target)
        .with_context(|| format!("Failed to vendor {} from {:?}", alias, source_path))
}

fn copy_mirrored(
    packages_dir: &Path,
    source_path: &Path,
    vendor_dir: &Path,
    alias: &str,
) -> Result<()> {
    let relative_path = source_path
        .strip_prefix(packages_dir)
        .with_context(|| "Failed to create relative path for mirrored package")?;

    let vendor_target = vendor_dir.join(relative_path);
    if let Some(parent) = vendor_target.parent() {
        fs::create_dir_all(parent)?;
    }

    utils::copy_dir_recursive(source_path, &vendor_target)?;

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

    Ok(())
}
