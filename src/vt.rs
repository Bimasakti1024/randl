// src/vt.rs
use ureq::Agent;

/*
    A function helper to check URL Safety
    using VirusTotal API.
    Parameter:
        - Agent: Agent to use for requesting
        - vt_api_key: The API key
        - url: The URL to scan
*/
pub fn scan_url(
    agent: &Agent,
    vt_api_key: &str,
    url: &str,
) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
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
    Ok(report_json)
}
