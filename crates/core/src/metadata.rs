use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Well-known agent fields included in upload metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub model: Option<String>,
    pub provider: Option<String>,
    pub tool: Option<String>,
}

/// Per-upload metadata containing well-known fields and arbitrary custom fields.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Metadata {
    pub title: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    pub agent: Option<AgentInfo>,
    pub trigger: Option<String>,
    #[serde(default)]
    pub custom: HashMap<String, String>,
}

/// Full version metadata stored in `meta.json` alongside each uploaded file version.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionMeta {
    pub version: u32,
    pub filename: String,
    pub content_type: String,
    pub size_bytes: u64,
    pub uploaded_at: DateTime<Utc>,
    pub uploaded_by: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub metadata: Metadata,
}

/// Top-level upload record stored in `upload.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadRecord {
    pub uid: String,
    pub owner: String,
    pub collection: Option<String>,
    pub latest_version: u32,
    pub created_at: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,
}

/// A single member entry within a collection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionMember {
    pub uid: String,
    pub added_at: DateTime<Utc>,
}

/// Collection record stored in `collections/{name}.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionRecord {
    pub name: String,
    pub members: Vec<CollectionMember>,
    pub created_at: DateTime<Utc>,
}

/// A single user entry within the users config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRecord {
    pub public_key: String,
    pub display_name: Option<String>,
    pub is_admin: bool,
    pub created_at: DateTime<Utc>,
}

/// Full users config stored in `users.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsersConfig {
    pub users: HashMap<String, UserRecord>,
}
