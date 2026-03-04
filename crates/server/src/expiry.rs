#![deny(unsafe_code)]

use std::sync::Arc;

use agentbin_core::FileStorage;
use chrono::Utc;

/// Run the expiration sweeper loop.
///
/// Scans all uploads periodically and removes expired versions. Runs until the
/// spawned task is dropped (i.e. the process exits).
pub async fn run_sweeper(storage: Arc<FileStorage>, interval_secs: u64) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(interval_secs));

    loop {
        interval.tick().await;

        tracing::debug!("expiry sweeper: starting scan");

        let uids = match storage.list_all_upload_uids() {
            Ok(u) => u,
            Err(e) => {
                tracing::error!(error = %e, "expiry sweeper: failed to list upload UIDs");
                continue;
            }
        };

        let mut deleted_versions: u64 = 0;
        let mut deleted_uploads: u64 = 0;
        let now = Utc::now();

        for uid in &uids {
            let metas = match storage.list_version_metas(uid) {
                Ok(m) => m,
                Err(e) => {
                    tracing::warn!(uid = %uid, error = %e, "expiry sweeper: failed to list versions");
                    continue;
                }
            };

            for meta in metas {
                let Some(expires_at) = meta.expires_at else {
                    continue;
                };

                if expires_at >= now {
                    continue;
                }

                match storage.delete_version(uid, meta.version) {
                    Ok(fully_deleted) => {
                        deleted_versions += 1;
                        tracing::info!(
                            uid = %uid,
                            version = meta.version,
                            expired_at = %expires_at,
                            "expiry sweeper: deleted expired version",
                        );
                        if fully_deleted {
                            deleted_uploads += 1;
                            tracing::info!(uid = %uid, "expiry sweeper: upload fully removed");
                            // All versions gone; the upload dir is already cleaned up.
                            break;
                        }
                    }
                    Err(e) => {
                        tracing::error!(
                            uid = %uid,
                            version = meta.version,
                            error = %e,
                            "expiry sweeper: failed to delete expired version",
                        );
                    }
                }
            }
        }

        tracing::debug!(
            total_uids = uids.len(),
            deleted_versions,
            deleted_uploads,
            "expiry sweeper: scan complete",
        );
    }
}
