pub mod admin;
pub mod collection;
pub mod delete;
pub mod keygen;
pub mod list;
pub mod upload;

use crate::{config::CliConfig, output::OutputFormat};
use clap::{Args, Subcommand};

/// Admin sub-actions.
#[derive(Debug, Subcommand)]
pub enum AdminAction {
    /// Add a new authorised user
    Add {
        /// Username for the new user
        username: String,
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
        /// Username identifying the user to update
        username: String,
        /// New display name
        #[arg(long)]
        name: Option<String>,
        /// Set or clear admin privileges (true/false)
        #[arg(long)]
        admin: Option<bool>,
    },
    /// Remove a user
    Remove {
        /// Username identifying the user to remove
        username: String,
    },
}

/// Arguments for the `upload` subcommand.
#[derive(Debug, Args)]
pub struct UploadArgs {
    /// Path to the file to upload
    pub file: String,
    /// Publish as a new version of an existing UID
    #[arg(long)]
    pub uid: Option<String>,
    /// Title metadata
    #[arg(long)]
    pub title: Option<String>,
    /// Description metadata
    #[arg(long)]
    pub description: Option<String>,
    /// Tag (repeatable: --tags foo --tags bar)
    #[arg(long)]
    pub tags: Vec<String>,
    /// Agent model
    #[arg(long)]
    pub agent_model: Option<String>,
    /// Agent provider
    #[arg(long)]
    pub agent_provider: Option<String>,
    /// Agent tool
    #[arg(long)]
    pub agent_tool: Option<String>,
    /// Trigger context
    #[arg(long)]
    pub trigger: Option<String>,
    /// Custom metadata KEY=VALUE (repeatable: --meta sprint=42 --meta reviewer=alice)
    #[arg(long)]
    pub meta: Vec<String>,
    /// Collection to assign this upload to
    #[arg(long)]
    pub collection: Option<String>,
    /// Expiry in days
    #[arg(long)]
    pub expiry: Option<u64>,
}

/// Collection sub-actions.
#[derive(Debug, Subcommand)]
pub enum CollectionAction {
    /// Add a file to a collection
    Add {
        /// Collection name
        name: String,
        /// Upload UID to add
        uid: String,
    },
    /// Remove a file from a collection
    Remove {
        /// Collection name
        name: String,
        /// Upload UID to remove
        uid: String,
    },
}

/// Top-level CLI subcommands.
#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Upload a file and receive a public URL
    Upload(Box<UploadArgs>),
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
    /// Manage collection membership
    Collection {
        #[command(subcommand)]
        action: CollectionAction,
    },
}

impl Commands {
    /// Execute the subcommand.
    pub async fn execute(&self, config: &CliConfig, format: &OutputFormat) -> anyhow::Result<()> {
        match self {
            Commands::Upload(args) => {
                let opts = upload::UploadOptions {
                    uid: args.uid.as_deref(),
                    title: args.title.as_deref(),
                    description: args.description.as_deref(),
                    tags: &args.tags,
                    agent_model: args.agent_model.as_deref(),
                    agent_provider: args.agent_provider.as_deref(),
                    agent_tool: args.agent_tool.as_deref(),
                    trigger: args.trigger.as_deref(),
                    meta: &args.meta,
                    collection: args.collection.as_deref(),
                    expiry: args.expiry,
                };
                upload::execute(&args.file, &opts, config, format).await?;
            }
            Commands::List { collection } => {
                list::execute(collection.as_deref(), config, format).await?;
            }
            Commands::Delete { uid, version } => match version {
                Some(v) => delete::execute(uid, *v, config, format).await?,
                None => anyhow::bail!(
                    "Version number is required. Use --version N to specify the version to delete."
                ),
            },
            Commands::Keygen { force } => {
                keygen::execute(*force, format).await?;
            }
            Commands::Admin { action } => match action {
                AdminAction::Add {
                    username,
                    public_key,
                    name,
                    admin,
                } => {
                    admin::execute_add(
                        username,
                        public_key,
                        name.as_deref(),
                        *admin,
                        config,
                        format,
                    )
                    .await?;
                }
                AdminAction::Update {
                    username,
                    name,
                    admin,
                } => {
                    admin::execute_update(username, name.as_deref(), *admin, config, format)
                        .await?;
                }
                AdminAction::Remove { username } => {
                    admin::execute_remove(username, config, format).await?;
                }
            },
            Commands::Collection { action } => match action {
                CollectionAction::Add { name, uid } => {
                    collection::execute_add(name, uid, config, format).await?;
                }
                CollectionAction::Remove { name, uid } => {
                    collection::execute_remove(name, uid, config, format).await?;
                }
            },
        }
        Ok(())
    }
}
