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
    /// Authenticate with the Cliptown ecosystem
    #[command(alias = "login")]
    #[command(alias = "signin")]
    Auth {
        #[command(subcommand)]
        action: AuthAction,
    },
    
    /// Get items from the ecosystem
    Get {
        #[command(subcommand)]
        resource: GetResource,
    },
}

#[derive(Subcommand)]
enum AuthAction {
    /// Log in to Cliptown
    #[command(alias = "signin")]
    Login,
    
    /// Log out of Cliptown
    #[command(alias = "logout")]
    #[command(alias = "signout")]
    Signout,
}

#[derive(Subcommand)]
enum GetResource {
    /// Retrieve clips
    Clips {
        /// Retrieve all clipboard items
        #[arg(short, long)]
        all: bool,
        
        /// JSON string filter query to match clips
        #[arg(short, long)]
        filter: Option<String>,
    }
}

#[tokio::main]
async fn main() {
    // Flags-to-env conceptually parses .cli-flags.toml here, mapping args to ENV vars.
    let cli = Cli::parse();

    match &cli.command {
        Commands::Auth { action } => {
            match action {
                AuthAction::Login => {
                    println!("Initiating Cliptown authentication...");
                    // TODO: trigger auth flow
                }
                AuthAction::Signout => {
                    println!("Signing out of Cliptown...");
                    // TODO: revoke auth tokens
                }
            }
        }
        Commands::Get { resource } => {
            match resource {
                GetResource::Clips { all, filter } => {
                    if *all {
                        println!("Fetching ALL clips...");
                    } else if let Some(f) = filter {
                        println!("Fetching clips with filter: {}", f);
                    } else {
                        println!("Fetching default recent clips limit...");
                    }
                }
            }
        }
    }
}
