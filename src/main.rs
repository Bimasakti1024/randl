// src/main.rs
mod archive;
mod cli;
mod commands;
mod config;
mod download;
mod security;
mod util;

use crate::{
    cli::{Cli, Commands},
    config::get_repos_file,
};
use clap::Parser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    std::panic::set_hook(Box::new(|info| {
        let location = info
            .location()
            .map(|l| format!("{}:{}", l.file(), l.line()))
            .unwrap_or("unknown".to_string());
        let message = info.payload().downcast_ref::<&str>().unwrap_or(&"unknown");
        eprintln!("Oops! randl just crashed at {}: {}", location, message);
        eprintln!("Please report this at https://github.com/Bimasakti1024/randl/issues");
        eprintln!("           :(");
    }));

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
