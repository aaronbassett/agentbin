#![deny(unsafe_code)]

pub mod auth;
pub mod error;
pub mod filetype;
pub mod metadata;
pub mod render;
pub mod uid;

pub use auth::{
    construct_signing_payload, generate_keypair, sign_request, validate_timestamp, verify_signature,
};
pub use error::CoreError;
pub use filetype::FileType;
pub use metadata::{
    AgentInfo, CollectionMember, CollectionRecord, Metadata, UploadRecord, UserRecord, UsersConfig,
    VersionMeta,
};
pub use render::{highlight_code, render_markdown, wrap_plain_text};
pub use uid::generate_uid;
