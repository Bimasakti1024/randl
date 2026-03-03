// src/cli.rs
use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "randl", about = "Random Downloader")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Pull a random file from repository
    Pull(PullArgs),

    /// Manage repositories
    Repository {
        #[command(subcommand)]
        action: RepositoryAction,
    },
}

#[derive(Args)]
pub struct PullArgs {
    /// Maximum depth for nested repository
    #[arg(short, long, default_value_t = 0)]
    pub max_depth: u32,

    /// The output directory the reward will be saved
    #[arg(short, long, default_value = ".")]
    pub output_directory: std::path::PathBuf,

    /// Toggle dry run
    #[arg(short, long, default_value_t = false)]
    pub dry_run: bool,
}

#[derive(Subcommand)]
pub enum RepositoryAction {
    /// Add a repository
    Add { url: String },

    /// Remove a repository
    Remove { url: String },

    /// List all repositories
    List,

    /// Synchronize all repositories
    Sync,
}
