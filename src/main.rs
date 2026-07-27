mod command;
mod config;
mod error;

use command::Command;
use config::RuntimeConfig;

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("cliptown: {error}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), error::CliError> {
    let config = RuntimeConfig::from_env()?;
    let command = Command::from_env()?;
    command.execute(config).await
}
