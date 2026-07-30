use directories::ProjectDirs;

use crate::error::CliError;

#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub endpoint: String,
    pub json: bool,
    pub config_dir: std::path::PathBuf,
}

impl RuntimeConfig {
    pub fn output_json_requested() -> bool {
        matches!(
            std::env::var("CLIPTOWN_OUTPUT_JSON").as_deref(),
            Ok("true" | "1")
        )
    }

    pub fn from_env() -> Result<Self, CliError> {
        let endpoint = std::env::var("CLIPTOWN_ENDPOINT")
            .unwrap_or_else(|_| "https://api.cliptown.app".into())
            .trim_end_matches('/')
            .to_owned();
        let secure = endpoint.starts_with("https://")
            || endpoint.starts_with("http://localhost")
            || endpoint.starts_with("http://127.0.0.1");
        if !secure {
            return Err(CliError::Configuration(
                "endpoint must use HTTPS outside localhost".into(),
            ));
        }
        let json = Self::output_json_requested();
        let dirs = ProjectDirs::from("app", "ClipTown", "ClipTown")
            .ok_or_else(|| CliError::Configuration("cannot resolve config directory".into()))?;
        Ok(Self {
            endpoint,
            json,
            config_dir: dirs.config_dir().to_path_buf(),
        })
    }
}
