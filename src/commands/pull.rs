// src/commands/pull.rs
use asky::Confirm;
use rand::prelude::*;
use reqwest::blocking::Client;
use size::Size;
use std::fs::{self, File, read_to_string};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

use crate::cli::PullArgs;
use crate::config::get_sync_dir;

enum FollowResult {
    Done,
    Retry,
    Error(Box<dyn std::error::Error>),
}

pub fn run(args: PullArgs) -> Result<(), Box<dyn std::error::Error>> {
    println!("Loading repositories...");

    let mut repo_files: Vec<PathBuf> = Vec::new();
    for entry in fs::read_dir(get_sync_dir())? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            repo_files.push(path);
        }
    }

    if repo_files.is_empty() {
        eprintln!("No repositories synced. Run: randl repository sync");
        return Ok(());
    }

    let mut rng = rand::rng();
    let repo_path = repo_files
        .choose(&mut rng)
        .ok_or("Failed to choose a random repository.")?;

    let repo_content = read_to_string(repo_path)?;

    // Collect repository and remove comment
    let repos: Vec<String> = repo_content
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .map(|l| l.to_string())
        .collect();

    if repos.is_empty() {
        let name = repo_path.file_name().unwrap_or_default().to_string_lossy();
        eprintln!("Repository {} is empty.", name);
        return Ok(());
    }

    let client = Client::new();
    loop {
        match follow(&repos, &client, &args, 1) {
            FollowResult::Done => break,
            FollowResult::Retry => continue,
            FollowResult::Error(e) => {
                eprintln!("Error: {}", e);
                break;
            }
        }
    }

    Ok(())
}

fn follow(
    repos: &[String],
    client: &Client,
    args: &PullArgs,
    current_depth: u32,
) -> FollowResult {
    let max_depth = args.max_depth;
    if current_depth > max_depth && max_depth > 0 {
        eprintln!("Max depth reached.");
        return match Confirm::new("Retry?").prompt() {
            Ok(true) => FollowResult::Retry,
            Ok(false) => FollowResult::Done,
            Err(e) => FollowResult::Error(e.into()),
        };
    }

    let mut rng = rand::rng();
    let line = match repos.choose(&mut rng) {
        Some(l) => l,
        None => return FollowResult::Error("Repository has no lines.".into()),
    };

    match line.splitn(2, ' ').collect::<Vec<_>>().as_slice() {
        [url] => {
            // Attempt download if not dry run
            if args.dry_run {
                println!("Reward: {}.", filename_from_url(url));
                println!("Reward is not downloaded because it is a dry run.");
                FollowResult::Done
            } else {
                match download(url, client, args.output_directory.as_path()) {
                    Ok(_) => FollowResult::Done,
                    Err(e) => {
                        // Distinguish user cancellation from real errors
                        if e.to_string() == "cancelled" {
                            println!("Re-rolling...");
                            FollowResult::Retry
                        } else {
                            eprintln!("Download failed: {}\nRetrying...", e);
                            FollowResult::Retry
                        }
                    }
                }
            }
        }
        ["Nested", url] => {
            // Fetch nested repo and recurse
            match fetch_lines(client, url) {
                Ok(nested) => follow(&nested, client, args, current_depth + 1),
                Err(e) => FollowResult::Error(e),
            }
        }
        _ => {
            // If line format in unrecognised, will retry
            eprintln!("Unrecognised line format, retrying...");
            FollowResult::Retry
        }
    }
}

fn fetch_lines(client: &Client, url: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut response = client.get(url).send()?.error_for_status()?;
    let mut content = String::new();
    response.read_to_string(&mut content)?;
    Ok(content
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .map(|l| l.to_string())
        .collect())
}

fn filename_from_url(url: &str) -> String {
    // Strip query parameters before extracting filename
    let path = url.split('?').next().unwrap_or(url);
    path.split('/')
        .next_back()
        .filter(|s| !s.is_empty())
        .unwrap_or("randl-reward")
        .to_string()
}

fn download(
    url: &str,
    client: &Client,
    output_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let output_path = output_dir.join(filename_from_url(url));
    let output_filename = output_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    // HEAD request to get file size
    let head = client.head(url).send()?;
    let size: Option<u64> = head
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse().ok());

    match size {
        Some(s) => println!(
            "  File: {}\n  Size: {}",
            output_filename,
            Size::from_bytes(s)
        ),
        None => println!("  File: {}\n  Size: unknown", output_filename),
    }

    if !Confirm::new("Download this reward?").prompt()? {
        return Err("cancelled".into());
    }

    let mut response = client.get(url).send()?.error_for_status()?;
    let mut file = File::create(&output_path)?;

    println!("Downloading {}...", output_filename);

    let mut buffer = [0u8; 8192];
    let mut bytes_written: u64 = 0;
    let mut last_reported = 0u64;

    loop {
        let n = response.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        file.write_all(&buffer[..n])?;
        bytes_written += n as u64;

        // Print progress every ~512KB
        if bytes_written - last_reported >= 524_288 {
            match size {
                Some(total) => print!("\r  {:.1}%", bytes_written as f64 / total as f64 * 100.0),
                None => print!("\r  {}", Size::from_bytes(bytes_written)),
            }
            io::stdout().flush()?;
            last_reported = bytes_written;
        }
    }

    println!("\rSaved to {}", output_filename);
    Ok(())
}
