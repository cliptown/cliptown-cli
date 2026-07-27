#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error("configuration: {0}")]
    Configuration(String),
    #[error("command-line parsing: {0}")]
    Parsing(String),
    #[error("authentication: {0}")]
    Authentication(String),
    #[error("clipboard: {0}")]
    Clipboard(String),
    #[error("I/O: {0}")]
    Io(#[from] std::io::Error),
    #[error("client: {0}")]
    Client(#[from] cliptown_client_rust::ClientError),
}
