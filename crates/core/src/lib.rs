#![deny(unsafe_code)]

pub mod error;
pub mod filetype;
pub mod metadata;
pub mod uid;

pub use error::CoreError;
pub use filetype::FileType;
pub use metadata::{
    AgentInfo, CollectionMember, CollectionRecord, Metadata, UploadRecord, UserRecord, UsersConfig,
    VersionMeta,
};
pub use uid::generate_uid;
