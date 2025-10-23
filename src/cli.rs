use clap::{Parser, Subcommand, ValueEnum};
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

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum Realm {
    Shared,
    Server,
    Dev,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(name = "sync")]
    Sync(SyncArgs),
}

#[derive(Parser)]
pub struct SyncArgs {
    /// Path to dependency configuration file
    #[arg(short, long)]
    pub deps: Option<PathBuf>,

    /// The dependency realms to vendor. Can be specified multiple times (ex. --realm Server --realm Dev --realm Shared to include all).
    #[arg(long = "realm", value_enum)]
    pub realms: Vec<Realm>,

    /// Path to Wally packages directory
    #[arg(short, long, default_value = "Packages")]
    pub packages_dir: PathBuf,

    /// Path to vendor output directory
    #[arg(short, long, default_value = "WallyVendor")]
    pub vendor_dir: PathBuf,

    /// Output directory for `shared` dependencies.
    #[arg(long)]
    pub shared_dir: Option<PathBuf>,

    /// Output directory for `server` dependencies.
    #[arg(long)]
    pub server_dir: Option<PathBuf>,

    /// Output directory for `dev` dependencies.
    #[arg(long)]
    pub dev_dir: Option<PathBuf>,

    /// The number of parallel jobs to use for vendoring
    #[arg(short, long)]
    pub jobs: Option<usize>,

    /// Fail if any required dependency is missing
    #[arg(short, long)]
    pub strict: bool,

    /// Remove existing vendor directory before syncing
    #[arg(long)]
    pub clean: bool,

    /// Fail if wally.lock is not found
    #[arg(long)]
    pub locked: bool,
}
