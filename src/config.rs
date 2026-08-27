use directories::ProjectDirs;

use crate::env_map::{truthy, value, EnvMap};
use crate::error::CliError;

#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    pub endpoint: String,
    pub json: bool,
    pub config_dir: std::path::PathBuf,
}

impl RuntimeConfig {
    pub fn output_json_requested() -> bool {
        Self::output_json_from(&std::env::vars().collect())
    }

    pub fn output_json_from(env: &EnvMap) -> bool {
        truthy(env, "CLIPTOWN_OUTPUT_JSON")
    }

    #[allow(dead_code)]
    pub fn from_env() -> Result<Self, CliError> {
        Self::from_env_map(&std::env::vars().collect())
    }

    pub fn from_env_map(env: &EnvMap) -> Result<Self, CliError> {
        let endpoint = value(env, "CLIPTOWN_ENDPOINT")
            .unwrap_or("https://api.cliptown.app")
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
        let json = Self::output_json_from(env);
        let dirs = ProjectDirs::from("app", "ClipTown", "ClipTown")
            .ok_or_else(|| CliError::Configuration("cannot resolve config directory".into()))?;
        Ok(Self {
            endpoint,
            json,
            config_dir: dirs.config_dir().to_path_buf(),
        })
    }
}
