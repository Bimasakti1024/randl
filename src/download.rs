// src/download.rs
use ::ureq::Agent;
use asky::Confirm;
use size::Size;
use std::fs::File;
use std::io::{self, Read, Write};
use std::path::Path;

use crate::util::filename_from_url;

pub fn download_file(
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
