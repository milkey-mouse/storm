use serde::{Deserialize, Serialize};
#[cfg(feature = "interactive")]
use serde_diff::{simple_serde_diff, SerdeDiff};
use std::path::PathBuf;

#[cfg_attr(feature = "interactive", derive(Clone, PartialEq, SerdeDiff))]
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct GentooRepo {
    location: PathBuf,
    sync_type: SyncType,
    sync_uri: String,
}

#[cfg_attr(feature = "interactive", derive(Clone, PartialEq/*, SerdeDiff*/))]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum SyncType {
    Cvs,
    Git,
    Rsync,
    Svn,
    WebRsync,
}

simple_serde_diff!(SyncType);

impl Default for SyncType {
    fn default() -> Self {
        Self::Rsync
    }
}
