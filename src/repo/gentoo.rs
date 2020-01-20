use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct GentooRepo {
    location: PathBuf,
    sync_type: SyncType,
    sync_uri: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum SyncType {
    Cvs,
    Git,
    Rsync,
    Svn,
    WebRsync,
}

impl Default for SyncType {
    fn default() -> Self {
        Self::Rsync
    }
}
