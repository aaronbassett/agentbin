use anyhow::Context;

/// Execute the delete command.
///
/// Sends an authenticated `DELETE /api/uploads/{uid}/v{version}` request and
/// displays the result.
pub async fn execute(
    uid: &str,
    version: u32,
    config: &crate::config::CliConfig,
    format: &crate::output::OutputFormat,
) -> anyhow::Result<()> {
    let private_key = config.read_private_key()?;
    let api_path = format!("/api/uploads/{uid}/v{version}");
    let url = format!("{}{}", config.server_url, api_path);

    let signed = crate::signing::sign_http_request(&private_key, "DELETE", &api_path, &[])?;

    let client = reqwest::Client::new();
    let builder = crate::signing::apply_auth_headers(client.delete(&url), &signed);

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
        let message = resp_json
            .get("error")
            .and_then(|e| e.get("message"))
            .or_else(|| resp_json.get("message"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown server error");
        anyhow::bail!("Server error {status}: {message}");
    }

    match format {
        crate::output::OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&resp_json).context("Failed to format response")?
            );
        }
        crate::output::OutputFormat::Human => {
            println!("Deleted: {uid} v{version}");
        }
    }

    Ok(())
}
