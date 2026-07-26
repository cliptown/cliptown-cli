use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "cliptown")]
#[command(about = "CLI for Cliptown, the secure, cross-platform clipboard manager", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Authenticate with your 6-digit PIN
    Login {
        #[arg(short, long)]
        pin: String,
    },
    /// Sync the clipboard
    Sync,
    /// List recent clipboard items
    List {
        #[arg(short, long, default_value_t = 10)]
        limit: usize,
    },
    /// Push text to the clipboard
    Push {
        #[arg(short, long)]
        text: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Login { pin } => {
            println!("Logging in with PIN: {}", pin);
            // TODO: Implement login logic using cliptown-client-rust
        }
        Commands::Sync => {
            println!("Syncing clipboard items...");
            // TODO: Implement sync logic using cliptown-client-rust
        }
        Commands::List { limit } => {
            println!("Listing the last {} clipboard items...", limit);
            // TODO: Fetch items
        }
        Commands::Push { text } => {
            println!("Pushing '{}' to clipboard securely...", text);
            // TODO: Encrypt and push
        }
    }
}
