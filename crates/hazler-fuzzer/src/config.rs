//! Configuration for the fuzzer module

use serde::{Deserialize, Serialize};

/// Configuration for the fuzzer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuzzerConfig {
    /// Enable pluralization mutations
    pub enable_pluralization: bool,
    
    /// Enable file extension mutations
    pub enable_extensions: bool,
    
    /// Enable API versioning mutations
    pub enable_versioning: bool,
    
    /// Enable parameter discovery
    pub enable_param_discovery: bool,
    
    /// Enable BOLA/IDOR detection
    pub enable_bola_detection: bool,
    
    /// API versions to test (e.g., ["v1", "v2", "v3"])
    pub api_versions: Vec<String>,
    
    /// File extensions to test
    pub file_extensions: Vec<String>,
    
    /// Maximum mutations per URL
    pub max_mutations: usize,
}

impl Default for FuzzerConfig {
    fn default() -> Self {
        Self {
            enable_pluralization: true,
            enable_extensions: true,
            enable_versioning: true,
            enable_param_discovery: true,
            enable_bola_detection: true,
            api_versions: vec![
                "v1".to_string(),
                "v2".to_string(),
                "v3".to_string(),
                "v4".to_string(),
            ],
            file_extensions: vec![
                "json".to_string(),
                "xml".to_string(),
                "html".to_string(),
                "txt".to_string(),
                "php".to_string(),
                "asp".to_string(),
                "aspx".to_string(),
                "jsp".to_string(),
            ],
            max_mutations: 100,
        }
    }
}

impl FuzzerConfig {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Create an aggressive configuration with more mutations
    pub fn aggressive() -> Self {
        Self {
            enable_pluralization: true,
            enable_extensions: true,
            enable_versioning: true,
            enable_param_discovery: true,
            enable_bola_detection: true,
            api_versions: vec![
                "v1".to_string(),
                "v2".to_string(),
                "v3".to_string(),
                "v4".to_string(),
                "v5".to_string(),
            ],
            file_extensions: vec![
                "json".to_string(),
                "xml".to_string(),
                "html".to_string(),
                "txt".to_string(),
                "php".to_string(),
                "asp".to_string(),
                "aspx".to_string(),
                "jsp".to_string(),
                "yaml".to_string(),
                "yml".to_string(),
                "csv".to_string(),
            ],
            max_mutations: 200,
        }
    }
    
    /// Create a minimal configuration for targeted fuzzing
    pub fn minimal() -> Self {
        Self {
            enable_pluralization: true,
            enable_extensions: false,
            enable_versioning: true,
            enable_param_discovery: false,
            enable_bola_detection: false,
            api_versions: vec!["v1".to_string(), "v2".to_string()],
            file_extensions: vec!["json".to_string()],
            max_mutations: 20,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = FuzzerConfig::default();
        assert!(config.enable_pluralization);
        assert!(config.enable_extensions);
        assert!(config.enable_versioning);
        assert_eq!(config.api_versions.len(), 4);
    }

    #[test]
    fn test_aggressive_config() {
        let config = FuzzerConfig::aggressive();
        assert!(config.enable_pluralization);
        assert_eq!(config.api_versions.len(), 5);
        assert_eq!(config.max_mutations, 200);
    }

    #[test]
    fn test_minimal_config() {
        let config = FuzzerConfig::minimal();
        assert!(config.enable_pluralization);
        assert!(!config.enable_extensions);
        assert_eq!(config.max_mutations, 20);
    }
}
