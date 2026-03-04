use std::env;

/// Log output format.
#[derive(Debug, Clone, Default)]
pub enum LogFormat {
    Json,
    #[default]
    Pretty,
}

/// Server configuration loaded from environment variables.
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Path to the storage directory. `AGENTBIN_STORAGE_PATH`, default `./data`.
    pub storage_path: String,
    /// TCP address the server binds to. `AGENTBIN_LISTEN_ADDR`, default `0.0.0.0:8080`.
    pub listen_addr: String,
    /// Public base URL used for link generation. `AGENTBIN_BASE_URL`, default `http://localhost:8080`.
    pub base_url: String,
    /// Log output format. `AGENTBIN_LOG_FORMAT` (`json` | `pretty`), default `pretty`.
    pub log_format: LogFormat,
    /// How often to run the expiry sweeper in seconds. `AGENTBIN_SWEEP_INTERVAL`, default `60`.
    pub sweep_interval_secs: u64,
}

impl ServerConfig {
    /// Load configuration from environment variables, falling back to defaults.
    pub fn from_env() -> Self {
        let log_format = match env::var("AGENTBIN_LOG_FORMAT")
            .unwrap_or_default()
            .to_lowercase()
            .as_str()
        {
            "json" => LogFormat::Json,
            _ => LogFormat::Pretty,
        };

        let sweep_interval_secs = env::var("AGENTBIN_SWEEP_INTERVAL")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(60);

        Self {
            storage_path: env::var("AGENTBIN_STORAGE_PATH")
                .unwrap_or_else(|_| "./data".to_string()),
            listen_addr: env::var("AGENTBIN_LISTEN_ADDR")
                .unwrap_or_else(|_| "0.0.0.0:8080".to_string()),
            base_url: env::var("AGENTBIN_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:8080".to_string()),
            log_format,
            sweep_interval_secs,
        }
    }
}
