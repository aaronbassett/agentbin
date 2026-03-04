pub mod keygen;

use crate::{config::CliConfig, output::OutputFormat};
use clap::Subcommand;

/// Admin sub-actions.
#[derive(Debug, Subcommand)]
pub enum AdminAction {
    /// Add a new authorised user
    Add {
        /// Ed25519 public key (base64)
        public_key: String,
        /// Optional display name
        #[arg(long)]
        name: Option<String>,
        /// Grant admin privileges
        #[arg(long)]
        admin: bool,
    },
    /// Update an existing user's attributes
    Update {
        /// Ed25519 public key (base64) identifying the user
        public_key: String,
        /// New display name
        #[arg(long)]
        name: Option<String>,
        /// Set or clear admin privileges (true/false)
        #[arg(long)]
        admin: Option<bool>,
    },
    /// Remove a user
    Remove {
        /// Ed25519 public key (base64) identifying the user
        public_key: String,
    },
}

/// Top-level CLI subcommands.
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Upload a file and receive a public URL
    Upload {
        /// Path to the file to upload
        file: String,
        /// Publish as a new version of an existing UID
        #[arg(long)]
        uid: Option<String>,
        /// Optional title for the upload
        #[arg(long)]
        title: Option<String>,
        /// Comma-separated tags
        #[arg(long)]
        tags: Option<String>,
        /// Collection to add this upload to
        #[arg(long)]
        collection: Option<String>,
        /// TTL in seconds before the upload expires
        #[arg(long)]
        ttl: Option<u64>,
    },
    /// List your uploads
    List {
        /// Filter by collection name
        #[arg(long)]
        collection: Option<String>,
    },
    /// Delete a specific version (or entire upload)
    Delete {
        /// Upload UID
        uid: String,
        /// Version number to delete
        #[arg(long)]
        version: Option<u32>,
    },
    /// Generate a new Ed25519 key pair and save to the key file
    Keygen {
        /// Overwrite an existing key file without prompting
        #[arg(long)]
        force: bool,
    },
    /// Admin user management (requires admin key)
    Admin {
        #[command(subcommand)]
        action: AdminAction,
    },
}

impl Commands {
    /// Execute the subcommand. All variants are stubs pending full implementation.
    pub async fn execute(&self, _config: &CliConfig, _format: &OutputFormat) -> anyhow::Result<()> {
        match self {
            Commands::Upload { .. } => {
                println!("not yet implemented: upload");
            }
            Commands::List { .. } => {
                println!("not yet implemented: list");
            }
            Commands::Delete { .. } => {
                println!("not yet implemented: delete");
            }
            Commands::Keygen { force } => {
                keygen::execute(*force, _format).await?;
            }
            Commands::Admin { action } => match action {
                AdminAction::Add { .. } => {
                    println!("not yet implemented: admin add");
                }
                AdminAction::Update { .. } => {
                    println!("not yet implemented: admin update");
                }
                AdminAction::Remove { .. } => {
                    println!("not yet implemented: admin remove");
                }
            },
        }
        Ok(())
    }
}
