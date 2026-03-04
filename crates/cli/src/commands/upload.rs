use anyhow::Context;
use rand::RngCore;
use std::path::Path;

const MAX_FILE_SIZE: usize = 1024 * 1024; // 1 MB

/// Optional metadata attached to an upload.
pub struct UploadOptions<'a> {
    pub uid: Option<&'a str>,
    pub title: Option<&'a str>,
    pub description: Option<&'a str>,
    pub tags: &'a [String],
    pub agent_model: Option<&'a str>,
    pub agent_provider: Option<&'a str>,
    pub agent_tool: Option<&'a str>,
    pub trigger: Option<&'a str>,
    /// Each entry is a `KEY=VALUE` string.
    pub meta: &'a [String],
    pub collection: Option<&'a str>,
    /// Expiry in days from now.
    pub expiry: Option<u64>,
}

/// Execute the upload command.
///
/// Reads a file from disk, signs the request body, and POSTs it to the
/// agentbin server as a multipart upload. When `opts.uid` is provided the
/// upload becomes a new version of that existing resource.
pub async fn execute(
    file: &str,
    opts: &UploadOptions<'_>,
    config: &crate::config::CliConfig,
    format: &crate::output::OutputFormat,
) -> anyhow::Result<()> {
    // Read file from disk.
    let path = Path::new(file);
    let file_bytes =
        std::fs::read(path).with_context(|| format!("File not found or unreadable: {file}"))?;

    // Enforce 1 MB limit before attempting the upload.
    if file_bytes.len() > MAX_FILE_SIZE {
        anyhow::bail!(
            "File too large: {} bytes (maximum is 1 MB)",
            file_bytes.len()
        );
    }

    // Extract the filename component from the path.
    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .with_context(|| format!("Could not determine filename from path: {file}"))?;

    // Load the signing key.
    let private_key = config.read_private_key()?;

    // Resolve the target URL and its path-only form used in the signed payload.
    let (url, api_path) = match opts.uid {
        Some(id) => (
            format!("{}/api/upload/{id}", config.server_url),
            format!("/api/upload/{id}"),
        ),
        None => (
            format!("{}/api/upload", config.server_url),
            "/api/upload".to_string(),
        ),
    };

    // Build a metadata JSON string from the provided options.
    let metadata_json = build_metadata_json(opts);

    // Build the raw multipart body so we can sign the exact bytes that will
    // be sent — the server's auth middleware signs the buffered request body.
    let boundary = generate_boundary();
    let body_bytes =
        build_multipart_body(&boundary, filename, &file_bytes, metadata_json.as_deref());

    // Sign: METHOD + path + SHA-256 of body bytes.
    let signed = crate::signing::sign_http_request(&private_key, "POST", &api_path, &body_bytes)?;

    // Send the signed request.
    let content_type = format!("multipart/form-data; boundary={boundary}");
    let client = reqwest::Client::new();
    let builder = client
        .post(&url)
        .header("Content-Type", content_type)
        .body(body_bytes);
    let builder = crate::signing::apply_auth_headers(builder, &signed);

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
            .get("message")
            .or_else(|| resp_json.get("error"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown server error");
        anyhow::bail!("Server error {status}: {message}");
    }

    // Display the result.
    match format {
        crate::output::OutputFormat::Human => {
            let url_out = resp_json
                .get("url")
                .and_then(|v| v.as_str())
                .unwrap_or("(unknown)");
            let raw_url = resp_json
                .get("raw_url")
                .and_then(|v| v.as_str())
                .unwrap_or("(unknown)");
            let uid_out = resp_json
                .get("uid")
                .and_then(|v| v.as_str())
                .unwrap_or("(unknown)");
            let version = resp_json
                .get("version")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            println!("Uploaded! URL: {url_out}");
            println!("Raw: {raw_url}");
            println!("UID: {uid_out}");
            println!("Version: {version}");
        }
        crate::output::OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&resp_json).context("Failed to format response")?
            );
        }
    }

    Ok(())
}

/// Build a JSON metadata string from upload options.
///
/// Returns `None` when no metadata fields were provided so that the multipart
/// body omits the optional `metadata` part entirely.
fn build_metadata_json(opts: &UploadOptions<'_>) -> Option<String> {
    let mut obj = serde_json::Map::new();

    if let Some(title) = opts.title {
        obj.insert(
            "title".to_string(),
            serde_json::Value::String(title.to_string()),
        );
    }
    if let Some(desc) = opts.description {
        obj.insert(
            "description".to_string(),
            serde_json::Value::String(desc.to_string()),
        );
    }
    if !opts.tags.is_empty() {
        obj.insert(
            "tags".to_string(),
            serde_json::Value::Array(
                opts.tags
                    .iter()
                    .map(|t| serde_json::Value::String(t.clone()))
                    .collect(),
            ),
        );
    }
    if let Some(trigger) = opts.trigger {
        obj.insert(
            "trigger".to_string(),
            serde_json::Value::String(trigger.to_string()),
        );
    }

    // Agent sub-object — only included when at least one agent field is set.
    let mut agent = serde_json::Map::new();
    if let Some(model) = opts.agent_model {
        agent.insert(
            "model".to_string(),
            serde_json::Value::String(model.to_string()),
        );
    }
    if let Some(provider) = opts.agent_provider {
        agent.insert(
            "provider".to_string(),
            serde_json::Value::String(provider.to_string()),
        );
    }
    if let Some(tool) = opts.agent_tool {
        agent.insert(
            "tool".to_string(),
            serde_json::Value::String(tool.to_string()),
        );
    }
    if !agent.is_empty() {
        obj.insert("agent".to_string(), serde_json::Value::Object(agent));
    }

    // Custom KEY=VALUE pairs.
    let mut custom = serde_json::Map::new();
    for kv in opts.meta {
        if let Some((k, v)) = kv.split_once('=') {
            custom.insert(k.to_string(), serde_json::Value::String(v.to_string()));
        }
    }
    if !custom.is_empty() {
        obj.insert("custom".to_string(), serde_json::Value::Object(custom));
    }

    // Upload-level fields interpreted by the server outside of `Metadata`.
    if let Some(collection) = opts.collection {
        obj.insert(
            "collection".to_string(),
            serde_json::Value::String(collection.to_string()),
        );
    }
    if let Some(expiry) = opts.expiry {
        obj.insert(
            "expiry".to_string(),
            serde_json::Value::Number(serde_json::Number::from(expiry)),
        );
    }

    if obj.is_empty() {
        None
    } else {
        Some(serde_json::Value::Object(obj).to_string())
    }
}

/// Generate a random multipart boundary string.
fn generate_boundary() -> String {
    let mut bytes = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut bytes);
    let hex: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
    format!("----FormBoundary{hex}")
}

/// Construct the raw `multipart/form-data` body bytes.
///
/// All parts are written in one pass so the result is deterministic and can
/// be included verbatim in the Ed25519 signing payload.
fn build_multipart_body(
    boundary: &str,
    filename: &str,
    file_bytes: &[u8],
    metadata_json: Option<&str>,
) -> Vec<u8> {
    let mut body: Vec<u8> = Vec::new();

    // Primary file part.
    let file_header = format!(
        "--{boundary}\r\n\
         Content-Disposition: form-data; name=\"file\"; filename=\"{filename}\"\r\n\
         Content-Type: application/octet-stream\r\n\r\n"
    );
    body.extend_from_slice(file_header.as_bytes());
    body.extend_from_slice(file_bytes);
    body.extend_from_slice(b"\r\n");

    // Optional metadata JSON field.
    if let Some(json) = metadata_json {
        append_text_part(&mut body, boundary, "metadata", json);
    }

    // Closing delimiter.
    body.extend_from_slice(format!("--{boundary}--\r\n").as_bytes());
    body
}

/// Append a plain-text form field part to a multipart body buffer.
fn append_text_part(body: &mut Vec<u8>, boundary: &str, name: &str, value: &str) {
    let part = format!(
        "--{boundary}\r\n\
         Content-Disposition: form-data; name=\"{name}\"\r\n\r\n\
         {value}\r\n"
    );
    body.extend_from_slice(part.as_bytes());
}
