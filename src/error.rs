#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("configuration: {0}")]
    Configuration(String),
    #[error("command-line parsing: {0}")]
    Parsing(String),
    #[error("clipboard: {0}")]
    Clipboard(String),
    #[error("I/O: {0}")]
    Io(#[from] std::io::Error),
    #[error("client: {0}")]
    Client(#[from] cliptown_client_rust::ClientError),
}

impl CliError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::Parsing(_) => "invalid_arguments",
            Self::Configuration(_) => "invalid_configuration",
            Self::Clipboard(_) => "clipboard_unavailable",
            Self::Io(_) => "io_error",
            Self::Client(_) => "client_error",
        }
    }

    pub fn exit_code(&self) -> i32 {
        match self {
            Self::Parsing(_) => 2,
            Self::Configuration(_) => 3,
            Self::Clipboard(_) => 4,
            Self::Io(_) => 5,
            Self::Client(_) => 6,
        }
    }

    pub fn report(&self, json: bool) {
        match json {
            true => eprintln!(
                "{}",
                serde_json::json!({
                    "schema_version": 1,
                    "ok": false,
                    "error": {
                        "code": self.code(),
                        "message": self.to_string(),
                    }
                })
            ),
            false => eprintln!("cliptown: {self}"),
        }
    }
}
