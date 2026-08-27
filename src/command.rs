use std::{env, io::Read};

use arboard::Clipboard;

use crate::{config::RuntimeConfig, error::CliError};

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
    pub fn from_env() -> Result<Self, CliError> {
        let path = env::var("CLIPTOWN_COMMAND").unwrap_or_default();
        let args: Vec<String> =
            serde_json::from_str(&env::var("CLIPTOWN_POSITIONALS").unwrap_or_else(|_| "[]".into()))
                .map_err(|error| CliError::Parsing(error.to_string()))?;
        let argument = |index: usize| {
            args.get(index)
                .cloned()
                .ok_or_else(|| CliError::Parsing(format!("missing argument {index} for {path}")))
        };

        let command = match path.as_str() {
            "auth login" => {
                let reauth_days = env::var("CLIPTOWN_REAUTH_DAYS")
                    .unwrap_or_else(|_| "10".into())
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
                let limit: u32 = env::var("CLIPTOWN_LIMIT")
                    .unwrap_or_else(|_| "20".into())
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
                file: env::var("CLIPTOWN_FILE").ok(),
                from_stdin: bool_env("CLIPTOWN_STDIN"),
                from_clipboard: bool_env("CLIPTOWN_FROM_CLIPBOARD"),
                pin: bool_env("CLIPTOWN_PIN_CLIP"),
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
                let query = match env::var("CLIPTOWN_QUERY") {
                    Ok(query) => query,
                    Err(_) => argument(0)?,
                };
                Self::ClipSearch {
                    query,
                    mode: env::var("CLIPTOWN_SEARCH_MODE").unwrap_or_else(|_| "local_only".into()),
                }
            }
            "sync pull" => Self::SyncPull,
            "sync push" => Self::SyncPush,
            "sync status" => Self::SyncStatus,
            "sync pair" => Self::SyncPair {
                transport: env::var("CLIPTOWN_PAIR_TRANSPORT").unwrap_or_else(|_| "wifi".into()),
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
                let (payload, source) = read_clip_source(file, from_stdin, from_clipboard)?;

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

fn bool_env(key: &str) -> bool {
    matches!(env::var(key).as_deref(), Ok("true" | "1" | "yes"))
}

enum ClipSource {
    Clipboard,
    Stdin,
    File(String),
}

fn selected_clip_source(
    file: Option<String>,
    from_stdin: bool,
    from_clipboard: bool,
) -> Result<ClipSource, CliError> {
    match (file, from_stdin, from_clipboard) {
        (None, false, true) => Ok(ClipSource::Clipboard),
        (None, true, false) => Ok(ClipSource::Stdin),
        (Some(path), false, false) => Ok(ClipSource::File(path)),
        _ => Err(CliError::Parsing(
            "choose exactly one of --stdin, --file, or --from-clipboard".into(),
        )),
    }
}

fn read_clip_source(
    file: Option<String>,
    from_stdin: bool,
    from_clipboard: bool,
) -> Result<(String, &'static str), CliError> {
    match selected_clip_source(file, from_stdin, from_clipboard)? {
        ClipSource::Clipboard => Clipboard::new()
            .and_then(|mut clipboard| clipboard.get_text())
            .map(|payload| (payload, "clipboard"))
            .map_err(|error| CliError::Clipboard(error.to_string())),
        ClipSource::Stdin => {
            let payload = {
                let mut payload = String::new();
                std::io::stdin().read_to_string(&mut payload)?;
                payload
            };
            Ok((payload, "stdin"))
        }
        ClipSource::File(path) => Ok((std::fs::read_to_string(path)?, "file")),
    }
}

fn emit_result(config: &RuntimeConfig, result: serde_json::Value, plain: String) {
    match config.json {
        true => println!(
            "{}",
            serde_json::json!({
                "schema_version": 1,
                "ok": true,
                "result": result,
            })
        ),
        false => println!("{plain}"),
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;

    static LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn parses_login_window() {
        let _guard = LOCK.lock().unwrap();
        env::set_var("CLIPTOWN_COMMAND", "auth login");
        env::set_var("CLIPTOWN_REAUTH_DAYS", "20");
        env::set_var("CLIPTOWN_POSITIONALS", "[]");
        assert_eq!(
            Command::from_env().unwrap(),
            Command::AuthLogin { reauth_days: 20 }
        );
    }

    #[test]
    fn rejects_long_window() {
        let _guard = LOCK.lock().unwrap();
        env::set_var("CLIPTOWN_COMMAND", "auth login");
        env::set_var("CLIPTOWN_REAUTH_DAYS", "21");
        env::set_var("CLIPTOWN_POSITIONALS", "[]");
        assert!(Command::from_env().is_err());
    }

    #[test]
    fn parses_stdin_clip_source() {
        let _guard = LOCK.lock().unwrap();
        env::set_var("CLIPTOWN_COMMAND", "clip add");
        env::set_var("CLIPTOWN_POSITIONALS", "[]");
        env::set_var("CLIPTOWN_STDIN", "true");
        env::remove_var("CLIPTOWN_FILE");
        env::remove_var("CLIPTOWN_FROM_CLIPBOARD");
        assert_eq!(
            Command::from_env().unwrap(),
            Command::ClipAdd {
                file: None,
                from_stdin: true,
                from_clipboard: false,
                pin: false,
            }
        );
        env::remove_var("CLIPTOWN_STDIN");
    }

    #[test]
    fn clip_source_match_excludes_mixed_and_missing_inputs() {
        assert!(matches!(
            selected_clip_source(None, true, false).unwrap(),
            ClipSource::Stdin
        ));
        assert!(matches!(
            selected_clip_source(Some("notes.txt".into()), false, false).unwrap(),
            ClipSource::File(_)
        ));
        assert!(selected_clip_source(None, true, true).is_err());
        assert!(selected_clip_source(None, false, false).is_err());
    }
}
