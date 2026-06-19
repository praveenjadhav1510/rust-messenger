#![allow(dead_code, unused_imports)]
use anyhow::Result;
use clap::{Parser, Subcommand};

mod chat;
mod commands;
mod config;
mod connection;
mod contacts;
mod crypto;
mod handshake;
mod ice;
mod network;
mod peer;
mod presence;
mod protocol;
mod registry;
mod secure;
mod session;
mod storage;
mod transport;

#[derive(Parser)]
#[command(name = "rust-messenger")]
#[command(about = "A privacy-first messaging identity client", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum ContactsCommands {
    /// Add a contact from the registry
    Add {
        /// The username to add
        username: String,
    },
    /// Remove a contact locally
    Remove {
        /// The username to remove
        username: String,
    },
    /// List all local contacts
    List,
    /// Show detailed info for a contact
    Show {
        /// The username to show
        username: String,
    },
}

#[derive(Subcommand)]
enum RequestsCommands {
    /// List all local message requests
    List,
    /// Accept a pending message request
    Accept {
        /// The username to accept
        username: String,
    },
    /// Reject a pending message request
    Reject {
        /// The username to reject
        username: String,
    },
}

#[derive(Subcommand)]
enum MessageCommands {
    /// Send a message to a contact
    Send {
        /// The contact username
        username: String,
        /// The message body
        text: String,
    },
    /// Display message history for a contact
    History {
        /// The contact username
        username: String,
    },
    /// List all active conversations
    List,
    /// Clear conversation history for a contact
    Clear {
        /// The contact username
        username: String,
    },
}

#[derive(Subcommand)]
enum ConversationCommands {
    /// Show detailed metadata for a conversation
    Show {
        /// The contact username
        username: String,
    },
}

#[derive(Subcommand)]
enum DevCommands {
    /// Inject a local incoming message for testing
    Inject {
        /// The sender username
        username: String,
        /// The message body
        text: String,
    },
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
    /// Recover a username with a recovery code
    Recover {
        /// The username to recover
        username: String,
    },
    /// Rename a registered identity
    Rename {
        /// The new username
        new_username: String,
    },
    /// Deactivate/remove the current identity
    Remove,
    /// Restore a deactivated identity
    Restore {
        /// The username to restore
        username: String,
    },
    /// Lock the current identity
    Lock,
    /// Unlock the current identity
    Unlock,
    /// Manage contacts locally
    Contacts {
        #[command(subcommand)]
        command: ContactsCommands,
    },
    /// Verify a contact's identity locally
    Verify {
        /// The username to verify
        username: String,
    },
    /// Unverify a contact locally
    Unverify {
        /// The username to unverify
        username: String,
    },
    /// Block a contact locally
    Block {
        /// The username to block
        username: String,
    },
    /// Unblock a contact locally
    Unblock {
        /// The username to unblock
        username: String,
    },
    /// Manage message requests locally
    Requests {
        #[command(subcommand)]
        command: RequestsCommands,
    },
    /// Manage messages locally
    Message {
        #[command(subcommand)]
        command: MessageCommands,
    },
    /// Manage conversations locally
    Conversation {
        #[command(subcommand)]
        command: ConversationCommands,
    },
    /// Announce that the identity is online
    Online,
    /// Mark the local session and registry status offline
    Offline,
    /// Query the registry for a user's presence status
    Status {
        /// The username to query
        username: String,
    },
    /// Connect to a peer
    Connect {
        /// The username to connect to
        username: String,
    },
    /// Disconnect from a peer
    Disconnect {
        /// The username to disconnect from
        username: String,
    },
    /// List all active peer sessions
    Peers,
    /// Ping a connected peer
    Ping {
        /// The username to ping
        username: String,
    },
    /// Discover NAT type and local network interface details
    Netinfo,
    /// List prioritized ICE candidate pairs for a peer
    CandidatePairs {
        /// The username of the peer
        username: String,
    },
    /// Execute ICE connectivity checks with a peer
    IceCheck {
        /// The username of the peer
        username: String,
    },
    /// Discover peer details (online status, capabilities, and candidates)
    Discover {
        /// The username of the peer to discover
        username: String,
    },
    /// Negotiate a connection with a peer contact
    Negotiate {
        /// The username of the peer to negotiate with
        username: String,
    },
    /// Display local client capabilities
    Capabilities,
    /// Establish a secure encrypted session with a peer
    SecureSession {
        /// The username of the peer
        username: String,
    },
    /// Test local UDP transport functionality
    TestUdp,
    /// Developer utilities for local simulation
    Dev {
        #[command(subcommand)]
        command: DevCommands,
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
        Commands::Recover { username } => {
            commands::recover::exec(&username).await?;
        }
        Commands::Rename { new_username } => {
            commands::rename::exec(&new_username).await?;
        }
        Commands::Remove => {
            commands::remove::exec().await?;
        }
        Commands::Restore { username } => {
            commands::restore::exec(&username).await?;
        }
        Commands::Lock => {
            commands::lock::exec().await?;
        }
        Commands::Unlock => {
            commands::unlock::exec().await?;
        }
        Commands::Contacts { command } => match command {
            ContactsCommands::Add { username } => {
                commands::contacts::exec_add(&username).await?;
            }
            ContactsCommands::Remove { username } => {
                commands::contacts::exec_remove(&username)?;
            }
            ContactsCommands::List => {
                commands::contacts::exec_list()?;
            }
            ContactsCommands::Show { username } => {
                commands::contacts::exec_show(&username)?;
            }
        },
        Commands::Verify { username } => {
            commands::verify::exec_verify(&username)?;
        }
        Commands::Unverify { username } => {
            commands::verify::exec_unverify(&username)?;
        }
        Commands::Block { username } => {
            commands::block::exec_block(&username)?;
        }
        Commands::Unblock { username } => {
            commands::block::exec_unblock(&username)?;
        }
        Commands::Requests { command } => match command {
            RequestsCommands::List => {
                commands::requests::exec_list()?;
            }
            RequestsCommands::Accept { username } => {
                commands::requests::exec_accept(&username)?;
            }
            RequestsCommands::Reject { username } => {
                commands::requests::exec_reject(&username)?;
            }
        },
        Commands::Message { command } => match command {
            MessageCommands::Send { username, text } => {
                commands::message::exec_send(&username, &text)?;
            }
            MessageCommands::History { username } => {
                commands::message::exec_history(&username)?;
            }
            MessageCommands::List => {
                commands::message::exec_list()?;
            }
            MessageCommands::Clear { username } => {
                commands::message::exec_clear(&username)?;
            }
        },
        Commands::Conversation { command } => match command {
            ConversationCommands::Show { username } => {
                commands::conversation::exec_show(&username)?;
            }
        },
        Commands::Online => {
            commands::online::exec().await?;
        }
        Commands::Offline => {
            commands::offline::exec().await?;
        }
        Commands::Status { username } => {
            commands::status::exec(&username).await?;
        }
        Commands::Connect { username } => {
            commands::connect::exec(&username).await?;
        }
        Commands::Disconnect { username } => {
            commands::disconnect::exec(&username)?;
        }
        Commands::Peers => {
            commands::peers::exec()?;
        }
        Commands::Ping { username } => {
            commands::ping::exec(&username).await?;
        }
        Commands::Netinfo => {
            commands::netinfo::exec().await?;
        }
        Commands::CandidatePairs { username } => {
            commands::candidate_pairs::exec(&username).await?;
        }
        Commands::IceCheck { username } => {
            commands::ice_check::exec(&username).await?;
        }
        Commands::Discover { username } => {
            commands::discover::exec(&username).await?;
        }
        Commands::Negotiate { username } => {
            commands::negotiate::exec(&username).await?;
        }
        Commands::Capabilities => {
            commands::capabilities::exec()?;
        }
        Commands::SecureSession { username } => {
            commands::secure_session::exec(&username).await?;
        }
        Commands::TestUdp => {
            commands::test_udp::exec()?;
        }
        Commands::Dev { command } => match command {
            DevCommands::Inject { username, text } => {
                commands::dev::exec_inject(&username, &text)?;
            }
        },
    }

    Ok(())
}
