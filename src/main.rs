// src/main.rs
mod cli;
mod commands;
mod config;
mod security;

use crate::{
    cli::{Cli, Commands},
    config::get_repos_file,
};
use clap::Parser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if get_repos_file().exists() {
        eprintln!(
            "Warning: repos.txt is deprecated, please refer to the latest documentation for migration assistance."
        );
    }
    let cli: Cli = Cli::parse();

    match cli.command {
        Commands::Pull(args) => commands::pull::run(args),
        Commands::Repository { action } => commands::repository::run(action),
    }
}
