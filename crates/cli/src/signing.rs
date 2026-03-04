use base64::{engine::general_purpose::STANDARD, Engine as _};
use ed25519_dalek::SigningKey;

/// Headers computed for an authenticated HTTP request.
pub struct SignedRequest {
    pub public_key: String,
    pub signature: String,
    pub timestamp: i64,
}

/// Sign an outgoing HTTP request and return the auth headers.
///
/// Builds the canonical signing payload, signs it with the provided Ed25519
/// private key (base64-encoded raw bytes), and derives the matching public key.
pub fn sign_http_request(
    private_key_b64: &str,
    method: &str,
    path: &str,
    body: &[u8],
) -> anyhow::Result<SignedRequest> {
    let timestamp = chrono::Utc::now().timestamp();

    let payload = agentbin_core::construct_signing_payload(method, path, timestamp, body);
    let signature = agentbin_core::sign_request(private_key_b64, &payload)
        .map_err(|e| anyhow::anyhow!("Failed to sign request: {e}"))?;

    let key_bytes = STANDARD
        .decode(private_key_b64)
        .map_err(|e| anyhow::anyhow!("Invalid private key encoding: {e}"))?;

    let key_array: [u8; 32] = key_bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("Private key must be exactly 32 bytes"))?;

    let public_key = STANDARD.encode(
        SigningKey::from_bytes(&key_array)
            .verifying_key()
            .to_bytes(),
    );

    Ok(SignedRequest {
        public_key,
        signature,
        timestamp,
    })
}

/// Attach Ed25519 auth headers to a `reqwest` request builder.
pub fn apply_auth_headers(
    builder: reqwest::RequestBuilder,
    signed: &SignedRequest,
) -> reqwest::RequestBuilder {
    builder
        .header("X-AgentBin-PublicKey", &signed.public_key)
        .header("X-AgentBin-Signature", &signed.signature)
        .header("X-AgentBin-Timestamp", signed.timestamp.to_string())
}
