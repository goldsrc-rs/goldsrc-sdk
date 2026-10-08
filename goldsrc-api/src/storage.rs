//! Unified cross-platform storage abstraction for GoldSrc plugins.
//!
//! Provides a safe, isolated key-value persistence interface.
//! On `wasm32` guests, operations are routed directly to the engine sandbox host.
//! On native targets (such as host tests, unit testing, or CLI tools), an in-memory
//! thread-safe registry is utilized transparently without requiring `#[cfg(target_arch = "wasm32")]`.

#[cfg(not(target_arch = "wasm32"))]
use std::sync::{LazyLock, RwLock};

#[cfg(not(target_arch = "wasm32"))]
use std::collections::HashMap;

#[cfg(not(target_arch = "wasm32"))]
type MockStorageMap = HashMap<String, HashMap<String, Vec<u8>>>;

#[cfg(not(target_arch = "wasm32"))]
static MOCK_STORAGE: LazyLock<RwLock<MockStorageMap>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

/// Retrieves a binary blob from the named bucket by key.
pub fn get(bucket: &str, key: &str) -> Option<Vec<u8>> {
    #[cfg(target_arch = "wasm32")]
    {
        crate::bindings::goldsrc::engine::api::host_storage_get(bucket, key)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let lock = MOCK_STORAGE.read().unwrap_or_else(|e| e.into_inner());
        lock.get(bucket).and_then(|b| b.get(key).cloned())
    }
}

/// Stores a binary blob into the named bucket by key.
/// Returns `true` if saved successfully, or `false` if rejected by host policy.
pub fn set(bucket: &str, key: &str, val: &[u8]) -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        crate::bindings::goldsrc::engine::api::host_storage_set(bucket, key, val)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut lock = MOCK_STORAGE.write().unwrap_or_else(|e| e.into_inner());
        lock.entry(bucket.to_string())
            .or_default()
            .insert(key.to_string(), val.to_vec());
        true
    }
}

/// Deletes a key from the named bucket.
/// Returns `true` if key was deleted, or `false` if it did not exist or was rejected.
pub fn delete(bucket: &str, key: &str) -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        crate::bindings::goldsrc::engine::api::host_storage_delete(bucket, key)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut lock = MOCK_STORAGE.write().unwrap_or_else(|e| e.into_inner());
        lock.get_mut(bucket)
            .map(|b| b.remove(key).is_some())
            .unwrap_or(false)
    }
}

/// Atomically increments or decrements a 64-bit integer in the named bucket.
/// Returns the new integer value after applying delta.
pub fn fetch_add(bucket: &str, key: &str, delta: i64) -> i64 {
    #[cfg(target_arch = "wasm32")]
    {
        crate::bindings::goldsrc::engine::api::host_storage_fetch_add(bucket, key, delta)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let mut lock = MOCK_STORAGE.write().unwrap_or_else(|e| e.into_inner());
        let b = lock.entry(bucket.to_string()).or_default();
        let current = if let Some(bytes) = b.get(key) {
            if bytes.len() == 8 {
                i64::from_le_bytes(bytes.as_slice().try_into().unwrap_or_default())
            } else {
                0
            }
        } else {
            0
        };
        let next = current.saturating_add(delta);
        b.insert(key.to_string(), next.to_le_bytes().to_vec());
        next
    }
}

/// Clears mock storage state (intended for native test harnesses).
#[cfg(not(target_arch = "wasm32"))]
pub fn clear_mock_storage() {
    let mut lock = MOCK_STORAGE.write().unwrap_or_else(|e| e.into_inner());
    lock.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_mock_roundtrip() {
        clear_mock_storage();
        assert_eq!(get("test_bucket", "key1"), None);
        assert!(set("test_bucket", "key1", b"hello world"));
        assert_eq!(get("test_bucket", "key1"), Some(b"hello world".to_vec()));
        assert!(delete("test_bucket", "key1"));
        assert_eq!(get("test_bucket", "key1"), None);
    }

    #[test]
    fn test_storage_mock_fetch_add() {
        clear_mock_storage();
        assert_eq!(fetch_add("stats", "kills", 1), 1);
        assert_eq!(fetch_add("stats", "kills", 5), 6);
        assert_eq!(fetch_add("stats", "kills", -2), 4);
    }
}
