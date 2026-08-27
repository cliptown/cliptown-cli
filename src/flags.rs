use std::collections::HashMap;
use std::path::Path;

use crate::env_map::{merge_env, EnvMap};
use crate::error::CliError;
use flags2env::BundledFlags2Env;

pub fn parse_cli_flags(
    argv: &[String],
    config_path: &Path,
) -> Result<HashMap<String, String>, CliError> {
    let config_path = config_path
        .to_str()
        .ok_or_else(|| CliError::Configuration(".cli-flags.toml path is not valid UTF-8".into()))?;
    let parser = BundledFlags2Env::new();
    parser.audit_config(Some(config_path)).map_err(|error| {
        CliError::Configuration(format!("flags-2-env configuration audit failed: {error}"))
    })?;
    let parsed = parser
        .parse_structured(argv, Some(config_path))
        .map_err(|error| CliError::Parsing(format!("flags-2-env parse failed: {error}")))?;
    if !parsed.unknown_options.is_empty() {
        return Err(CliError::Parsing(format!(
            "unknown command-line option(s): {}",
            parsed.unknown_options.join(", ")
        )));
    }
    if !parsed.errors.is_empty() {
        return Err(CliError::Parsing(format!(
            "invalid command-line value(s): {}",
            parsed.errors.join("; ")
        )));
    }
    let mut flags = parsed.flags;
    if !parsed.command.is_empty() {
        flags.insert("CLIPTOWN_COMMAND".into(), parsed.command);
    }
    if !parsed.extras.is_empty() {
        flags.insert(
            "CLIPTOWN_POSITIONALS".into(),
            serde_json::to_string(&parsed.extras).unwrap_or_else(|_| "[]".into()),
        );
    }
    Ok(flags)
}

pub fn apply_cli_flags() -> Result<EnvMap, CliError> {
    apply_cli_flags_from(
        std::env::args().collect(),
        std::env::vars().collect(),
        Path::new(".cli-flags.toml"),
    )
}

pub fn apply_cli_flags_from(
    argv: Vec<String>,
    initial: EnvMap,
    config_path: &Path,
) -> Result<EnvMap, CliError> {
    Ok(merge_env(initial, parse_cli_flags(&argv, config_path)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env_map::value;

    fn config_path() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join(".cli-flags.toml")
    }

    #[test]
    fn doctor_command_merges_without_mutating_process_environment() {
        let before = std::env::var_os("CLIPTOWN_ENDPOINT");
        let env = apply_cli_flags_from(
            vec!["cliptown".into(), "doctor".into()],
            EnvMap::from([(
                "CLIPTOWN_ENDPOINT".into(),
                "https://api.cliptown.app".into(),
            )]),
            &config_path(),
        )
        .expect("valid flags");
        assert_eq!(value(&env, "CLIPTOWN_COMMAND"), Some("doctor"));
        assert_eq!(
            value(&env, "CLIPTOWN_ENDPOINT"),
            Some("https://api.cliptown.app")
        );
        assert_eq!(std::env::var_os("CLIPTOWN_ENDPOINT"), before);
    }

    #[test]
    fn parse_failure_does_not_mutate_process_environment() {
        let before = std::env::var_os("CLIPTOWN_ENDPOINT");
        assert!(apply_cli_flags_from(
            vec![
                "cliptown".into(),
                "doctor".into(),
                "--this-flag-is-not-declared".into()
            ],
            EnvMap::from([("CLIPTOWN_ENDPOINT".into(), "keep".into())]),
            &config_path(),
        )
        .is_err());
        assert_eq!(std::env::var_os("CLIPTOWN_ENDPOINT"), before);
    }

    #[test]
    fn source_does_not_mutate_process_environment() {
        const SRC: &str = include_str!("flags.rs");
        let production = SRC.split("#[cfg(test)]").next().unwrap_or(SRC);
        assert!(!production.contains("std::env::set_var"));
        assert!(!production.contains("env::set_var"));
    }
}
