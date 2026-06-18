use anyhow::Result;
use clap::{Parser, Subcommand};

mod commands;
mod config;
mod crypto;
mod registry;
mod storage;

#[derive(Parser)]
#[command(name = "rust-messenger")]
#[command(about = "A privacy-first messaging identity client", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate identity keys and initialize local profile
    Init {
        /// Overwrite existing identity
        #[arg(long, short)]
        force: bool,
    },
    /// Display local identity information
    Whoami,
    /// Register a username with the identity registry
    Register {
        /// The username to register
        username: String,
    },
    /// Search for users in the registry
    Search {
        /// The query to search for
        query: String,
    },
    /// Look up a specific user's information
    Lookup {
        /// The username to look up
        username: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { force } => {
            commands::init::exec(force)?;
        }
        Commands::Whoami => {
            commands::whoami::exec()?;
        }
        Commands::Register { username } => {
            commands::register::exec(&username).await?;
        }
        Commands::Search { query } => {
            commands::search::exec(&query).await?;
        }
        Commands::Lookup { username } => {
            commands::lookup::exec(&username).await?;
        }
    }

    Ok(())
}
