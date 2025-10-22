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
        cli::Commands::Sync(args) => commands::sync::execute(args),
    }
}
