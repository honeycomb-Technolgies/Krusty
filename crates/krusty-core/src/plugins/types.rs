use std::{collections::BTreeMap, path::PathBuf};

use serde::{Deserialize, Serialize};

fn default_manifest_version() -> u32 {
    1
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PluginRenderCapability {
    Text,
    Frame,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PluginPermissionSet {
    #[serde(default)]
    pub fs_read: bool,
    #[serde(default)]
    pub fs_write: bool,
    #[serde(default)]
    pub network: bool,
    #[serde(default)]
    pub process: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PluginCompat {
    #[serde(default)]
    pub krusty_min: Option<String>,
    #[serde(default)]
    pub krusty_max: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginRelease {
    pub url: String,
    pub sha256: String,
    pub signature: String,
    pub signing_key_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginManifestV1 {
    #[serde(default = "default_manifest_version")]
    pub manifest_version: u32,
    pub id: String,
    pub name: String,
    pub version: String,
    pub publisher: String,
    #[serde(default)]
    pub description: Option<String>,
    pub entry_component: String,
    #[serde(default)]
    pub render_capabilities: Vec<PluginRenderCapability>,
    #[serde(default)]
    pub requested_permissions: PluginPermissionSet,
    pub release: PluginRelease,
    #[serde(default)]
    pub compat: PluginCompat,
}

impl PluginManifestV1 {
    pub fn normalized_render_capabilities(&self) -> Vec<PluginRenderCapability> {
        if self.render_capabilities.is_empty() {
            vec![PluginRenderCapability::Text]
        } else {
            self.render_capabilities.clone()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstalledPlugin {
    pub id: String,
    pub name: String,
    pub version: String,
    pub publisher: String,
    #[serde(default)]
    pub description: Option<String>,
    pub install_path: PathBuf,
    pub manifest_path: PathBuf,
    pub entry_component_path: PathBuf,
    pub enabled: bool,
    pub pinned: bool,
    #[serde(default)]
    pub render_capabilities: Vec<PluginRenderCapability>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginSource {
    pub name: String,
    pub manifest_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginSourcesFile {
    #[serde(default = "default_manifest_version")]
    pub version: u32,
    #[serde(default)]
    pub sources: Vec<PluginSource>,
}

impl Default for PluginSourcesFile {
    fn default() -> Self {
        Self {
            version: default_manifest_version(),
            sources: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginLockEntry {
    pub id: String,
    pub version: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub pinned: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginLockfile {
    #[serde(default = "default_manifest_version")]
    pub version: u32,
    #[serde(default)]
    pub plugins: Vec<PluginLockEntry>,
}

impl Default for PluginLockfile {
    fn default() -> Self {
        Self {
            version: default_manifest_version(),
            plugins: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PluginTrustPolicy {
    #[serde(default)]
    pub allowed_publishers: Vec<String>,
    #[serde(default)]
    pub keys: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PluginPermissionsFile {
    #[serde(default)]
    pub plugins: BTreeMap<String, PluginPermissionSet>,
}
