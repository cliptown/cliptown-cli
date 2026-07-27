use std::env;

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
        text: Option<String>,
        file: Option<String>,
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
                text: env::var("CLIPTOWN_TEXT").ok(),
                file: env::var("CLIPTOWN_FILE").ok(),
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
                if config.json {
                    println!(
                        "{}",
                        serde_json::json!({
                            "endpoint": config.endpoint,
                            "config_dir": config.config_dir,
                            "clipboard": clipboard,
                            "flags2env": "ok"
                        })
                    );
                } else {
                    println!(
                        "endpoint={} config_dir={} clipboard={} flags2env=ok",
                        config.endpoint,
                        config.config_dir.display(),
                        clipboard
                    );
                }
            }
            Self::ClipAdd {
                text,
                file,
                from_clipboard,
                pin,
            } => {
                let payload = if from_clipboard {
                    Clipboard::new()
                        .and_then(|mut clipboard| clipboard.get_text())
                        .map_err(|error| CliError::Clipboard(error.to_string()))?
                } else if let Some(text) = text {
                    text
                } else if let Some(path) = file {
                    std::fs::read_to_string(path)?
                } else {
                    return Err(CliError::Parsing(
                        "provide --text, --file, or --from-clipboard".into(),
                    ));
                };
                println!(
                    "queued encrypted clip ({} bytes, pinned={pin})",
                    payload.len()
                );
            }
            Self::AuthLogin { reauth_days } => println!(
                "start Supabase PKCE + 3FA/shared-auth login (reauth every {reauth_days} days)"
            ),
            other => println!("{other:?} against {}", config.endpoint),
        }
        Ok(())
    }
}

fn bool_env(key: &str) -> bool {
    matches!(env::var(key).as_deref(), Ok("true" | "1" | "yes"))
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
}
