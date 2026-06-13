use crate::integrity::{compute_package_digest, PackageIntegrityPolicy, PackageIntegrityReport};
use crate::package::{load_skill_with_integrity_policy, LoadedSkill};
use crate::resolver::{resolve_skill_path, SkillPackageError};
use serde_json::{json, Value};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkillReference {
    pub kind: SkillReferenceKind,
    pub skill_id: String,
    pub publisher_did: Option<String>,
    pub version: Option<String>,
    pub digest: Option<String>,
}

impl SkillReference {
    pub fn local_path(path: impl Into<PathBuf>) -> Self {
        Self {
            kind: SkillReferenceKind::LocalPath(path.into()),
            skill_id: "local".to_owned(),
            publisher_did: None,
            version: None,
            digest: None,
        }
    }

    pub fn registry_id(
        skill_id: impl Into<String>,
        selector: SkillVersionSelector,
        publisher_did: Option<String>,
    ) -> Self {
        Self {
            kind: SkillReferenceKind::RegistryId {
                selector,
                prerelease: false,
            },
            skill_id: skill_id.into(),
            publisher_did,
            version: None,
            digest: None,
        }
    }

    pub fn registry_id_with_prerelease(
        skill_id: impl Into<String>,
        selector: SkillVersionSelector,
        publisher_did: Option<String>,
        allow_prerelease: bool,
    ) -> Self {
        Self {
            kind: SkillReferenceKind::RegistryId {
                selector,
                prerelease: allow_prerelease,
            },
            skill_id: skill_id.into(),
            publisher_did,
            version: None,
            digest: None,
        }
    }

    pub fn package_url(
        skill_id: impl Into<String>,
        package_url: impl Into<String>,
        publisher_did: impl Into<String>,
        version: impl Into<String>,
        digest: impl Into<String>,
    ) -> Self {
        Self {
            kind: SkillReferenceKind::PackageUrl(package_url.into()),
            skill_id: skill_id.into(),
            publisher_did: Some(publisher_did.into()),
            version: Some(version.into()),
            digest: Some(digest.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkillReferenceKind {
    LocalPath(PathBuf),
    PackageUrl(String),
    RegistryId {
        selector: SkillVersionSelector,
        prerelease: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SkillVersionSelector {
    Latest,
    Pinned(String),
    Rollback { before_version: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistrySkillEntry {
    pub skill_id: String,
    pub publisher_did: String,
    pub version: String,
    pub digest: String,
    pub package_path: PathBuf,
    pub package_url: Option<String>,
    pub prerelease: bool,
}

impl RegistrySkillEntry {
    pub fn cache_key(&self) -> SkillCacheKey {
        SkillCacheKey {
            publisher_did: self.publisher_did.clone(),
            skill_id: self.skill_id.clone(),
            version: self.version.clone(),
            digest: self.digest.clone(),
        }
    }

    pub fn reference(&self) -> SkillReference {
        SkillReference {
            kind: match &self.package_url {
                Some(url) => SkillReferenceKind::PackageUrl(url.clone()),
                None => SkillReferenceKind::LocalPath(self.package_path.clone()),
            },
            skill_id: self.skill_id.clone(),
            publisher_did: Some(self.publisher_did.clone()),
            version: Some(self.version.clone()),
            digest: Some(self.digest.clone()),
        }
    }
}

pub trait SkillRegistry {
    fn resolve_skill(
        &self,
        reference: &SkillReference,
    ) -> Result<RegistrySkillEntry, SkillPackageError>;
}

#[derive(Debug, Clone, Default)]
pub struct LocalSkillRegistry {
    entries: Vec<RegistrySkillEntry>,
}

impl LocalSkillRegistry {
    pub fn new(entries: impl IntoIterator<Item = RegistrySkillEntry>) -> Self {
        Self {
            entries: entries.into_iter().collect(),
        }
    }

    pub fn entries(&self) -> &[RegistrySkillEntry] {
        &self.entries
    }
}

impl SkillRegistry for LocalSkillRegistry {
    fn resolve_skill(
        &self,
        reference: &SkillReference,
    ) -> Result<RegistrySkillEntry, SkillPackageError> {
        match &reference.kind {
            SkillReferenceKind::LocalPath(path) => registry_entry_from_local_path(reference, path),
            SkillReferenceKind::PackageUrl(url) => self
                .entries
                .iter()
                .find(|entry| entry.package_url.as_deref() == Some(url.as_str()))
                .filter(|entry| entry_matches_reference(entry, reference))
                .cloned()
                .ok_or_else(|| registry_not_found(reference)),
            SkillReferenceKind::RegistryId {
                selector,
                prerelease,
            } => select_registry_entry(&self.entries, reference, selector, *prerelease),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SkillCacheKey {
    pub publisher_did: String,
    pub skill_id: String,
    pub version: String,
    pub digest: String,
}

impl SkillCacheKey {
    pub fn directory_name(&self) -> String {
        format!(
            "{}__{}__{}__{}",
            sanitize_cache_part(&self.publisher_did),
            sanitize_cache_part(&self.skill_id),
            sanitize_cache_part(&self.version),
            sanitize_cache_part(&self.digest)
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CachedSkillMetadata {
    pub key: SkillCacheKey,
    pub package_source: PackageSourceSummary,
    pub package_ref: String,
    pub integrity: PackageIntegrityReport,
    pub readonly: bool,
    pub quarantined: bool,
}

impl CachedSkillMetadata {
    pub fn audit_summary(&self) -> Value {
        json!({
            "packageSource": self.package_source,
            "packageRef": self.package_ref,
            "publisherDid": self.key.publisher_did,
            "skillId": self.key.skill_id,
            "version": self.key.version,
            "digest": {
                "algorithm": self.integrity.digest.algorithm,
                "value": self.integrity.digest.value,
            },
            "supplyChain": {
                "status": self.integrity.status.as_str(),
                "trustedPublisher": self.integrity.trusted_publisher,
                "quarantine": self.integrity.quarantine,
                "productionReady": self.integrity.production_ready,
                "issueCodes": self.integrity.issue_codes,
            },
            "cache": {
                "readonly": self.readonly,
                "quarantined": self.quarantined,
            }
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CachedSkill {
    pub root: PathBuf,
    pub metadata: CachedSkillMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PackageSourceSummary {
    pub source_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

pub struct SkillCache {
    root: PathBuf,
    rollback_pins: BTreeMap<String, SkillCacheKey>,
}

impl SkillCache {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            rollback_pins: BTreeMap::new(),
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn load_or_insert(
        &mut self,
        entry: &RegistrySkillEntry,
        policy: &PackageIntegrityPolicy,
    ) -> Result<CachedSkill, SkillPackageError> {
        let source_root = resolve_skill_path(&entry.package_path)?;
        let actual_digest = compute_package_digest(&source_root)?;
        if actual_digest.value != entry.digest {
            return Err(SkillPackageError::PackageQuarantined {
                reason: "digest_mismatch".to_owned(),
            });
        }
        load_skill_with_integrity_policy(&source_root, policy)?;

        let key = entry.cache_key();
        let cache_dir = self.root.join(key.directory_name());
        let root = if cache_dir.exists() {
            set_readonly_recursive(&cache_dir)?;
            cache_dir
        } else {
            fs::create_dir_all(&cache_dir).map_err(|source| SkillPackageError::ReadFile {
                path: cache_dir.clone(),
                source,
            })?;
            copy_package_dir(&source_root, &cache_dir)?;
            set_readonly_recursive(&cache_dir)?;
            cache_dir
        };

        let loaded = load_skill_with_integrity_policy(&root, policy)?;
        if loaded.integrity.digest.value != entry.digest {
            return Err(SkillPackageError::PackageQuarantined {
                reason: "digest_mismatch".to_owned(),
            });
        }
        let metadata = CachedSkillMetadata {
            key,
            package_source: PackageSourceSummary {
                source_type: if entry.package_url.is_some() {
                    "package-url".to_owned()
                } else {
                    "registry-local-path".to_owned()
                },
                url: entry.package_url.as_deref().map(redacted_package_url),
            },
            package_ref: format!("sha256:{}", loaded.integrity.digest.value),
            integrity: loaded.integrity,
            readonly: is_readonly_package(&root)?,
            quarantined: false,
        };
        Ok(CachedSkill { root, metadata })
    }

    pub fn load_skill(
        &mut self,
        entry: &RegistrySkillEntry,
        policy: &PackageIntegrityPolicy,
    ) -> Result<(LoadedSkill, CachedSkillMetadata), SkillPackageError> {
        let cached = self.load_or_insert(entry, policy)?;
        let loaded = load_skill_with_integrity_policy(&cached.root, policy)?;
        Ok((loaded, cached.metadata))
    }

    pub fn pin_rollback(&mut self, key: SkillCacheKey) {
        self.rollback_pins
            .insert(rollback_scope(&key.publisher_did, &key.skill_id), key);
    }

    pub fn rollback_pin(&self, publisher_did: &str, skill_id: &str) -> Option<&SkillCacheKey> {
        self.rollback_pins
            .get(&rollback_scope(publisher_did, skill_id))
    }

    pub fn evict_unpinned(&self, retain: impl IntoIterator<Item = SkillCacheKey>) -> usize {
        let retain = retain.into_iter().collect::<BTreeSet<_>>();
        let pinned = self
            .rollback_pins
            .values()
            .cloned()
            .collect::<BTreeSet<_>>();
        let Ok(entries) = fs::read_dir(&self.root) else {
            return 0;
        };
        let mut removed = 0;
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            let keep_by_retain = retain.iter().any(|key| key.directory_name() == name);
            let keep_by_pin = pinned.iter().any(|key| key.directory_name() == name);
            if keep_by_retain || keep_by_pin {
                continue;
            }
            let _ = make_writable_recursive(&path);
            if fs::remove_dir_all(&path).is_ok() {
                removed += 1;
            }
        }
        removed
    }
}

pub fn load_registry_skill<R: SkillRegistry>(
    registry: &R,
    cache: &mut SkillCache,
    reference: &SkillReference,
    policy: &PackageIntegrityPolicy,
) -> Result<(LoadedSkill, CachedSkillMetadata), SkillPackageError> {
    let entry = registry.resolve_skill(reference)?;
    cache.load_skill(&entry, policy)
}

fn registry_entry_from_local_path(
    reference: &SkillReference,
    path: &Path,
) -> Result<RegistrySkillEntry, SkillPackageError> {
    let root = resolve_skill_path(path)?;
    let digest = compute_package_digest(&root)?;
    Ok(RegistrySkillEntry {
        skill_id: reference.skill_id.clone(),
        publisher_did: reference
            .publisher_did
            .clone()
            .unwrap_or_else(|| "did:wba:local-dev.example".to_owned()),
        version: reference
            .version
            .clone()
            .unwrap_or_else(|| "0.0.0-local".to_owned()),
        digest: reference.digest.clone().unwrap_or(digest.value),
        package_path: root,
        package_url: None,
        prerelease: true,
    })
}

fn select_registry_entry(
    entries: &[RegistrySkillEntry],
    reference: &SkillReference,
    selector: &SkillVersionSelector,
    allow_prerelease: bool,
) -> Result<RegistrySkillEntry, SkillPackageError> {
    let mut candidates = entries
        .iter()
        .filter(|entry| entry_matches_reference(entry, reference))
        .filter(|entry| allow_prerelease || !entry.prerelease)
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| compare_versions(&left.version, &right.version));

    match selector {
        SkillVersionSelector::Latest => candidates
            .last()
            .map(|entry| (*entry).clone())
            .ok_or_else(|| registry_not_found(reference)),
        SkillVersionSelector::Pinned(version) => candidates
            .into_iter()
            .find(|entry| &entry.version == version)
            .cloned()
            .ok_or_else(|| registry_not_found(reference)),
        SkillVersionSelector::Rollback { before_version } => candidates
            .into_iter()
            .filter(|entry| compare_versions(&entry.version, before_version) == Ordering::Less)
            .next_back()
            .cloned()
            .ok_or_else(|| registry_not_found(reference)),
    }
}

fn entry_matches_reference(entry: &RegistrySkillEntry, reference: &SkillReference) -> bool {
    if entry.skill_id != reference.skill_id {
        return false;
    }
    if reference
        .publisher_did
        .as_ref()
        .is_some_and(|publisher_did| publisher_did != &entry.publisher_did)
    {
        return false;
    }
    if reference
        .version
        .as_ref()
        .is_some_and(|version| version != &entry.version)
    {
        return false;
    }
    if reference
        .digest
        .as_ref()
        .is_some_and(|digest| digest != &entry.digest)
    {
        return false;
    }
    true
}

fn registry_not_found(reference: &SkillReference) -> SkillPackageError {
    SkillPackageError::PackageQuarantined {
        reason: format!("registry_entry_not_found:{}", reference.skill_id),
    }
}

fn copy_package_dir(source: &Path, target: &Path) -> Result<(), SkillPackageError> {
    for entry in fs::read_dir(source).map_err(|source_error| SkillPackageError::ReadFile {
        path: source.to_path_buf(),
        source: source_error,
    })? {
        let entry = entry.map_err(|source_error| SkillPackageError::ReadFile {
            path: source.to_path_buf(),
            source: source_error,
        })?;
        let source_path = entry.path();
        let target_path = target.join(entry.file_name());
        let metadata = fs::symlink_metadata(&source_path).map_err(|source_error| {
            SkillPackageError::ReadFile {
                path: source_path.clone(),
                source: source_error,
            }
        })?;
        if metadata.file_type().is_symlink() {
            let canonical =
                source_path
                    .canonicalize()
                    .map_err(|source_error| SkillPackageError::ReadFile {
                        path: source_path.clone(),
                        source: source_error,
                    })?;
            fs::copy(&canonical, &target_path).map_err(|source_error| {
                SkillPackageError::ReadFile {
                    path: canonical,
                    source: source_error,
                }
            })?;
        } else if metadata.is_dir() {
            fs::create_dir_all(&target_path).map_err(|source_error| {
                SkillPackageError::ReadFile {
                    path: target_path.clone(),
                    source: source_error,
                }
            })?;
            copy_package_dir(&source_path, &target_path)?;
        } else if metadata.is_file() {
            fs::copy(&source_path, &target_path).map_err(|source_error| {
                SkillPackageError::ReadFile {
                    path: source_path.clone(),
                    source: source_error,
                }
            })?;
        }
    }
    Ok(())
}

fn set_readonly_recursive(path: &Path) -> Result<(), SkillPackageError> {
    let metadata = fs::metadata(path).map_err(|source| SkillPackageError::ReadFile {
        path: path.to_path_buf(),
        source,
    })?;
    if metadata.is_dir() {
        for entry in fs::read_dir(path).map_err(|source| SkillPackageError::ReadFile {
            path: path.to_path_buf(),
            source,
        })? {
            let entry = entry.map_err(|source| SkillPackageError::ReadFile {
                path: path.to_path_buf(),
                source,
            })?;
            set_readonly_recursive(&entry.path())?;
        }
    }
    let mut permissions = metadata.permissions();
    permissions.set_readonly(true);
    fs::set_permissions(path, permissions).map_err(|source| SkillPackageError::ReadFile {
        path: path.to_path_buf(),
        source,
    })
}

fn make_writable_recursive(path: &Path) -> Result<(), SkillPackageError> {
    let metadata = fs::metadata(path).map_err(|source| SkillPackageError::ReadFile {
        path: path.to_path_buf(),
        source,
    })?;
    make_path_writable(path, &metadata)?;
    if metadata.is_dir() {
        for entry in fs::read_dir(path).map_err(|source| SkillPackageError::ReadFile {
            path: path.to_path_buf(),
            source,
        })? {
            let entry = entry.map_err(|source| SkillPackageError::ReadFile {
                path: path.to_path_buf(),
                source,
            })?;
            make_writable_recursive(&entry.path())?;
        }
    }
    Ok(())
}

#[cfg(unix)]
fn make_path_writable(path: &Path, metadata: &fs::Metadata) -> Result<(), SkillPackageError> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = metadata.permissions();
    let write_bits = if metadata.is_dir() { 0o700 } else { 0o600 };
    permissions.set_mode(permissions.mode() | write_bits);
    fs::set_permissions(path, permissions).map_err(|source| SkillPackageError::ReadFile {
        path: path.to_path_buf(),
        source,
    })
}

#[cfg(not(unix))]
fn make_path_writable(path: &Path, metadata: &fs::Metadata) -> Result<(), SkillPackageError> {
    let mut permissions = metadata.permissions();
    permissions.set_readonly(false);
    fs::set_permissions(path, permissions).map_err(|source| SkillPackageError::ReadFile {
        path: path.to_path_buf(),
        source,
    })
}

fn is_readonly_package(path: &Path) -> Result<bool, SkillPackageError> {
    let metadata = fs::metadata(path).map_err(|source| SkillPackageError::ReadFile {
        path: path.to_path_buf(),
        source,
    })?;
    if !metadata.permissions().readonly() {
        return Ok(false);
    }
    if metadata.is_dir() {
        for entry in fs::read_dir(path).map_err(|source| SkillPackageError::ReadFile {
            path: path.to_path_buf(),
            source,
        })? {
            let entry = entry.map_err(|source| SkillPackageError::ReadFile {
                path: path.to_path_buf(),
                source,
            })?;
            if !is_readonly_package(&entry.path())? {
                return Ok(false);
            }
        }
    }
    Ok(true)
}

fn sanitize_cache_part(value: &str) -> String {
    value
        .chars()
        .map(|character| match character {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '.' | '-' | '_' => character,
            _ => '_',
        })
        .collect()
}

fn rollback_scope(publisher_did: &str, skill_id: &str) -> String {
    format!("{publisher_did}/{skill_id}")
}

fn compare_versions(left: &str, right: &str) -> Ordering {
    let (left_main, left_pre) = split_prerelease(left);
    let (right_main, right_pre) = split_prerelease(right);
    let left_parts = version_parts(left_main);
    let right_parts = version_parts(right_main);
    let max_len = left_parts.len().max(right_parts.len());
    for index in 0..max_len {
        let left_part = left_parts.get(index).copied().unwrap_or(0);
        let right_part = right_parts.get(index).copied().unwrap_or(0);
        match left_part.cmp(&right_part) {
            Ordering::Equal => {}
            ordering => return ordering,
        }
    }
    match (left_pre, right_pre) {
        (None, None) => Ordering::Equal,
        (None, Some(_)) => Ordering::Greater,
        (Some(_), None) => Ordering::Less,
        (Some(left_pre), Some(right_pre)) => left_pre.cmp(right_pre),
    }
}

fn split_prerelease(version: &str) -> (&str, Option<&str>) {
    match version.split_once('-') {
        Some((main, prerelease)) => (main, Some(prerelease)),
        None => (version, None),
    }
}

fn version_parts(version: &str) -> Vec<u64> {
    version
        .split('.')
        .map(|part| part.parse::<u64>().unwrap_or(0))
        .collect()
}

fn redacted_package_url(url: &str) -> String {
    let lower = url.to_ascii_lowercase();
    if [
        "authorization",
        "signature",
        "token",
        "secret",
        "credential",
        "private",
        "password",
    ]
    .iter()
    .any(|marker| lower.contains(marker))
    {
        return "[REDACTED]".to_owned();
    }
    url.split(['?', '#']).next().unwrap_or(url).to_owned()
}
