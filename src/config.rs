// src/config.rs
use dirs::config_dir;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::{self, create_dir_all, read_to_string, write};
use std::path::PathBuf;
use std::time::Duration;
use ureq::Agent;

pub const DEFAULT_CONFIG: &str = r#"[configuration]
max_depth = 3
output_directory = "."
repeat = 1
timeout = 30
no_confirm = false
dry_run = false
keep_cache = false

[repositories]
"#;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub configuration: Configuration,
    pub repositories: HashMap<String, Repository>,
}

#[derive(Deserialize, Debug)]
pub struct Configuration {
    pub max_depth: u32,
    pub output_directory: PathBuf,
    pub repeat: u32,
    pub timeout: u64,
    pub no_confirm: bool,
    pub dry_run: bool,
    pub keep_cache: bool,
}

#[derive(Deserialize, Debug)]
pub struct Repository {
    pub enabled: bool,
    pub url: String,
}

pub fn get_config_dir() -> PathBuf {
    let dir = config_dir()
        .expect("Could not find configuration directory.")
        .join("randl");

    if !dir.exists() {
        create_dir_all(&dir).expect("Failed to create config directory");
        let config_file = dir.join("config.toml");
        write(&config_file, DEFAULT_CONFIG).expect("Failed to write default config");
        eprintln!("Created config file at {}", config_file.display());
    }

    dir
}

pub fn get_config_file() -> PathBuf {
    get_config_dir().join("config.toml")
}

pub fn get_toml_config() -> toml::Value {
    let config_file = get_config_file();
    let content = read_to_string(config_file).expect("Failed to read config file");
    toml::from_str(&content).expect("Failed to parse config")
}

pub fn get_repos_file() -> PathBuf {
    get_config_dir().join("repos.txt")
}

pub fn get_sync_dir() -> PathBuf {
    let dir = get_config_dir().join("sync");

    if !dir.exists() {
        fs::create_dir_all(&dir).expect("Failed to create repository synchronization directory");
    }

    dir
}

pub fn create_agent(t: Option<u64>) -> Agent {
    let timeout = t.unwrap_or_else(|| {
        get_toml_config()
            .as_table()
            .and_then(|doc| doc["configuration"].as_table())
            .and_then(|conf| conf["timeout"].as_integer())
            .unwrap_or(30)
            .try_into()
            .unwrap_or(30)
    });

    Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(timeout)))
        .build()
        .into()
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use crate::config::{Config, DEFAULT_CONFIG, create_agent};

    #[test]
    fn create_agent_with_timeout() {
        create_agent(Some(1));
    }

    #[test]
    fn create_agent_without_timeout() {
        create_agent(None);
    }

    #[test]
    fn test_default_config_parse() {
        let val: toml::Value = toml::from_str(DEFAULT_CONFIG).unwrap();
        assert!(val["configuration"].is_table());
        assert!(val["repositories"].is_table());
    }

    #[test]
    fn test_default_config_deserialize() {
        let config: Config = toml::from_str(DEFAULT_CONFIG).unwrap();
        assert_eq!(config.configuration.repeat, 1);
        assert_eq!(config.configuration.timeout, 30);
        assert_eq!(config.configuration.max_depth, 3);
        assert_eq!(config.configuration.output_directory, PathBuf::from("."));
        assert!(!config.configuration.dry_run);
        assert!(!config.configuration.no_confirm);
        assert!(!config.configuration.keep_cache);
    }
}
