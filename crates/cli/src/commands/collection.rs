use anyhow::Context;
use serde_json::json;

use crate::{config::CliConfig, output::OutputFormat, signing};

fn extract_error_message(resp_json: &serde_json::Value) -> &str {
    resp_json
        .get("error")
        .and_then(|e| e.get("message"))
        .and_then(|v| v.as_str())
        .or_else(|| resp_json.get("message").and_then(|v| v.as_str()))
        .unwrap_or("unknown server error")
}

/// Execute `collection add` — add a file to a collection via the server API.
pub async fn execute_add(
    name: &str,
    uid: &str,
    config: &CliConfig,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    let private_key = config.read_private_key()?;
    let api_path = format!("/api/collections/{name}/members");
    let url = format!("{}{api_path}", config.server_url);

    let body = json!({ "uid": uid });
    let body_bytes = serde_json::to_vec(&body).context("Failed to serialise request body")?;

    let signed = signing::sign_http_request(&private_key, "POST", &api_path, &body_bytes)?;

    let client = reqwest::Client::new();
    let builder = client
        .post(&url)
        .header("Content-Type", "application/json")
        .body(body_bytes);
    let builder = signing::apply_auth_headers(builder, &signed);

    let response = builder
        .send()
        .await
        .with_context(|| format!("Failed to connect to server at {url}"))?;

    let status = response.status();
    let resp_json: serde_json::Value = response
        .json()
        .await
        .context("Failed to parse server response as JSON")?;

    if !status.is_success() {
        let message = extract_error_message(&resp_json);
        anyhow::bail!("Server error {status}: {message}");
    }

    match format {
        OutputFormat::Human => {
            println!("Added {uid} to collection '{name}'");
        }
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&resp_json).context("Failed to format response")?
            );
        }
    }

    Ok(())
}

/// Execute `collection remove` — remove a file from a collection via the server API.
pub async fn execute_remove(
    name: &str,
    uid: &str,
    config: &CliConfig,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    let private_key = config.read_private_key()?;
    let api_path = format!("/api/collections/{name}/members/{uid}");
    let url = format!("{}{api_path}", config.server_url);

    let signed = signing::sign_http_request(&private_key, "DELETE", &api_path, &[])?;

    let client = reqwest::Client::new();
    let builder = client.delete(&url);
    let builder = signing::apply_auth_headers(builder, &signed);

    let response = builder
        .send()
        .await
        .with_context(|| format!("Failed to connect to server at {url}"))?;

    let status = response.status();
    let resp_json: serde_json::Value = response
        .json()
        .await
        .context("Failed to parse server response as JSON")?;

    if !status.is_success() {
        let message = extract_error_message(&resp_json);
        anyhow::bail!("Server error {status}: {message}");
    }

    match format {
        OutputFormat::Human => {
            let collection_deleted = resp_json
                .get("collection_deleted")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if collection_deleted {
                println!("Removed {uid} from collection '{name}' (collection is now empty and was deleted)");
            } else {
                println!("Removed {uid} from collection '{name}'");
            }
        }
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&resp_json).context("Failed to format response")?
            );
        }
    }

    Ok(())
}
