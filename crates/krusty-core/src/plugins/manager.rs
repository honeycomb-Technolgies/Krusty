use std::{
    ffi::OsStr,
    path::{Component, Path, PathBuf},
};

use anyhow::{anyhow, bail, Context, Result};
use serde::{de::DeserializeOwned, Serialize};
use sha2::{Digest as _, Sha256};
use tokio::fs;
use tracing::warn;
use url::Url;

use super::{
    signing::verify_artifact_signature, InstalledPlugin, PluginLockEntry, PluginLockfile,
    PluginManifestV1, PluginSource, PluginSourcesFile, PluginTrustPolicy,
};

#[derive(Debug, Clone)]
enum ManifestLocation {
    Remote(Url),
    Local(PathBuf),
}

#[derive(Debug, Clone)]
enum ArtifactLocation {
    Remote(Url),
    Local(PathBuf),
}

/// Central manager for installable TUI plugins.
#[derive(Clone)]
pub struct PluginManager {
    root: PathBuf,
    http_client: reqwest::Client,
}

impl PluginManager {
    pub fn new(http_client: reqwest::Client, root: PathBuf) -> Self {
        Self { root, http_client }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn installed_root(&self) -> PathBuf {
        self.root.join("installed")
    }

    pub fn active_root(&self) -> PathBuf {
        self.root.join("active")
    }

    pub fn state_root(&self) -> PathBuf {
        self.root.join("state")
    }

    pub fn index_root(&self) -> PathBuf {
        self.root.join("index")
    }

    pub fn trust_root(&self) -> PathBuf {
        self.root.join("trust")
    }

    pub fn lockfile_path(&self) -> PathBuf {
        self.root.join("plugins.lock")
    }

    fn trust_file_path(&self) -> PathBuf {
        self.trust_root().join("allowlist.toml")
    }

    fn sources_file_path(&self) -> PathBuf {
        self.index_root().join("sources.toml")
    }

    /// Ensure required plugin directories and config files exist.
    pub async fn ensure_layout(&self) -> Result<()> {
        fs::create_dir_all(self.installed_root()).await?;
        fs::create_dir_all(self.active_root()).await?;
        fs::create_dir_all(self.state_root()).await?;
        fs::create_dir_all(self.index_root()).await?;
        fs::create_dir_all(self.trust_root()).await?;

        self.load_lockfile().await?;
        self.load_trust_policy().await?;
        self.load_sources().await?;

        Ok(())
    }

    pub async fn list_sources(&self) -> Result<Vec<PluginSource>> {
        Ok(self.load_sources().await?.sources)
    }

    pub async fn add_source(&self, name: Option<&str>, manifest_url: &str) -> Result<PluginSource> {
        let mut sources = self.load_sources().await?;
        let resolved_name = match name {
            Some(explicit) if !explicit.trim().is_empty() => explicit.trim().to_string(),
            _ => infer_source_name(manifest_url),
        };

        let source = PluginSource {
            name: resolved_name,
            manifest_url: manifest_url.to_string(),
        };

        sources
            .sources
            .retain(|entry| entry.name != source.name && entry.manifest_url != source.manifest_url);
        sources.sources.push(source.clone());
        sources.sources.sort_by(|a, b| a.name.cmp(&b.name));

        self.save_sources(&sources).await?;
        Ok(source)
    }

    pub async fn add_allowed_publisher(&self, publisher: &str) -> Result<()> {
        let mut trust = self.load_trust_policy().await?;
        if !trust
            .allowed_publishers
            .iter()
            .any(|existing| existing == publisher)
        {
            trust.allowed_publishers.push(publisher.to_string());
            trust.allowed_publishers.sort();
            self.save_trust_policy(&trust).await?;
        }
        Ok(())
    }

    pub async fn add_trusted_key(&self, key_id: &str, public_key_b64: &str) -> Result<()> {
        let mut trust = self.load_trust_policy().await?;
        trust
            .keys
            .insert(key_id.to_string(), public_key_b64.to_string());
        self.save_trust_policy(&trust).await
    }

    pub async fn set_plugin_enabled(&self, plugin_id: &str, enabled: bool) -> Result<()> {
        let mut lock = self.load_lockfile().await?;
        let entry = lock
            .plugins
            .iter_mut()
            .find(|entry| entry.id == plugin_id)
            .with_context(|| format!("plugin '{}' is not installed", plugin_id))?;

        entry.enabled = enabled;
        self.save_lockfile(&lock).await
    }

    /// Explicit reload request. v1 behavior is descriptor refresh only.
    pub async fn reload_plugin(&self, plugin_id: &str) -> Result<()> {
        let installed = self.list_installed_plugins().await?;
        if installed.iter().all(|plugin| plugin.id != plugin_id) {
            bail!("plugin '{}' is not installed", plugin_id);
        }
        Ok(())
    }

    pub async fn list_installed_plugins(&self) -> Result<Vec<InstalledPlugin>> {
        let lock = self.load_lockfile().await?;
        let mut installed = Vec::new();

        for entry in lock.plugins {
            match self.read_installed_from_lock_entry(&entry).await {
                Ok(plugin) => installed.push(plugin),
                Err(err) => {
                    warn!(
                        "Skipping installed plugin {}@{} due to invalid metadata: {}",
                        entry.id, entry.version, err
                    );
                }
            }
        }

        installed.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        Ok(installed)
    }

    pub async fn install_from_manifest_ref(&self, manifest_ref: &str) -> Result<InstalledPlugin> {
        let (manifest, manifest_location) = self.read_manifest_from_ref(manifest_ref).await?;

        self.validate_manifest(&manifest)?;
        self.verify_publisher_allowed(&manifest.publisher).await?;

        let artifact_location =
            resolve_artifact_location(&manifest.release.url, &manifest_location)?;
        let artifact_bytes = self.read_artifact(&artifact_location).await?;

        self.verify_artifact_integrity(&manifest, &artifact_bytes)
            .await?;

        let install_dir = self
            .installed_root()
            .join(&manifest.id)
            .join(&manifest.version);
        fs::create_dir_all(&install_dir).await?;

        let manifest_path = install_dir.join("plugin.toml");
        self.write_toml(&manifest_path, &manifest).await?;

        let entry_component_rel = validate_relative_path(&manifest.entry_component)?;
        let entry_component_path = install_dir.join(&entry_component_rel);
        if let Some(parent) = entry_component_path.parent() {
            fs::create_dir_all(parent).await?;
        }
        fs::write(&entry_component_path, artifact_bytes).await?;

        self.upsert_lock_entry(&manifest.id, &manifest.version, true, true)
            .await?;

        self.read_installed_from_lock_entry(&PluginLockEntry {
            id: manifest.id,
            version: manifest.version,
            enabled: true,
            pinned: true,
        })
        .await
    }

    async fn read_installed_from_lock_entry(
        &self,
        entry: &PluginLockEntry,
    ) -> Result<InstalledPlugin> {
        let install_path = self.installed_root().join(&entry.id).join(&entry.version);
        let manifest_path = install_path.join("plugin.toml");

        let manifest: PluginManifestV1 = self
            .read_toml_or_json(&manifest_path)
            .await
            .with_context(|| {
                format!("failed to load manifest for {}@{}", entry.id, entry.version)
            })?;

        let entry_component_rel = validate_relative_path(&manifest.entry_component)?;
        let entry_component_path = install_path.join(&entry_component_rel);
        let render_capabilities = manifest.normalized_render_capabilities();

        Ok(InstalledPlugin {
            id: manifest.id,
            name: manifest.name,
            version: manifest.version,
            publisher: manifest.publisher,
            description: manifest.description,
            install_path,
            manifest_path,
            entry_component_path,
            enabled: entry.enabled,
            pinned: entry.pinned,
            render_capabilities,
        })
    }

    async fn upsert_lock_entry(
        &self,
        plugin_id: &str,
        version: &str,
        enabled: bool,
        pinned: bool,
    ) -> Result<()> {
        let mut lock = self.load_lockfile().await?;
        lock.plugins.retain(|entry| entry.id != plugin_id);
        lock.plugins.push(PluginLockEntry {
            id: plugin_id.to_string(),
            version: version.to_string(),
            enabled,
            pinned,
        });
        lock.plugins.sort_by(|a, b| a.id.cmp(&b.id));
        self.save_lockfile(&lock).await
    }

    async fn verify_publisher_allowed(&self, publisher: &str) -> Result<()> {
        let trust = self.load_trust_policy().await?;
        if trust
            .allowed_publishers
            .iter()
            .any(|allowed| allowed == publisher)
        {
            return Ok(());
        }

        bail!(
            "publisher '{}' is not allowlisted. Add it via plugin trust configuration first",
            publisher
        )
    }

    async fn verify_artifact_integrity(
        &self,
        manifest: &PluginManifestV1,
        artifact_bytes: &[u8],
    ) -> Result<()> {
        let digest = Sha256::digest(artifact_bytes);
        let digest_hex = format!("{:x}", digest);
        if digest_hex != manifest.release.sha256.to_lowercase() {
            bail!(
                "sha256 mismatch for '{}': expected {}, got {}",
                manifest.id,
                manifest.release.sha256,
                digest_hex
            );
        }

        let trust = self.load_trust_policy().await?;
        let public_key = trust
            .keys
            .get(&manifest.release.signing_key_id)
            .with_context(|| {
                format!(
                    "trusted key '{}' not found in trust policy",
                    manifest.release.signing_key_id
                )
            })?;

        verify_artifact_signature(artifact_bytes, &manifest.release.signature, public_key)?;
        Ok(())
    }

    async fn read_manifest_from_ref(
        &self,
        manifest_ref: &str,
    ) -> Result<(PluginManifestV1, ManifestLocation)> {
        if let Ok(url) = Url::parse(manifest_ref) {
            if matches!(url.scheme(), "http" | "https") {
                let bytes = self
                    .http_client
                    .get(url.clone())
                    .send()
                    .await
                    .with_context(|| format!("failed to fetch manifest from {}", url))?
                    .error_for_status()
                    .with_context(|| format!("manifest request failed for {}", url))?
                    .bytes()
                    .await
                    .context("failed to read manifest response bytes")?;

                let manifest = parse_toml_or_json::<PluginManifestV1>(&bytes)?;
                return Ok((manifest, ManifestLocation::Remote(url)));
            }

            if url.scheme() == "file" {
                let path = url
                    .to_file_path()
                    .map_err(|_| anyhow!("invalid file URL: {}", url))?;
                let bytes = fs::read(&path)
                    .await
                    .with_context(|| format!("failed to read manifest file {}", path.display()))?;
                let manifest = parse_toml_or_json::<PluginManifestV1>(&bytes)?;
                return Ok((manifest, ManifestLocation::Local(path)));
            }
        }

        let path = PathBuf::from(manifest_ref);
        let bytes = fs::read(&path)
            .await
            .with_context(|| format!("failed to read manifest file {}", path.display()))?;
        let manifest = parse_toml_or_json::<PluginManifestV1>(&bytes)?;
        Ok((manifest, ManifestLocation::Local(path)))
    }

    async fn read_artifact(&self, location: &ArtifactLocation) -> Result<Vec<u8>> {
        match location {
            ArtifactLocation::Remote(url) => {
                let bytes = self
                    .http_client
                    .get(url.clone())
                    .send()
                    .await
                    .with_context(|| format!("failed to fetch artifact from {}", url))?
                    .error_for_status()
                    .with_context(|| format!("artifact request failed for {}", url))?
                    .bytes()
                    .await
                    .context("failed to read artifact response bytes")?;
                Ok(bytes.to_vec())
            }
            ArtifactLocation::Local(path) => fs::read(path)
                .await
                .with_context(|| format!("failed to read artifact {}", path.display())),
        }
    }

    fn validate_manifest(&self, manifest: &PluginManifestV1) -> Result<()> {
        if manifest.id.trim().is_empty() {
            bail!("manifest id cannot be empty");
        }
        if manifest.name.trim().is_empty() {
            bail!("manifest name cannot be empty");
        }
        if manifest.version.trim().is_empty() {
            bail!("manifest version cannot be empty");
        }
        if manifest.publisher.trim().is_empty() {
            bail!("manifest publisher cannot be empty");
        }
        if manifest.release.url.trim().is_empty() {
            bail!("manifest release.url cannot be empty");
        }
        if manifest.release.sha256.trim().is_empty() {
            bail!("manifest release.sha256 cannot be empty");
        }
        if manifest.release.signature.trim().is_empty() {
            bail!("manifest release.signature cannot be empty");
        }
        if manifest.release.signing_key_id.trim().is_empty() {
            bail!("manifest release.signing_key_id cannot be empty");
        }

        validate_relative_path(&manifest.entry_component)?;
        Ok(())
    }

    async fn load_lockfile(&self) -> Result<PluginLockfile> {
        let path = self.lockfile_path();
        if !path.exists() {
            let lock = PluginLockfile::default();
            self.write_toml(&path, &lock).await?;
            return Ok(lock);
        }

        self.read_toml_or_json(&path).await
    }

    async fn save_lockfile(&self, lock: &PluginLockfile) -> Result<()> {
        self.write_toml(&self.lockfile_path(), lock).await
    }

    async fn load_trust_policy(&self) -> Result<PluginTrustPolicy> {
        let path = self.trust_file_path();
        if !path.exists() {
            let trust = PluginTrustPolicy::default();
            self.write_toml(&path, &trust).await?;
            return Ok(trust);
        }

        self.read_toml_or_json(&path).await
    }

    async fn save_trust_policy(&self, trust: &PluginTrustPolicy) -> Result<()> {
        self.write_toml(&self.trust_file_path(), trust).await
    }

    async fn load_sources(&self) -> Result<PluginSourcesFile> {
        let path = self.sources_file_path();
        if !path.exists() {
            let sources = PluginSourcesFile::default();
            self.write_toml(&path, &sources).await?;
            return Ok(sources);
        }

        self.read_toml_or_json(&path).await
    }

    async fn save_sources(&self, sources: &PluginSourcesFile) -> Result<()> {
        self.write_toml(&self.sources_file_path(), sources).await
    }

    async fn read_toml_or_json<T: DeserializeOwned>(&self, path: &Path) -> Result<T> {
        let bytes = fs::read(path)
            .await
            .with_context(|| format!("failed to read {}", path.display()))?;

        parse_toml_or_json(&bytes).with_context(|| format!("failed to parse {}", path.display()))
    }

    async fn write_toml<T: Serialize>(&self, path: &Path, value: &T) -> Result<()> {
        let content = toml::to_string_pretty(value).context("failed to serialize toml")?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await?;
        }
        fs::write(path, content)
            .await
            .with_context(|| format!("failed to write {}", path.display()))
    }
}

fn parse_toml_or_json<T: DeserializeOwned>(bytes: &[u8]) -> Result<T> {
    if let Ok(as_toml) = toml::from_str::<T>(&String::from_utf8_lossy(bytes)) {
        return Ok(as_toml);
    }

    serde_json::from_slice::<T>(bytes).context("content is neither valid TOML nor JSON")
}

fn validate_relative_path(path: &str) -> Result<PathBuf> {
    let candidate = PathBuf::from(path);

    if candidate.as_os_str().is_empty() {
        bail!("entry_component cannot be empty");
    }

    if candidate.is_absolute() {
        bail!("entry_component must be a relative path");
    }

    for component in candidate.components() {
        match component {
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                bail!("entry_component cannot contain path traversal")
            }
            _ => {}
        }
    }

    Ok(candidate)
}

fn infer_source_name(manifest_url: &str) -> String {
    if let Ok(url) = Url::parse(manifest_url) {
        if let Some(host) = url.host_str() {
            return host.to_string();
        }
    }

    Path::new(manifest_url)
        .file_name()
        .and_then(OsStr::to_str)
        .map(|s| s.to_string())
        .unwrap_or_else(|| "plugin-source".to_string())
}

fn resolve_artifact_location(
    release_ref: &str,
    manifest_location: &ManifestLocation,
) -> Result<ArtifactLocation> {
    if let Ok(url) = Url::parse(release_ref) {
        return match url.scheme() {
            "http" | "https" => Ok(ArtifactLocation::Remote(url)),
            "file" => Ok(ArtifactLocation::Local(
                url.to_file_path()
                    .map_err(|_| anyhow!("invalid file URL: {}", url))?,
            )),
            other => bail!("unsupported release URL scheme: {}", other),
        };
    }

    match manifest_location {
        ManifestLocation::Local(manifest_path) => {
            let parent = manifest_path
                .parent()
                .ok_or_else(|| anyhow!("manifest path has no parent directory"))?;
            Ok(ArtifactLocation::Local(parent.join(release_ref)))
        }
        ManifestLocation::Remote(manifest_url) => bail!(
            "relative release path '{}' is not allowed for remote manifest {}",
            release_ref,
            manifest_url
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
    use ed25519_dalek::{Signer as _, SigningKey};
    use tempfile::tempdir;

    #[tokio::test]
    async fn installs_local_manifest_with_signature_verification() {
        let temp = tempdir().expect("tempdir");
        let workspace = temp.path();
        let manifest_dir = workspace.join("manifest");
        fs::create_dir_all(&manifest_dir)
            .await
            .expect("create manifest dir");

        let artifact_bytes = b"fake-wasm-component".to_vec();
        let artifact_path = manifest_dir.join("demo.wasm");
        fs::write(&artifact_path, &artifact_bytes)
            .await
            .expect("write artifact");

        let signing_key = SigningKey::from_bytes(&[7u8; 32]);
        let signature = signing_key.sign(&artifact_bytes);
        let public_key_b64 = BASE64.encode(signing_key.verifying_key().to_bytes());
        let signature_b64 = BASE64.encode(signature.to_bytes());
        let sha = format!("{:x}", Sha256::digest(&artifact_bytes));

        let manifest_path = manifest_dir.join("plugin.toml");
        fs::write(
            &manifest_path,
            format!(
                r#"
manifest_version = 1
id = "demo.plugin"
name = "Demo Plugin"
version = "1.0.0"
publisher = "demo.publisher"
entry_component = "demo.wasm"

[release]
url = "demo.wasm"
sha256 = "{sha}"
signature = "{signature_b64}"
signing_key_id = "demo-key"
"#
            ),
        )
        .await
        .expect("write manifest");

        let manager = PluginManager::new(reqwest::Client::new(), workspace.join("plugins"));
        manager.ensure_layout().await.expect("ensure layout");
        manager
            .add_allowed_publisher("demo.publisher")
            .await
            .expect("allow publisher");
        manager
            .add_trusted_key("demo-key", &public_key_b64)
            .await
            .expect("add key");

        let installed = manager
            .install_from_manifest_ref(manifest_path.to_str().expect("manifest path utf8"))
            .await
            .expect("install plugin");

        assert_eq!(installed.id, "demo.plugin");
        assert_eq!(installed.version, "1.0.0");
        assert!(installed.entry_component_path.exists());

        let plugins = manager
            .list_installed_plugins()
            .await
            .expect("list installed");
        assert_eq!(plugins.len(), 1);
        assert_eq!(plugins[0].id, "demo.plugin");
    }

    #[tokio::test]
    async fn rejects_non_allowlisted_publishers() {
        let temp = tempdir().expect("tempdir");
        let workspace = temp.path();
        let manifest_dir = workspace.join("manifest");
        fs::create_dir_all(&manifest_dir)
            .await
            .expect("create manifest dir");

        let artifact_bytes = b"fake-wasm-component".to_vec();
        let artifact_path = manifest_dir.join("demo.wasm");
        fs::write(&artifact_path, &artifact_bytes)
            .await
            .expect("write artifact");

        let signing_key = SigningKey::from_bytes(&[11u8; 32]);
        let signature = signing_key.sign(&artifact_bytes);
        let public_key_b64 = BASE64.encode(signing_key.verifying_key().to_bytes());
        let signature_b64 = BASE64.encode(signature.to_bytes());
        let sha = format!("{:x}", Sha256::digest(&artifact_bytes));

        let manifest_path = manifest_dir.join("plugin.toml");
        fs::write(
            &manifest_path,
            format!(
                r#"
manifest_version = 1
id = "blocked.plugin"
name = "Blocked Plugin"
version = "1.0.0"
publisher = "blocked.publisher"
entry_component = "demo.wasm"

[release]
url = "demo.wasm"
sha256 = "{sha}"
signature = "{signature_b64}"
signing_key_id = "blocked-key"
"#
            ),
        )
        .await
        .expect("write manifest");

        let manager = PluginManager::new(reqwest::Client::new(), workspace.join("plugins"));
        manager.ensure_layout().await.expect("ensure layout");
        manager
            .add_trusted_key("blocked-key", &public_key_b64)
            .await
            .expect("add key");

        let err = manager
            .install_from_manifest_ref(manifest_path.to_str().expect("manifest path utf8"))
            .await
            .expect_err("install should fail");
        assert!(
            err.to_string().contains("is not allowlisted"),
            "unexpected error: {}",
            err
        );
    }
}
