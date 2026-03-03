// src/main.rs
mod cli;
mod commands;
mod config;

use crate::cli::{Cli, Commands};
use clap::Parser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli: Cli = Cli::parse();

    match cli.command {
        Commands::Pull(args) => commands::pull::run(args),
        Commands::Repository { action } => commands::repository::run(action),
    }
}
