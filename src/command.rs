use std::io::Read;

use arboard::Clipboard;

use crate::{
    config::RuntimeConfig,
    env_map::{truthy, value, EnvMap},
    error::CliError,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    AuthLogin {
        reauth_days: u8,
    },
    AuthStatus,
    AuthLogout,
    ClipList {
        limit: u32,
    },
    ClipGet {
        clip_id: String,
    },
    ClipAdd {
        file: Option<String>,
        from_stdin: bool,
        from_clipboard: bool,
        pin: bool,
    },
    ClipPin {
        clip_id: String,
        pinned: bool,
    },
    ClipDelete {
        clip_id: String,
    },
    ClipCopy {
        clip_id: String,
    },
    ClipSearch {
        query: String,
        mode: String,
    },
    SyncPull,
    SyncPush,
    SyncStatus,
    SyncPair {
        transport: String,
    },
    ConfigGet {
        key: String,
    },
    ConfigSet {
        key: String,
        value: String,
    },
    Doctor,
}

impl Command {
    #[allow(dead_code)]
    pub fn from_env() -> Result<Self, CliError> {
        Self::from_env_map(&std::env::vars().collect())
    }

    pub fn from_env_map(env: &EnvMap) -> Result<Self, CliError> {
        let path = value(env, "CLIPTOWN_COMMAND")
            .unwrap_or_default()
            .to_owned();
        let args: Vec<String> =
            serde_json::from_str(value(env, "CLIPTOWN_POSITIONALS").unwrap_or("[]"))
                .map_err(|error| CliError::Parsing(error.to_string()))?;
        let argument = |index: usize| {
            args.get(index)
                .cloned()
                .ok_or_else(|| CliError::Parsing(format!("missing argument {index} for {path}")))
        };

        let command = match path.as_str() {
            "auth login" => {
                let reauth_days = value(env, "CLIPTOWN_REAUTH_DAYS")
                    .unwrap_or("10")
                    .parse()
                    .map_err(|_| CliError::Parsing("reauth-days must be an integer".into()))?;
                if !(1..=20).contains(&reauth_days) {
                    return Err(CliError::Parsing("reauth-days must be 1..=20".into()));
                }
                Self::AuthLogin { reauth_days }
            }
            "auth status" => Self::AuthStatus,
            "auth logout" => Self::AuthLogout,
            "clip list" => {
                let limit: u32 = value(env, "CLIPTOWN_LIMIT")
                    .unwrap_or("20")
                    .parse()
                    .map_err(|_| CliError::Parsing("limit must be an integer".into()))?;
                if !(1..=500).contains(&limit) {
                    return Err(CliError::Parsing("limit must be 1..=500".into()));
                }
                Self::ClipList { limit }
            }
            "clip get" => Self::ClipGet {
                clip_id: argument(0)?,
            },
            "clip add" => Self::ClipAdd {
                file: value(env, "CLIPTOWN_FILE").map(str::to_owned),
                from_stdin: truthy(env, "CLIPTOWN_STDIN"),
                from_clipboard: truthy(env, "CLIPTOWN_FROM_CLIPBOARD"),
                pin: truthy(env, "CLIPTOWN_PIN_CLIP"),
            },
            "clip pin" => Self::ClipPin {
                clip_id: argument(0)?,
                pinned: true,
            },
            "clip unpin" => Self::ClipPin {
                clip_id: argument(0)?,
                pinned: false,
            },
            "clip delete" => Self::ClipDelete {
                clip_id: argument(0)?,
            },
            "clip copy" => Self::ClipCopy {
                clip_id: argument(0)?,
            },
            "clip search" => {
                let query = match value(env, "CLIPTOWN_QUERY") {
                    Some(query) => query.to_owned(),
                    None => argument(0)?,
                };
                Self::ClipSearch {
                    query,
                    mode: value(env, "CLIPTOWN_SEARCH_MODE")
                        .unwrap_or("local_only")
                        .to_owned(),
                }
            }
            "sync pull" => Self::SyncPull,
            "sync push" => Self::SyncPush,
            "sync status" => Self::SyncStatus,
            "sync pair" => Self::SyncPair {
                transport: value(env, "CLIPTOWN_PAIR_TRANSPORT")
                    .unwrap_or("wifi")
                    .to_owned(),
            },
            "config get" => Self::ConfigGet { key: argument(0)? },
            "config set" => Self::ConfigSet {
                key: argument(0)?,
                value: argument(1)?,
            },
            "doctor" => Self::Doctor,
            _ => {
                return Err(CliError::Parsing(format!(
                    "unknown or missing command: {path}"
                )))
            }
        };
        Ok(command)
    }

    pub async fn execute(self, config: RuntimeConfig) -> Result<(), CliError> {
        match self {
            Self::Doctor => {
                let clipboard = Clipboard::new().map(|_| "ok").unwrap_or("unavailable");
                emit_result(
                    &config,
                    serde_json::json!({
                        "command": "doctor",
                        "endpoint": config.endpoint,
                        "config_dir": config.config_dir,
                        "clipboard": clipboard,
                        "flags2env": "ok"
                    }),
                    format!(
                        "endpoint={} config_dir={} clipboard={} flags2env=ok",
                        config.endpoint,
                        config.config_dir.display(),
                        clipboard
                    ),
                );
            }
            Self::ClipAdd {
                file,
                from_stdin,
                from_clipboard,
                pin,
            } => {
                let source_count = [file.is_some(), from_stdin, from_clipboard]
                    .into_iter()
                    .filter(|selected| *selected)
                    .count();
                if source_count != 1 {
                    return Err(CliError::Parsing(
                        "choose exactly one of --stdin, --file, or --from-clipboard".into(),
                    ));
                }

                let (payload, source) = if from_clipboard {
                    Clipboard::new()
                        .and_then(|mut clipboard| clipboard.get_text())
                        .map(|payload| (payload, "clipboard"))
                        .map_err(|error| CliError::Clipboard(error.to_string()))?
                } else if from_stdin {
                    let mut payload = String::new();
                    std::io::stdin().read_to_string(&mut payload)?;
                    (payload, "stdin")
                } else if let Some(path) = file {
                    (std::fs::read_to_string(path)?, "file")
                } else {
                    unreachable!("source count was validated")
                };

                emit_result(
                    &config,
                    serde_json::json!({
                        "command": "clip.add",
                        "status": "queued",
                        "source": source,
                        "byte_count": payload.len(),
                        "pinned": pin,
                    }),
                    format!(
                        "queued encrypted clip ({} bytes, pinned={pin})",
                        payload.len()
                    ),
                );
            }
            Self::AuthLogin { reauth_days } => emit_result(
                &config,
                serde_json::json!({
                    "command": "auth.login",
                    "status": "not_implemented",
                    "reauth_days": reauth_days,
                }),
                format!(
                    "start Supabase PKCE + 3FA/shared-auth login \
                     (reauth every {reauth_days} days)"
                ),
            ),
            other => {
                let command = other.path();
                emit_result(
                    &config,
                    serde_json::json!({
                        "command": command,
                        "status": "not_implemented",
                        "endpoint": config.endpoint,
                    }),
                    format!("{command} is not implemented against {}", config.endpoint),
                );
            }
        }
        Ok(())
    }

    fn path(&self) -> &'static str {
        match self {
            Self::AuthLogin { .. } => "auth.login",
            Self::AuthStatus => "auth.status",
            Self::AuthLogout => "auth.logout",
            Self::ClipList { .. } => "clip.list",
            Self::ClipGet { .. } => "clip.get",
            Self::ClipAdd { .. } => "clip.add",
            Self::ClipPin { pinned: true, .. } => "clip.pin",
            Self::ClipPin { pinned: false, .. } => "clip.unpin",
            Self::ClipDelete { .. } => "clip.delete",
            Self::ClipCopy { .. } => "clip.copy",
            Self::ClipSearch { .. } => "clip.search",
            Self::SyncPull => "sync.pull",
            Self::SyncPush => "sync.push",
            Self::SyncStatus => "sync.status",
            Self::SyncPair { .. } => "sync.pair",
            Self::ConfigGet { .. } => "config.get",
            Self::ConfigSet { .. } => "config.set",
            Self::Doctor => "doctor",
        }
    }
}

fn emit_result(config: &RuntimeConfig, result: serde_json::Value, plain: String) {
    if config.json {
        println!(
            "{}",
            serde_json::json!({
                "schema_version": 1,
                "ok": true,
                "result": result,
            })
        );
    } else {
        println!("{plain}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_login_window() {
        let before = std::env::var_os("CLIPTOWN_REAUTH_DAYS");
        let env = EnvMap::from([
            ("CLIPTOWN_COMMAND".into(), "auth login".into()),
            ("CLIPTOWN_REAUTH_DAYS".into(), "20".into()),
            ("CLIPTOWN_POSITIONALS".into(), "[]".into()),
        ]);
        assert_eq!(
            Command::from_env_map(&env).unwrap(),
            Command::AuthLogin { reauth_days: 20 }
        );
        assert_eq!(std::env::var_os("CLIPTOWN_REAUTH_DAYS"), before);
    }

    #[test]
    fn rejects_long_window() {
        let env = EnvMap::from([
            ("CLIPTOWN_COMMAND".into(), "auth login".into()),
            ("CLIPTOWN_REAUTH_DAYS".into(), "21".into()),
            ("CLIPTOWN_POSITIONALS".into(), "[]".into()),
        ]);
        assert!(Command::from_env_map(&env).is_err());
    }

    #[test]
    fn parses_stdin_clip_source() {
        let env = EnvMap::from([
            ("CLIPTOWN_COMMAND".into(), "clip add".into()),
            ("CLIPTOWN_POSITIONALS".into(), "[]".into()),
            ("CLIPTOWN_STDIN".into(), "true".into()),
        ]);
        assert_eq!(
            Command::from_env_map(&env).unwrap(),
            Command::ClipAdd {
                file: None,
                from_stdin: true,
                from_clipboard: false,
                pin: false,
            }
        );
    }
}
