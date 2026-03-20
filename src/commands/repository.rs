// src/commands/repository.rs
use crate::cli::RepositoryAction;
use crate::config::{create_agent, get_config_file, get_sync_dir, get_toml_config};
use std::fs::{remove_file, write};
use ureq::Agent;

pub fn run(action: RepositoryAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        RepositoryAction::Add { name, url } => add(name, url),
        RepositoryAction::Remove { name, keep_cache } => remove(name, keep_cache),
        RepositoryAction::List => list(),
        RepositoryAction::Sync { name, timeout } => sync(name, timeout),
        RepositoryAction::Check { timeout } => check(timeout),
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
    if it is alive or dead
*/
fn check(timeout: Option<u64>) -> Result<(), Box<dyn std::error::Error>> {
    let doc: toml::Value = get_toml_config();
    let mut alive = 0;
    let mut dead = 0;
    let agent = create_agent(timeout);

    for (name, data) in doc["repositories"].as_table().unwrap() {
        if !data["enabled"].as_bool().unwrap() {
            continue;
        }
        let url = data["url"].as_str().unwrap().to_string();
        println!("Checking: {}", name);
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
    }

    println!("Checking done:\n Alive: {}\n Dead: {}", alive, dead);
    Ok(())
}
