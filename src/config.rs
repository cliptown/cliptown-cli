use directories::ProjectDirs;

use crate::error::CliError;

#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub endpoint: String,
    pub json: bool,
    pub config_dir: std::path::PathBuf,
}

impl RuntimeConfig {
    pub fn output_json_override() -> Option<bool> {
        std::env::var("CLIPTOWN_OUTPUT_JSON").ok().and_then(|value| {
            match value.trim().to_ascii_lowercase().as_str() {
                "1" | "true" | "yes" | "on" => Some(true),
                "0" | "false" | "no" | "off" => Some(false),
                _ => None,
            }
        })
    }

    pub fn output_json_requested() -> bool {
        Self::output_json_override().unwrap_or(false)
    }

    pub fn from_env(json: bool) -> Result<Self, CliError> {
        let endpoint = std::env::var("CLIPTOWN_ENDPOINT")
            .unwrap_or_else(|_| "https://api.cliptown.app".into())
            .trim_end_matches('/')
            .to_owned();
        require_secure_endpoint(&endpoint)?;
        let dirs = ProjectDirs::from("app", "ClipTown", "ClipTown")
            .ok_or_else(|| CliError::Configuration("cannot resolve config directory".into()))?;
        Ok(Self {
            endpoint,
            json,
            config_dir: dirs.config_dir().to_path_buf(),
        })
    }
}

fn require_secure_endpoint(endpoint: &str) -> Result<(), CliError> {
    match endpoint {
        value if value.starts_with("https://") => Ok(()),
        value if value.starts_with("http://localhost") => Ok(()),
        value if value.starts_with("http://127.0.0.1") => Ok(()),
        _ => Err(CliError::Configuration(
            "endpoint must use HTTPS outside localhost".into(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::require_secure_endpoint;

    #[test]
    fn localhost_http_and_https_are_accepted() {
        assert!(require_secure_endpoint("https://api.cliptown.app").is_ok());
        assert!(require_secure_endpoint("http://localhost:8080").is_ok());
        assert!(require_secure_endpoint("http://127.0.0.1:8080").is_ok());
        assert!(require_secure_endpoint("http://example.com").is_err());
    }
}
