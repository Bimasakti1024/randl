// src/cli.rs
use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(name = "randl", about = "Random Downloader powered by a federated network of static-hosted repositories.")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Pull a random file from repository
    Pull(PullArgs),

    /// Manage repositories
    #[command(alias = "repo")]
    Repository {
        #[command(subcommand)]
        action: RepositoryAction,
    },
}

#[derive(Args, serde::Serialize)]
pub struct PullArgs {
    /// Maximum depth for nested repository
    #[arg(short, long)]
    pub max_depth: Option<u32>,

    /// The output directory the reward will be saved
    #[arg(short, long)]
    pub output_directory: Option<std::path::PathBuf>,

    /// Toggle dry run
    #[arg(short, long, default_value_t = false)]
    pub dry_run: bool,

    /// Toggle confirmation before downloading
    #[arg(short, long, default_value_t = false)]
    pub no_confirm: bool,

    /// Repeat pull
    #[arg(short, long)]
    pub repeat: Option<u16>,

    /// Download timeout
    #[arg(short, long)]
    pub timeout: Option<u64>,

    /// Pull from a repository without adding
    #[arg(short, long)]
    pub from: Option<String>,
}

#[derive(Subcommand)]
pub enum RepositoryAction {
    /// Add a repository
    Add {
        /// Name of the repository
        name: String,

        /// url of the repository
        url: String
    },

    /// Remove a repository
    Remove {
        /// Name of the repository
        name: String,

        /// Remove cache
        #[arg(short, long, default_value_t = false)]
        keep_cache: bool,
    },

    /// List all repositories
    List,

    /// Synchronize a repository (all repository by default)
    Sync { name: Vec<String> },

    /// Check dead repository
    Check,
}
