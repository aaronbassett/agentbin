use anyhow::Context;
use serde::Deserialize;
use std::fs;

const DEFAULT_SERVER_URL: &str = "https://agentbin.fly.dev";
const DEFAULT_KEY_FILE: &str = "~/.config/agentbin/key.pem";
const CONFIG_FILE_PATH: &str = "~/.config/agentbin/config.json";

/// Persistent configuration loaded from file, with env/flag overrides.
#[derive(Debug, Deserialize, Default)]
struct ConfigFile {
    server_url: Option<String>,
    key_file: Option<String>,
}

/// Resolved CLI configuration after applying precedence:
/// flags > env > config file > defaults.
#[derive(Debug)]
pub struct CliConfig {
    pub server_url: String,
    pub key_file: String,
}

impl CliConfig {
    /// Load configuration with precedence: flags > env > config file > defaults.
    ///
    /// `server_url` and `key_file` arguments already incorporate flag and env values
    /// (clap's `env` attribute handles that). If `None`, we fall back to the config
    /// file, then to built-in defaults.
    pub fn load(server_url: Option<&str>, key_file: Option<&str>) -> Self {
        let file_config = read_config_file().unwrap_or_default();

        let server_url = server_url
            .map(str::to_owned)
            .or(file_config.server_url)
            .unwrap_or_else(|| DEFAULT_SERVER_URL.to_owned());

        let key_file = key_file
            .map(str::to_owned)
            .or(file_config.key_file)
            .unwrap_or_else(|| DEFAULT_KEY_FILE.to_owned());

        Self {
            server_url,
            key_file: expand_tilde(&key_file),
        }
    }

    /// Read the private key from the configured key file, stripping PEM headers.
    ///
    /// Returns the raw base64-encoded key bytes (no header/footer lines).
    pub fn read_private_key(&self) -> anyhow::Result<String> {
        let content = fs::read_to_string(&self.key_file)
            .with_context(|| format!("Failed to read key file: {}", self.key_file))?;

        let key_b64: String = content
            .lines()
            .filter(|line| !line.starts_with("-----"))
            .collect::<Vec<_>>()
            .join("");

        if key_b64.is_empty() {
            anyhow::bail!("Key file contains no key data: {}", self.key_file);
        }

        Ok(key_b64)
    }
}

/// Expand a leading `~` to the user's home directory.
fn expand_tilde(path: &str) -> String {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Ok(home) = std::env::var("HOME") {
            return format!("{home}/{rest}");
        }
    } else if path == "~" {
        if let Ok(home) = std::env::var("HOME") {
            return home;
        }
    }
    path.to_owned()
}

/// Attempt to read and parse the config file. Returns `None` on any error.
fn read_config_file() -> Option<ConfigFile> {
    let path = expand_tilde(CONFIG_FILE_PATH);
    let content = fs::read_to_string(&path).ok()?;
    serde_json::from_str(&content).ok()
}
