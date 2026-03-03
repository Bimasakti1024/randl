// src/config.rs
use dirs::config_dir;
use std::fs;
use std::path::PathBuf;

pub fn get_config_dir() -> PathBuf {
    let dir = config_dir()
        .expect("Could not find configuration directory.")
        .join("randl");

    if !dir.exists() {
        fs::create_dir_all(&dir).expect("Failed to create config directory");
    }

    dir
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
