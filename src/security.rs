// src/security.rs
use sha2::{Digest, Sha256};
use std::time::Duration;
use std::{fs, io, thread};
use ureq::Agent;

pub struct vt_report {
    pub url: String,
    pub harmless: u32,
    pub undetected: u32,
    pub suspicious: u32,
    pub malicious: u32,
}

/*
    A function helper to check URL Safety
    using VirusTotal API.
    Parameter:
        - Agent: Agent to use for requesting
        - vt_api_key: The API key
        - url: The URL to scan
    return a vt_report
*/
pub fn scan_url(
    agent: &Agent,
    vt_api_key: &str,
    url: &str,
) -> Result<vt_report, Box<dyn std::error::Error>> {
    let mut scan_response = agent
        .post("https://www.virustotal.com/api/v3/urls")
        .header("accept", "application/json")
        .header("x-apikey", vt_api_key)
        .send_form([("url", url)])?;

    let scan_json: serde_json::Value = scan_response.body_mut().read_json()?;
    let report_url = scan_json["data"]["links"]["self"]
        .as_str()
        .ok_or("missing report URL")?
        .to_string();

    let mut attempts = 0;
    let report_json = loop {
        if attempts >= 10 {
            return Err("VirusTotal analysis timed out".into());
        }
        let mut response = agent
            .get(&report_url)
            .header("accept", "application/json")
            .header("x-apikey", vt_api_key)
            .call()?;
        let json: serde_json::Value = response.body_mut().read_json()?;
        match json["data"]["attributes"]["status"].as_str() {
            Some("completed") => break json,
            Some("queued") | Some("in-progress") => {
                println!("Scanning queued or in-progress...");
                thread::sleep(Duration::from_secs(15));
            }
            _ => return Err("unexpected VirusTotal analysis status".into()),
        }
        attempts += 1;
    };

    Ok(vt_report {
        url: report_json["data"]["attributes"]["url"]
            .as_str()
            .ok_or("missing url")?
            .to_string(),
        harmless: report_json["data"]["attributes"]["stats"]["harmless"]
            .as_u64()
            .ok_or("missing harmless")? as u32,
        undetected: report_json["data"]["attributes"]["stats"]["undetected"]
            .as_u64()
            .ok_or("missing undetected")? as u32,
        suspicious: report_json["data"]["attributes"]["stats"]["suspicious"]
            .as_u64()
            .ok_or("missing suspicious")? as u32,
        malicious: report_json["data"]["attributes"]["stats"]["malicious"]
            .as_u64()
            .ok_or("missing malicious")? as u32,
    })
}

pub fn get_file_hash(path: &str) -> io::Result<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();

    // Stream the file content into the hasher
    io::copy(&mut file, &mut hasher)?;

    let hash = hasher.finalize();
    Ok(format!("{:x}", hash))
}
