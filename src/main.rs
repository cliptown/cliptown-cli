mod command;
mod config;
mod error;

use command::Command;
use config::RuntimeConfig;

const HELP: &str = "cliptown 0.1.0\n\nUsage: cliptown [global options] <command>\n\nCommands:\n  auth login|status|logout\n  clip list|get|add|pin|unpin|delete|copy|search\n  sync pull|push|status|pair\n  config get|set\n  doctor\n\nOptions:\n  -h, --help       Print this help\n  -V, --version    Print the CLI version\n\nDetailed flag help is generated from .cli-flags.toml.\n";
const VERSION: &str = concat!("cliptown ", env!("CARGO_PKG_VERSION"), "\n");

fn informational_output<I, S>(arguments: I) -> Option<&'static str>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    arguments
        .into_iter()
        .find_map(|argument| match argument.as_ref() {
            "-h" | "--help" => Some(HELP),
            "-V" | "--version" => Some(VERSION),
            _ => None,
        })
}

#[tokio::main]
async fn main() {
    if let Some(output) = informational_output(std::env::args().skip(1)) {
        print!("{output}");
        return;
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_and_version_are_available_without_configuration_or_network() {
        assert_eq!(informational_output(["--help"]), Some(HELP));
        assert_eq!(informational_output(["-h"]), Some(HELP));
        assert_eq!(informational_output(["--version"]), Some(VERSION));
        assert_eq!(informational_output(["doctor"]), None);
    }
}
