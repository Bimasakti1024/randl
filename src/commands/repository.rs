// src/commands/repository.rs
use crate::cli::RepositoryAction;
use crate::config::{get_config_file, get_sync_dir, get_toml_config};
use crate::security::get_file_hash;
use crate::util::create_agent;
use std::fs::{read_to_string, remove_file, write};
use std::time::SystemTime;
use ureq::Agent;

pub fn run(action: RepositoryAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        RepositoryAction::Add { name, url } => add(name, url),
        RepositoryAction::Remove { name, keep_cache } => remove(name, keep_cache),
        RepositoryAction::List => list(),
        RepositoryAction::Sync { name, timeout } => sync(name, timeout),
        RepositoryAction::Check { timeout } => check(timeout),
        RepositoryAction::Enable { name } => enable_repository(name),
        RepositoryAction::Disable { name } => disable_repository(name),
    }
}

pub enum RepositoryType {
    Reward,
    Nested,
    Archive,
    Unknown,
}

pub struct Repository {
    pub repo_type: RepositoryType,
    pub url: Option<String>,
}

pub fn parse_repository(entry: String) -> Repository {
    match entry.splitn(2, ' ').collect::<Vec<_>>().as_slice() {
        [url] => Repository {
            repo_type: RepositoryType::Reward,
            url: Some(url.to_string()),
        },
        ["Nested", url] => Repository {
            repo_type: RepositoryType::Nested,
            url: Some(url.to_string()),
        },
        ["Archive", url] => Repository {
            repo_type: RepositoryType::Archive,
            url: Some(url.to_string()),
        },
        _ => Repository {
            repo_type: RepositoryType::Unknown,
            url: None,
        },
    }
}

/*
   The function handler for the add subcommand,
   Parameter:
       - name: name of the repository
       - url: url of the repository

   It will first get the configuration toml
   and then add the url as a repository under the
   received name with it enabled by default.
*/
fn add(name: String, url: String) -> Result<(), Box<dyn std::error::Error>> {
    let mut doc: toml::Value = get_toml_config();

    let mut repo = toml::map::Map::new();
    repo.insert("url".to_string(), toml::Value::String(url));
    repo.insert("enabled".to_string(), toml::Value::Boolean(true));

    doc["repositories"]
        .as_table_mut()
        .unwrap()
        .insert(name, toml::Value::Table(repo));

    write(get_config_file(), toml::to_string(&doc)?)?;
    Ok(())
}

/*
    function handler for remove subcommand
    parameters:
        - name: the name of the repository

    It will parse the configuration first, then
    it will remove the selected repository and
    will write it to the configuration file again.
    After writing it to the configuration file, it will
    remove the repository cache (at the sync/ directory).
*/
fn remove(name: String, keep_cache: bool) -> Result<(), Box<dyn std::error::Error>> {
    let mut doc: toml::Value = get_toml_config();

    /*
        if keep_cache does not provided
        it will use the configuration as a fallback
    */
    let keep_cache = if keep_cache {
        true
    } else {
        doc["configuration"]
            .as_table()
            .and_then(|t| t["keep_cache"].as_bool())
            .unwrap_or(false)
    };

    if doc["repositories"].as_table().unwrap().contains_key(&name) {
        doc["repositories"].as_table_mut().unwrap().remove(&name);

        if !keep_cache {
            let cache_file = get_sync_dir().join(&name);
            if cache_file.exists() {
                remove_file(&cache_file)?;
            }
        }
    } else {
        eprintln!("Repository {} does not exist.", name);
    }

    write(get_config_file(), toml::to_string(&doc)?)?;
    Ok(())
}

/*
    function handler for subcommand list
    It will read the repositories and then print
    it out
*/
fn list() -> Result<(), Box<dyn std::error::Error>> {
    let doc: toml::Value = get_toml_config();

    for (name, val) in doc["repositories"].as_table().unwrap() {
        println!("{}:", name);
        println!("  url: {}", val["url"].as_str().unwrap());
        println!("  enabled: {}", val["enabled"])
    }

    Ok(())
}

/*
    A function to handle synchronization for a single
    repository.
    parameters:
        - name: the name of the repository
        - url: the url of the repository
    it will download the repository content and save it
    to the synchronization directory (sync/ directory in
    the config directory) under the name of the repository.
*/
fn sync_repo(name: String, url: String, agent: Agent) -> Result<(), Box<dyn std::error::Error>> {
    println!("Syncing {}...", name);
    let content = match agent.get(&url).call() {
        Ok(r) => match r.into_body().read_to_string() {
            Ok(text) => text,
            Err(e) => {
                eprintln!(" Failed to read response from {}: {}.", url, e);
                return Err("Failed to read response".into());
            }
        },
        Err(e) => {
            eprintln!(" Failed to fetch {}: {}.", url, e);
            return Err("Failed to fetch".into());
        }
    };
    write(get_sync_dir().join(name), content)?;
    Ok(())
}

/*
    function handler for subcommand sync
    parameters:
        - names: an optional argument for targetted synchronization
    It will iterate all repository (or only selected one) and will
    synchronize it using the sync_repo(name, url) function
*/
fn sync(names: Vec<String>, timeout: Option<u64>) -> Result<(), Box<dyn std::error::Error>> {
    let mut success = 0;
    let mut error = 0;
    let config = get_toml_config();
    let repos = config["repositories"].as_table().unwrap();
    let agent: Agent = create_agent(timeout);

    for (name, val) in repos {
        if !val["enabled"].as_bool().unwrap() {
            continue;
        }
        // if names is provided, only sync those
        if !names.is_empty() && !names.contains(name) {
            continue;
        }
        let url = val["url"].as_str().unwrap().to_string();
        match sync_repo(name.clone(), url, agent.clone()) {
            Ok(_) => success += 1,
            Err(_) => error += 1,
        }
    }
    println!("{} repository synced and {} failed.", success, error);
    Ok(())
}

/*
    function handler for subcommand check
    parameters:
        - timeout: timeout to check in seconds
    it will iterate all repositories and then check
    if it is alive or dead.
    It also will show status, last sync time, entry data,
    hash (sha-256) and errors
*/
fn check(timeout: Option<u64>) -> Result<(), Box<dyn std::error::Error>> {
    let doc: toml::Value = get_toml_config();
    let mut alive = 0;
    let mut dead = 0;
    let mut greward = 0;
    let mut gnested = 0;
    let mut garchive = 0;
    let mut gunknown = 0;
    let mut gentry = 0;
    let mut genabled = 0;
    let mut gdisabled = 0;
    let agent = create_agent(timeout);

    for (name, data) in doc["repositories"].as_table().unwrap() {
        println!("Checking: {}", name);
        if !data["enabled"].as_bool().unwrap() {
            println!("  Status: disabled");
            gdisabled += 1;
        } else {
            println!("  Status: enabled");
            genabled += 1;
        }

        let url = data["url"].as_str().unwrap().to_string();
        match agent.get(&url).call() {
            Ok(_) => {
                println!("  Alive");
                alive += 1;
            }
            Err(e) => {
                eprintln!("  Dead: {}", e);
                dead += 1;
            }
        }

        match std::fs::metadata(get_sync_dir().join(name)) {
            Ok(metadata) => {
                let modified = metadata.modified()?;
                let elapsed = SystemTime::now().duration_since(modified)?.as_secs();

                println!(
                    "  Last synced: {} ago",
                    humantime::format_duration(std::time::Duration::from_secs(elapsed))
                );
            }
            Err(_) => {
                println!("  Last synced: Never synced");
            }
        }
        let content = read_to_string(get_sync_dir().join(name))?;
        let entries: Vec<&str> = content
            .lines()
            .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
            .collect();

        let mut reward = 0;
        let mut nested = 0;
        let mut archive = 0;
        let mut unknown = 0;
        for entry in &entries {
            let repo_entry = parse_repository(entry.to_string());
            match repo_entry.repo_type {
                RepositoryType::Reward => {
                    reward += 1;
                }
                RepositoryType::Nested => {
                    nested += 1;
                }
                RepositoryType::Archive => {
                    archive += 1;
                }
                RepositoryType::Unknown => {
                    unknown += 1;
                }
            }
        }
        println!("  Entry: {}", entries.len());
        println!("      Reward: {}", reward);
        println!("      Nested: {}", nested);
        println!("      Archive: {}", archive);
        println!("      Unknown: {}", unknown);
        println!(
            "  Hash: {}",
            get_file_hash(get_sync_dir().join(name).to_str().unwrap()).unwrap()
        );
        greward += reward;
        gnested += nested;
        garchive += archive;
        gunknown += unknown;
        gentry += entries.len();
    }

    println!("====================");
    println!("Check report:");
    println!("  Alive: {}", alive);
    println!("  Dead: {}", dead);
    println!("  Entry: {}", gentry);
    println!("    Reward: {}", greward);
    println!("    Nested: {}", gnested);
    println!("    Archive: {}", garchive);
    println!("    Unknown: {}", gunknown);
    println!("  Enabled: {}", genabled);
    println!("  Disabled: {}", gdisabled);
    Ok(())
}

/*
    function handler for subcommand enable
    parameter:
        - name: name of the repository
    it will toggle enabled to the repository selected
*/
fn enable_repository(name: String) -> Result<(), Box<dyn std::error::Error>> {
    let mut doc: toml::Value = get_toml_config();

    doc["repositories"][name]["enabled"] = toml::Value::Boolean(true);

    write(get_config_file(), toml::to_string(&doc)?)?;
    Ok(())
}

/*
    function handler for subcommand disable
    parameter:
        - name: name of the repository
    it will toggle disabled to the repository selected
*/
fn disable_repository(name: String) -> Result<(), Box<dyn std::error::Error>> {
    let mut doc: toml::Value = get_toml_config();

    doc["repositories"][name]["enabled"] = toml::Value::Boolean(false);

    write(get_config_file(), toml::to_string(&doc)?)?;
    Ok(())
}
