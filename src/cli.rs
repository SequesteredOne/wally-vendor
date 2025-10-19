use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "wally-vendor",
    version,
    about = "Vendor wally packages",
    long_about = None
)]

pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(name = "sync-vendor")]
    SyncVendor(SyncVendorArgs),
}

#[derive(Parser)]
pub struct SyncVendorArgs {
    /// Path to dependency configuration file
    #[arg(short, long, default_value = "wally-vendor.toml")]
    pub deps: PathBuf,

    /// Path to Wally packages directory
    #[arg(short, long, default_value = "Packages")]
    pub packages_dir: PathBuf,

    /// Path to vendor output directory
    #[arg(short, long, default_value = "WallyVendor")]
    pub vendor_dir: PathBuf,

    /// Fail if any required dependency is missing
    #[arg(short, long)]
    pub strict: bool,

    /// Remove existing vendor directory before syncing
    #[arg(long)]
    pub clean: bool,
}
