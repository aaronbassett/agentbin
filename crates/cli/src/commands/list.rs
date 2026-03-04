use anyhow::Context;

/// Execute the list command.
///
/// Fetches the authenticated user's uploads from the server and displays them
/// as a formatted table (human mode) or raw JSON (json mode).
pub async fn execute(
    collection_filter: Option<&str>,
    config: &crate::config::CliConfig,
    format: &crate::output::OutputFormat,
) -> anyhow::Result<()> {
    let private_key = config.read_private_key()?;
    let api_path = "/api/uploads";
    let url = format!("{}{}", config.server_url, api_path);

    let signed = crate::signing::sign_http_request(&private_key, "GET", api_path, &[])?;

    let client = reqwest::Client::new();
    let builder = crate::signing::apply_auth_headers(client.get(&url), &signed);

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

    let empty_vec = Vec::new();
    let all_uploads = resp_json
        .get("uploads")
        .and_then(|v| v.as_array())
        .unwrap_or(&empty_vec);

    // Apply optional collection filter.
    let uploads: Vec<&serde_json::Value> = all_uploads
        .iter()
        .filter(|u| {
            if let Some(coll) = collection_filter {
                u.get("collection")
                    .and_then(|c| c.as_str())
                    .map(|c| c == coll)
                    .unwrap_or(false)
            } else {
                true
            }
        })
        .collect();

    match format {
        crate::output::OutputFormat::Json => {
            let out = serde_json::json!({ "uploads": uploads });
            println!(
                "{}",
                serde_json::to_string_pretty(&out).context("Failed to format response")?
            );
        }
        crate::output::OutputFormat::Human => {
            if uploads.is_empty() {
                println!("No uploads found.");
                return Ok(());
            }

            println!("{:<12} {:>4} {:<20} CREATED", "UID", "V", "COLLECTION");
            println!("{}", "-".repeat(64));

            for upload in &uploads {
                let uid = upload.get("uid").and_then(|v| v.as_str()).unwrap_or("?");
                let version = upload
                    .get("latest_version")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0);
                let collection = upload
                    .get("collection")
                    .and_then(|v| v.as_str())
                    .unwrap_or("—");
                let created_at = upload
                    .get("created_at")
                    .and_then(|v| v.as_str())
                    .unwrap_or("?");
                // Trim to seconds precision (drop sub-second and timezone suffix noise).
                let created_short = created_at
                    .find('.')
                    .map(|i| &created_at[..i])
                    .unwrap_or(created_at);

                println!(
                    "{:<12} {:>4} {:<20} {}",
                    uid, version, collection, created_short
                );
            }
        }
    }

    Ok(())
}
