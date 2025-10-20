mod cli;
mod commands;
mod config;
mod lockfile;
mod utils;

use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let cli = cli::Cli::parse();

    match cli.command {
        cli::Commands::SyncVendor(args) => commands::sync_vendor::execute(args),
    }
}
