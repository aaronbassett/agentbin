use base64::{engine::general_purpose::STANDARD, Engine as _};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};

use crate::error::CoreError;

/// Generate an Ed25519 keypair.
///
/// Returns `(base64_private_key, base64_public_key)`.
pub fn generate_keypair() -> Result<(String, String), CoreError> {
    let signing_key = SigningKey::generate(&mut OsRng);
    let verifying_key = signing_key.verifying_key();

    let private_key_b64 = STANDARD.encode(signing_key.to_bytes());
    let public_key_b64 = STANDARD.encode(verifying_key.to_bytes());

    Ok((private_key_b64, public_key_b64))
}

/// Construct the canonical signing payload.
///
/// Format: `"{METHOD}\n{PATH}\n{TIMESTAMP}\n{BODY_SHA256_HEX}"`
pub fn construct_signing_payload(method: &str, path: &str, timestamp: i64, body: &[u8]) -> String {
    let hash = Sha256::digest(body);
    let body_hash: String = hash.iter().map(|b| format!("{b:02x}")).collect();
    format!("{method}\n{path}\n{timestamp}\n{body_hash}")
}

/// Sign a payload with a base64-encoded Ed25519 private key.
///
/// Returns a base64-encoded signature.
pub fn sign_request(private_key_b64: &str, payload: &str) -> Result<String, CoreError> {
    let key_bytes = STANDARD
        .decode(private_key_b64)
        .map_err(|e| CoreError::AuthError(format!("Invalid private key encoding: {e}")))?;

    let key_array: [u8; 32] = key_bytes
        .try_into()
        .map_err(|_| CoreError::AuthError("Private key must be 32 bytes".to_string()))?;

    let signing_key = SigningKey::from_bytes(&key_array);
    let signature = signing_key.sign(payload.as_bytes());

    Ok(STANDARD.encode(signature.to_bytes()))
}

/// Verify an Ed25519 signature against a payload.
///
/// Returns `Ok(())` on success, `Err(CoreError::AuthError)` on failure.
pub fn verify_signature(
    public_key_b64: &str,
    payload: &str,
    signature_b64: &str,
) -> Result<(), CoreError> {
    let key_bytes = STANDARD
        .decode(public_key_b64)
        .map_err(|e| CoreError::AuthError(format!("Invalid public key encoding: {e}")))?;

    let key_array: [u8; 32] = key_bytes
        .try_into()
        .map_err(|_| CoreError::AuthError("Public key must be 32 bytes".to_string()))?;

    let verifying_key = VerifyingKey::from_bytes(&key_array)
        .map_err(|e| CoreError::AuthError(format!("Invalid public key: {e}")))?;

    let sig_bytes = STANDARD
        .decode(signature_b64)
        .map_err(|e| CoreError::AuthError(format!("Invalid signature encoding: {e}")))?;

    let sig_array: [u8; 64] = sig_bytes
        .try_into()
        .map_err(|_| CoreError::AuthError("Signature must be 64 bytes".to_string()))?;

    let signature = Signature::from_bytes(&sig_array);

    verifying_key
        .verify(payload.as_bytes(), &signature)
        .map_err(|e| CoreError::AuthError(format!("Signature verification failed: {e}")))
}

/// Validate that a request timestamp is within ±300 seconds of now.
///
/// Returns `Err(CoreError::AuthError)` if the timestamp is outside the window.
pub fn validate_timestamp(request_timestamp: i64) -> Result<(), CoreError> {
    let now = chrono::Utc::now().timestamp();
    let diff = (now - request_timestamp).abs();

    if diff > 300 {
        return Err(CoreError::AuthError(
            "Replay detected: timestamp outside acceptable window".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_keypair() {
        let (private_key, public_key) = generate_keypair().unwrap();
        let priv_bytes = STANDARD.decode(&private_key).unwrap();
        let pub_bytes = STANDARD.decode(&public_key).unwrap();
        assert_eq!(priv_bytes.len(), 32);
        assert_eq!(pub_bytes.len(), 32);
    }

    #[test]
    fn test_sign_and_verify() {
        let (private_key, public_key) = generate_keypair().unwrap();
        let payload = construct_signing_payload("POST", "/upload", 1_234_567_890, b"hello world");
        let signature = sign_request(&private_key, &payload).unwrap();
        assert!(verify_signature(&public_key, &payload, &signature).is_ok());
    }

    #[test]
    fn test_verify_bad_signature() {
        let (private_key, _) = generate_keypair().unwrap();
        let (_, other_public_key) = generate_keypair().unwrap();
        let payload = construct_signing_payload("POST", "/upload", 1_234_567_890, b"hello world");
        let signature = sign_request(&private_key, &payload).unwrap();
        assert!(verify_signature(&other_public_key, &payload, &signature).is_err());
    }

    #[test]
    fn test_validate_timestamp() {
        let now = chrono::Utc::now().timestamp();
        assert!(validate_timestamp(now).is_ok());
        assert!(validate_timestamp(now - 600).is_err());
        assert!(validate_timestamp(now + 600).is_err());
    }

    #[test]
    fn test_signing_payload_format() {
        let payload = construct_signing_payload("GET", "/files", 1_234_567_890, b"");
        let parts: Vec<&str> = payload.splitn(4, '\n').collect();
        assert_eq!(parts.len(), 4);
        assert_eq!(parts[0], "GET");
        assert_eq!(parts[1], "/files");
        assert_eq!(parts[2], "1234567890");
        // SHA-256 of empty string
        assert_eq!(
            parts[3],
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }
}
