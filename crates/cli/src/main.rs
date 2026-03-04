#![deny(unsafe_code)]
// Skeleton modules — infrastructure will be fully used once commands are implemented.
#![allow(dead_code)]

mod commands;
mod config;
mod output;
mod signing;

use clap::Parser;
use output::OutputFormat;

#[derive(Parser)]
#[command(name = "agentbin", about = "Share rendered documents from AI agents")]
struct Cli {
    /// Output results as JSON
    #[arg(long, global = true)]
    json: bool,

    /// Server URL
    #[arg(long, global = true, env = "AGENTBIN_SERVER_URL")]
    server_url: Option<String>,

    /// Path to the Ed25519 private key file
    #[arg(long, global = true, env = "AGENTBIN_KEY_FILE")]
    key_file: Option<String>,

    #[command(subcommand)]
    command: commands::Commands,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let config = config::CliConfig::load(cli.server_url.as_deref(), cli.key_file.as_deref());
    let format = if cli.json {
        OutputFormat::Json
    } else {
        OutputFormat::Human
    };
    cli.command.execute(&config, &format).await
}
