mod cli;
mod commands;
mod config;
mod lockfile;
mod utils;

use anyhow::Result;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    match cli.command {
<<<<<<< HEAD
        cli::Commands::SyncVendor(args) => commands::sync_vendor::execute(args).await,
=======
        cli::Commands::Sync(args) => commands::sync::execute(args),
>>>>>>> main
    }
}
