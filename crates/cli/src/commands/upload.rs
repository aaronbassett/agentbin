use anyhow::Context;
use rand::RngCore;
use std::path::Path;

const MAX_FILE_SIZE: usize = 1024 * 1024; // 1 MB

/// Optional metadata attached to an upload.
pub struct UploadOptions<'a> {
    pub uid: Option<&'a str>,
    pub title: Option<&'a str>,
    pub tags: Option<&'a str>,
    pub collection: Option<&'a str>,
    pub ttl: Option<u64>,
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

    // Build the raw multipart body so we can sign the exact bytes that will
    // be sent — the server's auth middleware signs the buffered request body.
    let boundary = generate_boundary();
    let body_bytes = build_multipart_body(
        &boundary,
        filename,
        &file_bytes,
        opts.title,
        opts.tags,
        opts.collection,
        opts.ttl,
    );

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
    title: Option<&str>,
    tags: Option<&str>,
    collection: Option<&str>,
    ttl: Option<u64>,
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

    // Optional metadata text fields.
    for (name, value) in [("title", title), ("tags", tags), ("collection", collection)] {
        if let Some(v) = value {
            append_text_part(&mut body, boundary, name, v);
        }
    }
    if let Some(ttl_val) = ttl {
        append_text_part(&mut body, boundary, "ttl", &ttl_val.to_string());
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
