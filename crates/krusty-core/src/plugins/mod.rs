//! Installable TUI plugin management.
//!
//! This module manages trusted plugin distribution metadata, installation,
//! lockfile pinning, and filesystem layout under `~/.krusty/plugins`.

mod manager;
mod signing;
mod types;

pub use manager::PluginManager;
pub use types::{
    InstalledPlugin, PluginCompat, PluginLockEntry, PluginLockfile, PluginManifestV1,
    PluginPermissionSet, PluginPermissionsFile, PluginRelease, PluginRenderCapability,
    PluginSource, PluginSourcesFile, PluginTrustPolicy,
};
