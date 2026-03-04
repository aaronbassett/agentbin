use anyhow::Context;
use serde_json::json;

use crate::{config::CliConfig, output::OutputFormat, signing};

/// Execute `admin add` — add a new authorised user via the server API.
pub async fn execute_add(
    username: &str,
    public_key: &str,
    display_name: Option<&str>,
    is_admin: bool,
    config: &CliConfig,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    let private_key = config.read_private_key()?;
    let api_path = "/api/admin/users";
    let url = format!("{}{api_path}", config.server_url);

    let body = json!({
        "username": username,
        "public_key": public_key,
        "display_name": display_name,
        "is_admin": is_admin,
    });
    let body_bytes = serde_json::to_vec(&body).context("Failed to serialise request body")?;

    let signed = signing::sign_http_request(&private_key, "POST", api_path, &body_bytes)?;

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
            let uname = resp_json
                .get("username")
                .and_then(|v| v.as_str())
                .unwrap_or(username);
            let admin_flag = resp_json
                .get("is_admin")
                .and_then(|v| v.as_bool())
                .unwrap_or(is_admin);
            println!("User added: {uname} (admin: {admin_flag})");
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

/// Execute `admin update` — update an existing user via the server API.
pub async fn execute_update(
    username: &str,
    display_name: Option<&str>,
    is_admin: Option<bool>,
    config: &CliConfig,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    let private_key = config.read_private_key()?;
    let api_path = format!("/api/admin/users/{username}");
    let url = format!("{}{api_path}", config.server_url);

    let mut body = serde_json::Map::new();
    if let Some(name) = display_name {
        body.insert("display_name".to_string(), json!(name));
    }
    if let Some(admin) = is_admin {
        body.insert("is_admin".to_string(), json!(admin));
    }
    let body_value = serde_json::Value::Object(body);
    let body_bytes = serde_json::to_vec(&body_value).context("Failed to serialise request body")?;

    let signed = signing::sign_http_request(&private_key, "PUT", &api_path, &body_bytes)?;

    let client = reqwest::Client::new();
    let builder = client
        .put(&url)
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
            let uname = resp_json
                .get("username")
                .and_then(|v| v.as_str())
                .unwrap_or(username);
            let admin_flag = resp_json
                .get("is_admin")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            println!("User updated: {uname} (admin: {admin_flag})");
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

/// Execute `admin remove` — remove a user via the server API.
pub async fn execute_remove(
    username: &str,
    config: &CliConfig,
    format: &OutputFormat,
) -> anyhow::Result<()> {
    let private_key = config.read_private_key()?;
    let api_path = format!("/api/admin/users/{username}");
    let url = format!("{}{api_path}", config.server_url);

    // DELETE with an empty body.
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
            println!("User removed: {username}");
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

/// Extract an error message from the API error response body.
///
/// Handles both the structured `{"error": {"message": "..."}}` envelope and
/// the flat `{"message": "..."}` variant.
fn extract_error_message(resp_json: &serde_json::Value) -> &str {
    resp_json
        .get("error")
        .and_then(|e| e.get("message"))
        .and_then(|v| v.as_str())
        .or_else(|| resp_json.get("message").and_then(|v| v.as_str()))
        .unwrap_or("unknown server error")
}
