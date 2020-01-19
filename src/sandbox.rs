use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase", tag = "type")]
pub enum SandboxConfig {
    Chroot,
    Firecracker,
    CrosVM,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self::Chroot
    }
}
