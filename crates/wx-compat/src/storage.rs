use serde_json::Value;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use thiserror::Error;

pub const DEFAULT_MAX_STORAGE_KEY_BYTES: usize = 128;
pub const DEFAULT_MAX_STORAGE_VALUE_BYTES: usize = 16 * 1024;
pub const DEFAULT_MAX_STORAGE_SCOPE_BYTES: usize = 128 * 1024;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StorageScope {
    pub user_did: String,
    pub merchant_did: String,
    pub skill_id: String,
}

impl StorageScope {
    pub fn new(
        user_did: impl Into<String>,
        merchant_did: impl Into<String>,
        skill_id: impl Into<String>,
    ) -> Self {
        Self {
            user_did: user_did.into(),
            merchant_did: merchant_did.into(),
            skill_id: skill_id.into(),
        }
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum StorageError {
    #[error("storage key is empty")]
    EmptyKey,

    #[error("storage key contains a NUL byte")]
    KeyContainsNul,

    #[error("storage key is too large")]
    KeyTooLarge,

    #[error("storage key is sensitive")]
    SensitiveKey,

    #[error("storage value is too large")]
    ValueTooLarge,

    #[error("storage value is not JSON-safe")]
    ValueNotJsonSafe,

    #[error("storage quota exceeded")]
    QuotaExceeded,

    #[error("storage lock is poisoned")]
    LockPoisoned,
}

pub trait ScopedStorage {
    fn get_storage(&self, scope: &StorageScope, key: &str) -> Result<Option<Value>, StorageError>;
    fn set_storage(
        &self,
        scope: &StorageScope,
        key: impl Into<String>,
        value: Value,
    ) -> Result<(), StorageError>;
    fn remove_storage(
        &self,
        scope: &StorageScope,
        key: &str,
    ) -> Result<Option<Value>, StorageError>;
    fn clear_storage(&self, scope: &StorageScope) -> Result<(), StorageError>;
}

#[derive(Debug, Clone, Default)]
pub struct InMemoryScopedStorage {
    inner: Arc<Mutex<BTreeMap<StorageScope, BTreeMap<String, Value>>>>,
}

impl InMemoryScopedStorage {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ScopedStorage for InMemoryScopedStorage {
    fn get_storage(&self, scope: &StorageScope, key: &str) -> Result<Option<Value>, StorageError> {
        validate_key(key)?;
        let guard = self.inner.lock().map_err(|_| StorageError::LockPoisoned)?;
        Ok(guard.get(scope).and_then(|values| values.get(key).cloned()))
    }

    fn set_storage(
        &self,
        scope: &StorageScope,
        key: impl Into<String>,
        value: Value,
    ) -> Result<(), StorageError> {
        let key = key.into();
        validate_key(&key)?;
        validate_value(&value)?;
        let mut guard = self.inner.lock().map_err(|_| StorageError::LockPoisoned)?;
        let scope_values = guard.entry(scope.clone()).or_default();
        let previous = scope_values.insert(key.clone(), value);
        if scope_size_bytes(scope_values)? > DEFAULT_MAX_STORAGE_SCOPE_BYTES {
            if let Some(previous) = previous {
                scope_values.insert(key, previous);
            } else {
                scope_values.remove(&key);
            }
            return Err(StorageError::QuotaExceeded);
        }
        Ok(())
    }

    fn remove_storage(
        &self,
        scope: &StorageScope,
        key: &str,
    ) -> Result<Option<Value>, StorageError> {
        validate_key(key)?;
        let mut guard = self.inner.lock().map_err(|_| StorageError::LockPoisoned)?;
        Ok(guard.get_mut(scope).and_then(|values| values.remove(key)))
    }

    fn clear_storage(&self, scope: &StorageScope) -> Result<(), StorageError> {
        let mut guard = self.inner.lock().map_err(|_| StorageError::LockPoisoned)?;
        guard.remove(scope);
        Ok(())
    }
}

fn validate_key(key: &str) -> Result<(), StorageError> {
    if key.trim().is_empty() {
        return Err(StorageError::EmptyKey);
    }
    if key.contains('\0') {
        return Err(StorageError::KeyContainsNul);
    }
    if key.len() > DEFAULT_MAX_STORAGE_KEY_BYTES {
        return Err(StorageError::KeyTooLarge);
    }
    let normalized = key.to_ascii_lowercase();
    if [
        "token",
        "authorization",
        "signature",
        "secret",
        "private",
        "credential",
        "phone",
        "address",
        "filecontent",
    ]
    .iter()
    .any(|sensitive| normalized.contains(sensitive))
    {
        return Err(StorageError::SensitiveKey);
    }
    Ok(())
}

fn validate_value(value: &Value) -> Result<(), StorageError> {
    let serialized = serde_json::to_vec(value).map_err(|_| StorageError::ValueNotJsonSafe)?;
    if serialized.len() > DEFAULT_MAX_STORAGE_VALUE_BYTES {
        return Err(StorageError::ValueTooLarge);
    }
    Ok(())
}

fn scope_size_bytes(values: &BTreeMap<String, Value>) -> Result<usize, StorageError> {
    values.iter().try_fold(0usize, |total, (key, value)| {
        let value_bytes = serde_json::to_vec(value).map_err(|_| StorageError::ValueNotJsonSafe)?;
        Ok(total + key.len() + value_bytes.len())
    })
}
