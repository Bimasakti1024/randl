// src/commands/pull.rs
use asky::Confirm;
use figment::providers::Format;
use figment::{
    Figment,
    providers::{Serialized, Toml},
};
use rand::prelude::*;
use size::Size;
use std::fs::read_to_string;
use std::io::Read;
use ureq::Agent;

use crate::archive::*;
use crate::cli::{ConfigOverride, PullArgs};
use crate::commands::repository::{Repository, RepositoryType, parse_repository};
use crate::config::{Config, get_config_file, get_sync_dir};
use crate::download::{download_file, get_download_size};
use crate::security::scan_url;
use crate::util::{create_agent, filename_from_url};

enum FollowResult {
    Done,
    Retry,
    Error(Box<dyn std::error::Error>),
}

pub fn run(args: PullArgs) -> Result<(), Box<dyn std::error::Error>> {
    let config: Config = Figment::new()
        .merge(Toml::file(get_config_file()))
        .merge(Serialized::globals(&ConfigOverride {
            configuration: &args,
        }))
        .extract()?;
    let conf_ref = &config.configuration;
    let agent = create_agent(Some(conf_ref.timeout));

    // if the form flag is provided an argument
    if let Some(ref url) = args.from {
        println!("Pulling from: {}", url);

        let repos = fetch_lines(&agent, url)?;
        for _ in 1..=conf_ref.repeat {
            loop {
                match follow(&repos, &agent, &config, 1) {
                    FollowResult::Done => break,
                    FollowResult::Retry => continue,
                    FollowResult::Error(e) => {
                        eprintln!("{}", e);
                        if conf_ref.no_confirm {
                            continue;
                        }
                        if Confirm::new("Continue? ").prompt()? {
                            continue;
                        } else {
                            break;
                        }
                    }
                }
            }
        }
        return Ok(());
    }
    println!("Loading repositories...");

    let repos = &config.repositories;
    let mut rng = rand::rng();

    let enabled: Vec<_> = repos.iter().filter(|(_, data)| data.enabled).collect();

    if enabled.is_empty() {
        eprintln!("No enabled repositories.");
        return Ok(());
    }

    let (srepo_name, _) = enabled.choose(&mut rng).unwrap();

    let repo_content = read_to_string(get_sync_dir().join(srepo_name))?;

    // Collect repository and remove comment
    let repos: Vec<String> = repo_content
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .map(|l| l.to_string())
        .collect();

    if repos.is_empty() {
        println!("Repository {} is empty.", srepo_name);
        return Ok(());
    }

    for _ in 1..=conf_ref.repeat {
        loop {
            match follow(&repos, &agent, &config, 1) {
                FollowResult::Done => break,
                FollowResult::Retry => continue,
                FollowResult::Error(e) => {
                    println!("{}", e);
                    if conf_ref.no_confirm {
                        continue;
                    }
                    if Confirm::new("Continue? ").prompt()? {
                        continue;
                    } else {
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}

fn follow(repos: &[String], agent: &Agent, config: &Config, current_depth: u32) -> FollowResult {
    let conf_ref = &config.configuration;
    let max_depth = conf_ref.max_depth;
    if current_depth > max_depth && max_depth > 0 {
        println!("Max depth reached.");
        if conf_ref.no_confirm {
            return FollowResult::Retry;
        }
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

    let repo: Repository = parse_repository(line.to_owned());
    let url_string = repo.url.unwrap();
    let url = url_string.as_str();
    match repo.repo_type {
        RepositoryType::Reward => {
            // Attempt download if not dry run
            println!("Reward: {}.", filename_from_url(url));
            if conf_ref.scan_reward_url {
                let Some(vt_api_key) = conf_ref.vt_api_key.as_deref() else {
                    eprintln!("scan_reward_url is enabled but vt_api_key is not set.");
                    return FollowResult::Retry;
                };
                let reward_check = scan_url(agent, vt_api_key, &url);

                match reward_check {
                    Ok(report) => {
                        println!("VirusTotal Check report:");
                        println!("  Malicious: {}", report.malicious);
                        println!("  Suspicious: {}", report.suspicious);
                        println!("  Undetected: {}", report.undetected);
                        println!("  Harmless: {}", report.harmless);
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                    }
                }
            }
            if conf_ref.dry_run {
                println!("Reward is not downloaded because it is a dry run.");
                FollowResult::Done
            } else {
                let output_path = conf_ref.output_directory.join(filename_from_url(url));
                let output_filename = output_path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();

                let size = get_download_size(agent, url);

                if !conf_ref.no_confirm {
                    match size {
                        Some(s) => println!(
                            "  File: {}\n  Size: {}",
                            output_filename,
                            Size::from_bytes(s)
                        ),
                        None => println!("  File: {}\n  Size: unknown", output_filename),
                    }

                    if !Confirm::new("Download this reward?").prompt().unwrap() {
                        return FollowResult::Retry;
                    }
                }
                println!("Downloading {}", output_filename);
                match download_file(url, agent, conf_ref.output_directory.as_path()) {
                    Ok(_) => {
                        println!("Download successful.");
                        FollowResult::Done
                    }
                    Err(e) => {
                        // Distinguish user cancellation from real errors
                        if e.to_string() == "cancelled" {
                            println!("Re-rolling...");
                            FollowResult::Retry
                        } else {
                            eprintln!("Download failed: {e}\nRetrying...");
                            FollowResult::Retry
                        }
                    }
                }
            }
        }
        RepositoryType::Nested => {
            // Fetch nested repo and recurse
            match fetch_lines(agent, url) {
                Ok(nested) => follow(&nested, agent, config, current_depth + 1),
                Err(e) => FollowResult::Error(e),
            }
        }
        RepositoryType::Archive => {
            // Get response and then extract
            let output_filename = filename_from_url(url);
            println!("Archived Reward: {}", filename_from_url(url));
            if conf_ref.scan_reward_url {
                let Some(vt_api_key) = conf_ref.vt_api_key.as_deref() else {
                    eprintln!("scan_reward_url is enabled but vt_api_key is not set.");
                    return FollowResult::Retry;
                };
                let reward_check = scan_url(agent, vt_api_key, &url);

                match reward_check {
                    Ok(report) => {
                        println!("VirusTotal Check report:");
                        println!("  Malicious: {}", report.malicious);
                        println!("  Suspicious: {}", report.suspicious);
                        println!("  Undetected: {}", report.undetected);
                        println!("  Harmless: {}", report.harmless);
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                    }
                }
            }
            if conf_ref.dry_run {
                println!("Reward is not downloaded because it is a dry run.");
                FollowResult::Done
            } else {
                let size = get_download_size(agent, url);

                if !conf_ref.no_confirm {
                    match size {
                        Some(s) => println!(
                            "  File: {}\n  Compressed size: {}",
                            output_filename,
                            Size::from_bytes(s)
                        ),
                        None => println!("  File: {}\n  Compressed size: unknown", output_filename),
                    }

                    if !Confirm::new("Download this reward?").prompt().unwrap() {
                        return FollowResult::Retry;
                    }
                }
                match agent.get(url).call() {
                    Ok(mut response) => {
                        if response.status() != 200 {
                            eprintln!("Server responded: {}", response.status());
                            return FollowResult::Retry;
                        }
                        let mut reader = response.body_mut().as_reader();

                        println!("Determining archive type...");
                        let mut magic = [0u8; 6];
                        match reader.read_exact(&mut magic) {
                            Ok(_) => {}
                            Err(e) => {
                                eprintln!("An error occured: {}", e);
                                return FollowResult::Error(Box::new(e));
                            }
                        }

                        let archive_type: ArchiveType = detect_type(&magic);

                        println!("Extracting...");
                        let full_reader = std::io::Cursor::new(magic).chain(reader);

                        if let Err(e) = extract(
                            full_reader,
                            archive_type,
                            conf_ref.output_directory.as_path(),
                        ) {
                            eprintln!("Extraction failed: {}", e);
                            return FollowResult::Error(e);
                        }
                        FollowResult::Done
                    }
                    Err(e) => {
                        eprintln!("An error occured: {}", e);
                        FollowResult::Retry
                    }
                }
            }
        }
        _ => {
            // If line format in unrecognised, will retry
            eprintln!("Unrecognised line format, retrying...");
            FollowResult::Retry
        }
    }
}

fn fetch_lines(agent: &ureq::Agent, url: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let content = agent.get(url).call()?.body_mut().read_to_string()?;
    Ok(content
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .map(|l| l.to_string())
        .collect())
}

#[cfg(test)]
mod test {
    use crate::commands::pull::filename_from_url;

    #[test]
    fn filename_from_url_test() {
        assert_eq!(filename_from_url("https://example.com/test"), "test");
    }

    #[test]
    fn test_filename_from_url_trailing_slash() {
        assert_eq!(filename_from_url("https://example.com/"), "randl-reward");
    }

    #[test]
    fn test_filename_from_url_no_path() {
        assert_eq!(filename_from_url("https://example.com"), "example.com");
    }
}
