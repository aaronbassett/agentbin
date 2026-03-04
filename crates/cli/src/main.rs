#![deny(unsafe_code)]
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

/// Map an error to a CLI exit code per the spec:
/// 0 = success, 1 = general, 2 = auth, 3 = not-found, 4 = validation, 5 = connection
fn exit_code_for_error(err: &anyhow::Error) -> i32 {
    let msg = err.to_string().to_lowercase();

    if msg.contains("failed to connect")
        || msg.contains("connection refused")
        || msg.contains("dns error")
    {
        5 // connection
    } else if msg.contains("status: 401")
        || msg.contains("status: 403")
        || msg.contains("key") && msg.contains("not found")
    {
        2 // auth
    } else if msg.contains("status: 404") || msg.contains("not found") {
        3 // not-found
    } else if msg.contains("status: 400")
        || msg.contains("status: 422")
        || msg.contains("validation")
    {
        4 // validation
    } else {
        1 // general
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let config = config::CliConfig::load(cli.server_url.as_deref(), cli.key_file.as_deref());
    let format = if cli.json {
        OutputFormat::Json
    } else {
        OutputFormat::Human
    };

    if let Err(e) = cli.command.execute(&config, &format).await {
        eprintln!("{}", output::format_error(&format, &e.to_string()));
        std::process::exit(exit_code_for_error(&e));
    }
}
