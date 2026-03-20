// src/cli.rs
use clap::{Args, Parser, Subcommand};
use serde::Serialize;
use serde_with::skip_serializing_none;

#[derive(Parser)]
#[command(
    name = "randl",
    about = "Random Downloader powered by a federated network of static-hosted repositories.",
    version
)]
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

#[skip_serializing_none]
#[derive(Args, serde::Serialize)]
pub struct PullArgs {
    /// Maximum depth for nested repository
    #[arg(short, long)]
    pub max_depth: Option<u32>,

    /// The output directory the reward will be saved
    #[arg(short, long)]
    pub output_directory: Option<std::path::PathBuf>,

    /// Toggle dry run
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    pub dry_run: Option<bool>,

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

    /// Scan reward url
    #[arg(short, long, default_value_t = false)]
    pub scan_reward_url: bool,
}

#[derive(Subcommand)]
pub enum RepositoryAction {
    /// Add a repository
    Add {
        /// Name of the repository
        name: String,

        /// url of the repository
        url: String,
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
    Sync {
        /// Names of the repository to sync (targetted)
        name: Vec<String>,

        /// Timeout
        #[arg(short, long)]
        timeout: Option<u64>,
    },

    /// Check dead repository
    Check {
        /// Timeout
        #[arg(short, long)]
        timeout: Option<u64>,
    },
}

#[derive(Serialize)]
pub struct ConfigOverride<T: Serialize> {
    pub configuration: T,
}

#[cfg(test)]
mod test {
    use super::*;
    use clap::Parser;

    #[test]
    fn test_pull_default() {
        let cli = Cli::parse_from(["randl", "pull"]);
        match cli.command {
            Commands::Pull(args) => {
                assert!(!args.dry_run.unwrap());
                assert!(!args.no_confirm);
                assert!(args.repeat.is_none());
            }
            _ => panic!("wrong command"),
        }
    }

    #[test]
    fn test_pull_with_flags() {
        let cli = Cli::parse_from([
            "randl",
            "pull",
            "--dry-run",
            "--no-confirm",
            "--repeat",
            "3",
        ]);
        match cli.command {
            Commands::Pull(args) => {
                assert!(args.dry_run.unwrap());
                assert!(args.no_confirm);
                assert_eq!(args.repeat, Some(3));
            }
            _ => panic!("wrong command"),
        }
    }

    #[test]
    fn test_pull_from_flag() {
        let cli = Cli::parse_from(["randl", "pull", "--from", "https://example.com"]);
        match cli.command {
            Commands::Pull(args) => {
                assert_eq!(args.from, Some("https://example.com".to_string()));
            }
            _ => panic!("wrong command"),
        }
    }

    #[test]
    fn test_repo_add() {
        let cli = Cli::parse_from(["randl", "repo", "add", "repo1", "https://example.com"]);
        match cli.command {
            Commands::Repository { action } => match action {
                RepositoryAction::Add { name, url } => {
                    assert_eq!(name, "repo1");
                    assert_eq!(url, "https://example.com");
                }
                _ => panic!("wrong action"),
            },
            _ => panic!("wrong command"),
        }
    }

    #[test]
    fn test_repo_sync() {
        let cli = Cli::parse_from(["randl", "repo", "sync", "--timeout", "1"]);
        match cli.command {
            Commands::Repository { action } => match action {
                RepositoryAction::Sync { name, timeout } => {
                    assert!(name.is_empty());
                    assert_eq!(timeout, Some(1));
                }
                _ => panic!("wrong action"),
            },
            _ => panic!("wrong command"),
        }
    }

    #[test]
    fn test_repo_sync_no_timeout() {
        let cli = Cli::parse_from(["randl", "repo", "sync"]);
        match cli.command {
            Commands::Repository { action } => match action {
                RepositoryAction::Sync { name, timeout } => {
                    assert_eq!(name.is_empty(), true);
                    assert_eq!(timeout, None);
                }
                _ => panic!("wrong action"),
            },
            _ => panic!("wrong command"),
        }
    }

    #[test]
    fn test_repo_targetted_sync() {
        let cli = Cli::parse_from(["randl", "repo", "sync", "repo1", "repo2", "--timeout", "10"]);
        match cli.command {
            Commands::Repository { action } => match action {
                RepositoryAction::Sync { name, timeout } => {
                    assert_eq!(name[0], "repo1");
                    assert_eq!(name[1], "repo2");
                    assert_eq!(timeout, Some(10));
                }
                _ => panic!("wrong action"),
            },
            _ => panic!("wrong command"),
        }
    }

    #[test]
    fn test_repo_remove_keep_cache() {
        let cli = Cli::parse_from(["randl", "repo", "remove", "test", "--keep-cache"]);
        match cli.command {
            Commands::Repository { action } => match action {
                RepositoryAction::Remove { name, keep_cache } => {
                    assert_eq!(name, "test");
                    assert_eq!(keep_cache, true);
                }
                _ => panic!("wrong action"),
            },
            _ => panic!("wrong command"),
        }
    }

    #[test]
    fn test_repo_remove() {
        let cli = Cli::parse_from(["randl", "repo", "remove", "test"]);
        match cli.command {
            Commands::Repository { action } => match action {
                RepositoryAction::Remove { name, keep_cache } => {
                    assert_eq!(name, "test");
                    assert_eq!(keep_cache, false);
                }
                _ => panic!("wrong action"),
            },
            _ => panic!("wrong command"),
        }
    }

    #[test]
    fn test_repo_check_timeout() {
        let cli = Cli::parse_from(["randl", "repo", "check", "--timeout", "1"]);
        match cli.command {
            Commands::Repository { action } => match action {
                RepositoryAction::Check { timeout } => {
                    assert_eq!(timeout, Some(1));
                }
                _ => panic!("wrong action"),
            },
            _ => panic!("wrong command"),
        }
    }

    #[test]
    fn test_repo_check_no_timeout() {
        let cli = Cli::parse_from(["randl", "repo", "check"]);
        match cli.command {
            Commands::Repository { action } => match action {
                RepositoryAction::Check { timeout } => {
                    assert_eq!(timeout, None);
                }
                _ => panic!("wrong action"),
            },
            _ => panic!("wrong command"),
        }
    }
}
