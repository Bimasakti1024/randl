// src/util.rs
use crate::config::get_toml_config;
use std::time::Duration;
use ureq::Agent;

/*
    a function to get a filename from a url
    parameter:
        - url: url
*/
pub fn filename_from_url(url: &str) -> String {
    // Strip query parameters before extracting filename
    let path = url.split('?').next().unwrap_or(url);
    path.split('/')
        .next_back()
        .filter(|s| !s.is_empty())
        .unwrap_or("randl-reward")
        .to_string()
}

/*
    a function to truncate a string with max length
    parameters:
        - s: the string to truncate
        - max_len: max length (if 0 will not truncate)
*/
pub fn truncate_string(s: &str, max_len: usize) -> String {
    if max_len == 0 || s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}

pub fn create_agent(t: Option<u64>) -> Agent {
    let timeout = t.unwrap_or_else(|| {
        get_toml_config()
            .as_table()
            .and_then(|doc| doc["configuration"].as_table())
            .and_then(|conf| conf["timeout"].as_integer())
            .unwrap_or(30)
            .try_into()
            .unwrap_or(30)
    });

    Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(timeout)))
        .build()
        .into()
}

#[cfg(test)]
mod test {
    use crate::util::{create_agent, filename_from_url, truncate_string};

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

    #[test]
    fn test_truncate_string() {
        assert_eq!(truncate_string("Hello, World!", 5), "Hello...");
    }

    #[test]
    fn test_truncate_string_zero() {
        assert_eq!(truncate_string("Hello, World!", 0), "Hello, World!");
    }

    #[test]
    fn create_agent_with_timeout() {
        create_agent(Some(1));
    }

    #[test]
    fn create_agent_without_timeout() {
        create_agent(None);
    }
}
