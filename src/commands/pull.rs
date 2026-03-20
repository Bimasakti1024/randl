// src/commands/pull.rs
use asky::Confirm;
use figment::providers::Format;
use figment::{
    Figment,
    providers::{Serialized, Toml},
};
use rand::prelude::*;
use size::Size;
use std::fs::{File, read_to_string};
use std::io::{self, Read, Write};
use std::path::Path;
use ureq::Agent;

use crate::cli::{ConfigOverride, PullArgs};
use crate::config::{Config, create_agent, get_config_file, get_sync_dir};
use crate::vt::{scan_url, vt_report};

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

    match line.splitn(2, ' ').collect::<Vec<_>>().as_slice() {
        [url] => {
            // Attempt download if not dry run
            println!("Reward: {}.", filename_from_url(url));
            if conf_ref.scan_reward_url {
                let Some(vt_api_key) = conf_ref.vt_api_key.as_deref() else {
                    eprintln!("scan_reward_url is enabled but vt_api_key is not set.");
                    return FollowResult::Retry;
                };
                let reward_check = scan_url(&agent, vt_api_key, url);

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
                match Confirm::new("Download?").prompt() {
                    Ok(answer) => {
                        if !answer {
                            return FollowResult::Done;
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        return FollowResult::Retry;
                    }
                }
            }
            if conf_ref.dry_run {
                println!("Reward is not downloaded because it is a dry run.");
                FollowResult::Done
            } else {
                match download(
                    url,
                    agent,
                    conf_ref.output_directory.as_path(),
                    conf_ref.no_confirm,
                ) {
                    Ok(_) => FollowResult::Done,
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
        ["Nested", url] => {
            // Fetch nested repo and recurse
            match fetch_lines(agent, url) {
                Ok(nested) => follow(&nested, agent, config, current_depth + 1),
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

fn fetch_lines(agent: &ureq::Agent, url: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let content = agent.get(url).call()?.body_mut().read_to_string()?;
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
    agent: &Agent,
    output_dir: &Path,
    no_confirm: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let output_path = output_dir.join(filename_from_url(url));
    let output_filename = output_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    // HEAD request to get file size
    let head = agent.head(url).call()?;
    let size: Option<u64> = head
        .headers()
        .get("content-length")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse().ok());

    if !no_confirm {
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
    }
    let mut response = agent.get(url).call()?;
    let mut file = File::create(&output_path)?;

    println!("Downloading {}...", output_filename);

    let mut buffer = [0u8; 8192];
    let mut bytes_written: u64 = 0;
    let mut last_reported = 0u64;

    let mut reader = response.body_mut().as_reader();

    loop {
        let n = reader.read(&mut buffer)?;
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
