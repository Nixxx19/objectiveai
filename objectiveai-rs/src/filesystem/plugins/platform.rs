use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A supported runtime target — the cross product of OS and CPU
/// architecture the cli knows how to install plugin binaries for.
/// Serialized as `<os>-<arch>` (e.g. `"linux-x86_64"`,
/// `"windows-aarch64"`). Used as the key type in
/// [`super::Manifest::binaries`] so a manifest can declare a distinct
/// release-asset filename per platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[schemars(rename = "filesystem.plugins.Platform")]
pub enum Platform {
    #[serde(rename = "linux-x86_64")]
    LinuxX86_64,
    #[serde(rename = "linux-aarch64")]
    LinuxAarch64,
    #[serde(rename = "windows-x86_64")]
    WindowsX86_64,
    #[serde(rename = "windows-aarch64")]
    WindowsAarch64,
    #[serde(rename = "macos-x86_64")]
    MacosX86_64,
    #[serde(rename = "macos-aarch64")]
    MacosAarch64,
}

impl Platform {
    /// The platform this binary was built for, if recognized. Returns
    /// `None` on exotic build targets (BSD, RISC-V, 32-bit ARM, etc.)
    /// — those simply have no manifest binding.
    pub fn current() -> Option<Self> {
        match (std::env::consts::OS, std::env::consts::ARCH) {
            ("linux", "x86_64") => Some(Self::LinuxX86_64),
            ("linux", "aarch64") => Some(Self::LinuxAarch64),
            ("windows", "x86_64") => Some(Self::WindowsX86_64),
            ("windows", "aarch64") => Some(Self::WindowsAarch64),
            ("macos", "x86_64") => Some(Self::MacosX86_64),
            ("macos", "aarch64") => Some(Self::MacosAarch64),
            _ => None,
        }
    }
}
