use crate::cli::SyncVendorArgs;
use crate::config::DepsConfig;
use crate::utils;
use anyhow::{Context, Result, bail};
use std::fs;

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

    let mut packages_vendored = 0;
    let mut missing_packages = Vec::new();

    for (alias, package_spec) in &config.packages {
        match utils::find_wally_package(&args.packages_dir, package_spec) {
            Some(source_path) => {
                let vendor_target = args.vendor_dir.join(alias);

                utils::copy_dir_recursive(&source_path, &vendor_target).with_context(|| {
                    format!("Failed to vendor {} from {:?}", alias, source_path)
                })?;

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
