// src/vt.rs
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
    // POST to submit URL for scanning
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

    // GET the analysis report
    let mut report_response = agent
        .get(&report_url)
        .header("accept", "application/json")
        .header("x-apikey", vt_api_key)
        .call()?;

    let report_json: serde_json::Value = report_response.body_mut().read_json()?;

    let stats = &report_json["data"]["attributes"]["stats"];

    Ok(vt_report {
        url: report_json["data"]["attributes"]["url"]
            .as_str()
            .ok_or("missing url")?
            .to_string(),
        harmless: stats["harmless"].as_u64().ok_or("missing harmless")? as u32,
        undetected: stats["undetected"].as_u64().ok_or("missing undetected")? as u32,
        suspicious: stats["suspicious"].as_u64().ok_or("missing suspicious")? as u32,
        malicious: stats["malicious"].as_u64().ok_or("missing malicious")? as u32,
    })
}
