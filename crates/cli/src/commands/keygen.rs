use std::fs;
use std::path::PathBuf;

use crate::output::OutputFormat;

/// Resolve the default key file path: `~/.config/agentbin/key.pem`.
fn default_key_path() -> anyhow::Result<PathBuf> {
    let home = std::env::var("HOME")
        .map_err(|_| anyhow::anyhow!("HOME environment variable is not set"))?;
    Ok(PathBuf::from(home)
        .join(".config")
        .join("agentbin")
        .join("key.pem"))
}

/// Generate an Ed25519 key pair and save the private key to disk.
pub async fn execute(force: bool, format: &OutputFormat) -> anyhow::Result<()> {
    let key_path = default_key_path()?;

    if key_path.exists() && !force {
        anyhow::bail!(
            "Key file already exists at {}. Use --force to overwrite.",
            key_path.display()
        );
    }

    // Create parent directory if needed.
    if let Some(parent) = key_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let (private_key_b64, public_key_b64) = agentbin_core::generate_keypair()?;

    let pem = format!(
        "-----BEGIN AGENTBIN PRIVATE KEY-----\n{private_key_b64}\n-----END AGENTBIN PRIVATE KEY-----\n"
    );

    fs::write(&key_path, &pem)?;

    // Restrict permissions to owner read/write only.
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::Permissions::from_mode(0o600);
        fs::set_permissions(&key_path, perms)?;
    }

    let key_path_str = key_path.to_string_lossy();

    match format {
        OutputFormat::Human => {
            println!(
                "Key pair generated.\nPrivate key saved to: {key_path_str}\nPublic key: {public_key_b64}\n\nShare the public key above with your admin."
            );
        }
        OutputFormat::Json => {
            let output = serde_json::json!({
                "private_key_path": key_path_str,
                "public_key": public_key_b64,
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
    }

    Ok(())
}
