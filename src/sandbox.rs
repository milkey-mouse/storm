use serde::{Deserialize, Serialize};
#[cfg(feature = "interactive")]
use serde_diff::{simple_serde_diff, SerdeDiff};

#[cfg_attr(feature = "interactive", derive(Clone, PartialEq/*, SerdeDiff*/))]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", tag = "type")]
pub enum SandboxConfig {
    Chroot,
    Firecracker,
    CrosVM,
}

simple_serde_diff!(SandboxConfig);

impl Default for SandboxConfig {
    fn default() -> Self {
        Self::Chroot
    }
}
