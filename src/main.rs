mod command;
mod config;
mod error;

use command::Command;
use config::RuntimeConfig;

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        error.report(RuntimeConfig::output_json_requested());
        std::process::exit(error.exit_code());
    }
}

async fn run() -> Result<(), error::CliError> {
    let config = RuntimeConfig::from_env()?;
    let command = Command::from_env()?;
    command.execute(config).await
}
