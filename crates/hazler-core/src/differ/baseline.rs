//! Baseline storage and management for response comparison
//!
//! This module handles saving and loading baseline responses for comparison.
//! Baselines are stored in JSON format and can be used to detect changes
//! in web application responses over time.

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use crate::differ::simhash::SimHash;
use crate::differ::noise_filter::NormalizedResponse;

/// A stored baseline entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineEntry {
    /// URL of the response
    pub url: String,
    /// SimHash of the response
    pub hash: SimHash,
    /// Normalized content
    pub normalized: NormalizedResponse,
    /// Timestamp when baseline was saved
    pub timestamp: u64,
}

/// Baseline storage container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineStorage {
    /// Map of URL to baseline entry
    entries: HashMap<String, BaselineEntry>,
    /// Version of the storage format
    version: String,
}

impl BaselineStorage {
    /// Create a new empty baseline storage
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            version: "1.0".to_string(),
        }
    }

    /// Add or update a baseline entry
    pub fn insert(&mut self, url: String, hash: SimHash, normalized: NormalizedResponse) {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.entries.insert(url.clone(), BaselineEntry {
            url,
            hash,
            normalized,
            timestamp,
        });
    }

    /// Get a baseline entry by URL
    pub fn get(&self, url: &str) -> Option<&BaselineEntry> {
        self.entries.get(url)
    }

    /// Get all entries
    pub fn entries(&self) -> &HashMap<String, BaselineEntry> {
        &self.entries
    }

    /// Get number of entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if storage is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Remove an entry
    pub fn remove(&mut self, url: &str) -> Option<BaselineEntry> {
        self.entries.remove(url)
    }

    /// Clear all entries
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

impl Default for BaselineStorage {
    fn default() -> Self {
        Self::new()
    }
}

/// Baseline manager for file I/O operations
pub struct BaselineManager {
    storage: BaselineStorage,
    file_path: String,
}

impl BaselineManager {
    /// Create a new baseline manager
    pub fn new(file_path: String) -> Self {
        Self {
            storage: BaselineStorage::new(),
            file_path,
        }
    }

    /// Load baseline from file
    pub fn load(&mut self) -> anyhow::Result<()> {
        if !Path::new(&self.file_path).exists() {
            return Ok(()); // No baseline file yet
        }

        let content = fs::read_to_string(&self.file_path)?;
        self.storage = serde_json::from_str(&content)?;
        Ok(())
    }

    /// Save baseline to file
    pub fn save(&self) -> anyhow::Result<()> {
        let content = serde_json::to_string_pretty(&self.storage)?;
        
        // Create parent directory if it doesn't exist
        if let Some(parent) = Path::new(&self.file_path).parent() {
            fs::create_dir_all(parent)?;
        }
        
        fs::write(&self.file_path, content)?;
        Ok(())
    }

    /// Save a baseline entry
    pub fn save_baseline(
        &mut self,
        url: &str,
        hash: SimHash,
        normalized: NormalizedResponse,
    ) -> anyhow::Result<()> {
        self.storage.insert(url.to_string(), hash, normalized);
        self.save()?;
        Ok(())
    }

    /// Load a baseline entry
    pub fn load_baseline(&self, url: &str) -> anyhow::Result<Option<BaselineEntry>> {
        Ok(self.storage.get(url).cloned())
    }

    /// Get all baseline URLs
    pub fn get_urls(&self) -> Vec<String> {
        self.storage.entries.keys().cloned().collect()
    }

    /// Get baseline storage
    pub fn storage(&self) -> &BaselineStorage {
        &self.storage
    }

    /// Get mutable baseline storage
    pub fn storage_mut(&mut self) -> &mut BaselineStorage {
        &mut self.storage
    }

    /// Get file path
    pub fn file_path(&self) -> &str {
        &self.file_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_baseline_storage_new() {
        let storage = BaselineStorage::new();
        assert!(storage.is_empty());
        assert_eq!(storage.len(), 0);
    }

    #[test]
    fn test_baseline_storage_insert() {
        let mut storage = BaselineStorage::new();
        let hash = SimHash::new(12345);
        let normalized = NormalizedResponse {
            content: "test".to_string(),
            removed_patterns: Vec::new(),
        };

        storage.insert("https://example.com".to_string(), hash, normalized);
        assert_eq!(storage.len(), 1);
        assert!(storage.get("https://example.com").is_some());
    }

    #[test]
    fn test_baseline_storage_get() {
        let mut storage = BaselineStorage::new();
        let hash = SimHash::new(12345);
        let normalized = NormalizedResponse {
            content: "test".to_string(),
            removed_patterns: Vec::new(),
        };

        storage.insert("https://example.com".to_string(), hash, normalized);
        let entry = storage.get("https://example.com");
        assert!(entry.is_some());
        assert_eq!(entry.unwrap().url, "https://example.com");
    }

    #[test]
    fn test_baseline_storage_remove() {
        let mut storage = BaselineStorage::new();
        let hash = SimHash::new(12345);
        let normalized = NormalizedResponse {
            content: "test".to_string(),
            removed_patterns: Vec::new(),
        };

        storage.insert("https://example.com".to_string(), hash, normalized);
        assert_eq!(storage.len(), 1);

        storage.remove("https://example.com");
        assert_eq!(storage.len(), 0);
    }

    #[test]
    fn test_baseline_storage_clear() {
        let mut storage = BaselineStorage::new();
        let hash = SimHash::new(12345);
        let normalized = NormalizedResponse {
            content: "test".to_string(),
            removed_patterns: Vec::new(),
        };

        storage.insert("url1".to_string(), hash, normalized.clone());
        storage.insert("url2".to_string(), hash, normalized);
        assert_eq!(storage.len(), 2);

        storage.clear();
        assert!(storage.is_empty());
    }

    #[test]
    fn test_baseline_manager_save_load() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("baseline.json").to_string_lossy().to_string();
        
        // Save baseline
        {
            let mut manager = BaselineManager::new(file_path.clone());
            let hash = SimHash::new(12345);
            let normalized = NormalizedResponse {
                content: "test content".to_string(),
                removed_patterns: vec!["timestamp".to_string()],
            };
            
            manager.save_baseline("https://example.com", hash, normalized).unwrap();
        }

        // Load baseline
        {
            let mut manager = BaselineManager::new(file_path.clone());
            manager.load().unwrap();
            
            let entry = manager.load_baseline("https://example.com").unwrap();
            assert!(entry.is_some());
            assert_eq!(entry.unwrap().url, "https://example.com");
        }
    }

    #[test]
    fn test_baseline_manager_load_nonexistent() {
        let mut manager = BaselineManager::new("/tmp/nonexistent_baseline_test.json".to_string());
        let result = manager.load();
        assert!(result.is_ok()); // Should not fail if file doesn't exist
    }

    #[test]
    fn test_baseline_manager_get_urls() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("baseline.json").to_string_lossy().to_string();
        let mut manager = BaselineManager::new(file_path);
        
        let hash = SimHash::new(12345);
        let normalized = NormalizedResponse {
            content: "test".to_string(),
            removed_patterns: Vec::new(),
        };

        manager.save_baseline("url1", hash, normalized.clone()).unwrap();
        manager.save_baseline("url2", hash, normalized).unwrap();

        let urls = manager.get_urls();
        assert_eq!(urls.len(), 2);
        assert!(urls.contains(&"url1".to_string()));
        assert!(urls.contains(&"url2".to_string()));
    }

    #[test]
    fn test_baseline_serialization() {
        let mut storage = BaselineStorage::new();
        let hash = SimHash::new(12345);
        let normalized = NormalizedResponse {
            content: "test".to_string(),
            removed_patterns: vec!["pattern1".to_string()],
        };

        storage.insert("https://example.com".to_string(), hash, normalized);

        // Serialize
        let json = serde_json::to_string(&storage).unwrap();
        assert!(json.contains("example.com"));

        // Deserialize
        let deserialized: BaselineStorage = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.len(), 1);
        assert!(deserialized.get("https://example.com").is_some());
    }
}
