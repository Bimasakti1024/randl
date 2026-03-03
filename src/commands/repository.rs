// src/commands/repository.rs
use crate::cli::RepositoryAction;
use crate::config::{get_repos_file, get_sync_dir};

use std::fs::{File, OpenOptions, read_to_string, write};
use std::io::{BufRead, BufReader, Write};

pub fn run(action: RepositoryAction) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        RepositoryAction::Add { url } => add(url),
        RepositoryAction::Remove { url } => remove(url),
        RepositoryAction::List => list(),
        RepositoryAction::Sync => sync(),
    }
}

// function handler for add subcommand
fn add(url: String) -> Result<(), Box<dyn std::error::Error>> {
    let repos_file = get_repos_file();

    // read pass
    if repos_file.exists() {
        let reader = BufReader::new(File::open(&repos_file)?);
        for line in reader.lines() {
            if line? == url {
                println!("Repository {} already exists.", url);
                return Ok(());
            }
        }
    }

    // append pass
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(&repos_file)?;

    writeln!(file, "{}", url)?;
    Ok(())
}

// function handler for remove command
fn remove(url: String) -> Result<(), Box<dyn std::error::Error>> {
    let repos_file = get_repos_file();
    let repos = read_to_string(repos_file)?;

    if !repos.lines().any(|line| line == url) {
        eprintln!("Repository {} not found.", url);
        return Ok(());
    }

    let new_repos_content: String = repos
        .lines()
        .filter(|line| *line != url)
        .collect::<Vec<&str>>()
        .join("\n")
        + "\n";

    let _ = write(get_repos_file(), new_repos_content);
    Ok(())
}

// function handler for list command
fn list() -> Result<(), Box<dyn std::error::Error>> {
    let repos_file = get_repos_file();
    if !repos_file.exists() {
        println!("No repositores added yet.");
        return Ok(());
    }
    println!("{}", read_to_string(repos_file)?);
    Ok(())
}

// function handler for command sync
fn sync() -> Result<(), Box<dyn std::error::Error>> {
    let repos_file = get_repos_file();
    if !repos_file.exists() {
        println!("No repositories added yet.");
        return Ok(());
    }

    let mut success = 0;
    let mut error = 0;
    let repos = read_to_string(repos_file)?;
    let sync_dir = get_sync_dir();

    for line in repos.lines() {
        if line.is_empty() {
            continue;
        }

        print!("Syncing {}: ", line);
        let content = match reqwest::blocking::get(line) {
            Ok(response) => match response.error_for_status() {
                Ok(r) => match r.text() {
                    Ok(text) => text,
                    Err(e) => {
                        eprintln!("Failed to read response from {}: {}.", line, e);
                        error += 1;
                        continue;
                    }
                },
                Err(e) => {
                    eprintln!("Failed to fetch {}: {}.", line, e);
                    error += 1;
                    continue;
                }
            },
            Err(e) => {
                eprintln!("Failed to fetch {}: {}.", line, e);
                error += 1;
                continue;
            }
        };

        // turn URL into a safe filename
        let filename = line.replace("https://", "").replace("/", "_");
        let dest = sync_dir.join(filename);

        write(dest, content)?;
        success += 1;
        println!("Ok.");
    }

    println!("{} Repository synced and {} failed.", success, error);
    Ok(())
}
