//! # Response Diffing Module
//!
//! This module provides advanced response comparison and diffing capabilities for detecting
//! changes in web application responses over time.
//!
//! ## Features
//!
//! - **SimHash**: Fast fuzzy hashing algorithm for near-duplicate detection
//! - **Response Clustering**: Group similar responses using K-means and DBSCAN
//! - **Smart Noise Filtering**: Remove dynamic content and focus on structural changes
//! - **Change Detection**: Compare responses before/after to identify meaningful changes
//! - **Baseline Mode**: Save and compare against baseline responses
//!
//! ## Example
//!
//! ```no_run
//! use hazler_core::differ::{ResponseDiffer, DifferConfig};
//!
//! let config = DifferConfig::default();
//! let differ = ResponseDiffer::new(config);
//!
//! // Compare two responses
//! let similarity = differ.compare_responses(&response1, &response2);
//! println!("Similarity: {:.2}%", similarity * 100.0);
//! ```

mod baseline;
mod change_detection;
mod clustering;
mod noise_filter;
mod simhash;

pub use baseline::{BaselineManager, BaselineStorage};
pub use change_detection::{ChangeDetector, ChangeType, ResponseChange};
pub use clustering::{ClusteringAlgorithm, DBSCANClusterer, KMeansClusterer, ResponseCluster};
pub use noise_filter::{NormalizedResponse, ResponseNormalizer};
pub use simhash::{SimHash, SimHashCalculator};

use serde::{Deserialize, Serialize};

/// Configuration for response differ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifferConfig {
    /// Similarity threshold for considering responses as duplicates (0.0 to 1.0)
    pub similarity_threshold: f64,
    /// Enable smart noise filtering
    pub enable_noise_filtering: bool,
    /// Enable clustering
    pub enable_clustering: bool,
    /// Clustering algorithm to use
    pub clustering_algorithm: String,
    /// Number of clusters for K-means
    pub num_clusters: usize,
    /// DBSCAN epsilon parameter
    pub dbscan_epsilon: f64,
    /// DBSCAN minimum points
    pub dbscan_min_points: usize,
}

impl Default for DifferConfig {
    fn default() -> Self {
        Self {
            similarity_threshold: 0.85,
            enable_noise_filtering: true,
            enable_clustering: false,
            clustering_algorithm: "kmeans".to_string(),
            num_clusters: 5,
            dbscan_epsilon: 0.3,
            dbscan_min_points: 2,
        }
    }
}

/// Main response differ
pub struct ResponseDiffer {
    config: DifferConfig,
    simhash_calculator: SimHashCalculator,
    normalizer: ResponseNormalizer,
    baseline_manager: Option<BaselineManager>,
}

impl ResponseDiffer {
    /// Create a new response differ with configuration
    pub fn new(config: DifferConfig) -> Self {
        Self {
            config: config.clone(),
            simhash_calculator: SimHashCalculator::new(),
            normalizer: ResponseNormalizer::new(),
            baseline_manager: None,
        }
    }

    /// Create a differ with baseline management
    pub fn with_baseline(config: DifferConfig, baseline_path: String) -> Self {
        let baseline_manager = BaselineManager::new(baseline_path);
        Self {
            config: config.clone(),
            simhash_calculator: SimHashCalculator::new(),
            normalizer: ResponseNormalizer::new(),
            baseline_manager: Some(baseline_manager),
        }
    }

    /// Compare two responses and return similarity score (0.0 to 1.0)
    pub fn compare_responses(&self, response1: &str, response2: &str) -> f64 {
        let hash1 = self.simhash_calculator.calculate(response1);
        let hash2 = self.simhash_calculator.calculate(response2);
        hash1.similarity(&hash2)
    }

    /// Compare normalized responses (with noise filtering)
    pub fn compare_normalized(&self, response1: &str, response2: &str) -> f64 {
        let norm1 = self.normalizer.normalize(response1);
        let norm2 = self.normalizer.normalize(response2);
        self.compare_responses(&norm1.content, &norm2.content)
    }

    /// Check if two responses are duplicates based on threshold
    pub fn are_duplicates(&self, response1: &str, response2: &str) -> bool {
        let similarity = if self.config.enable_noise_filtering {
            self.compare_normalized(response1, response2)
        } else {
            self.compare_responses(response1, response2)
        };
        similarity >= self.config.similarity_threshold
    }

    /// Save baseline responses
    pub fn save_baseline(&mut self, url: &str, response: &str) -> anyhow::Result<()> {
        if let Some(ref mut manager) = self.baseline_manager {
            let hash = self.simhash_calculator.calculate(response);
            let normalized = if self.config.enable_noise_filtering {
                self.normalizer.normalize(response)
            } else {
                NormalizedResponse {
                    content: response.to_string(),
                    removed_patterns: Vec::new(),
                }
            };
            manager.save_baseline(url, hash, normalized)?;
        }
        Ok(())
    }

    /// Compare against baseline
    pub fn compare_with_baseline(&self, url: &str, response: &str) -> Option<f64> {
        if let Some(ref manager) = self.baseline_manager {
            let current_hash = self.simhash_calculator.calculate(response);
            if let Ok(Some(baseline)) = manager.load_baseline(url) {
                return Some(current_hash.similarity(&baseline.hash));
            }
        }
        None
    }

    /// Get baseline manager
    pub fn baseline_manager(&self) -> Option<&BaselineManager> {
        self.baseline_manager.as_ref()
    }

    /// Get mutable baseline manager
    pub fn baseline_manager_mut(&mut self) -> Option<&mut BaselineManager> {
        self.baseline_manager.as_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_differ_creation() {
        let config = DifferConfig::default();
        let differ = ResponseDiffer::new(config);
        assert!(differ.baseline_manager.is_none());
    }

    #[test]
    fn test_differ_with_baseline() {
        let config = DifferConfig::default();
        let differ = ResponseDiffer::with_baseline(config, "/tmp/baseline.json".to_string());
        assert!(differ.baseline_manager.is_some());
    }

    #[test]
    fn test_compare_identical_responses() {
        let differ = ResponseDiffer::new(DifferConfig::default());
        let response = "<html><body>Hello World</body></html>";
        let similarity = differ.compare_responses(response, response);
        assert_eq!(similarity, 1.0);
    }

    #[test]
    fn test_compare_similar_responses() {
        let differ = ResponseDiffer::new(DifferConfig::default());
        let response1 = "<html><body>Hello World</body></html>";
        let response2 = "<html><body>Hello World!</body></html>";
        let similarity = differ.compare_responses(response1, response2);
        assert!(similarity > 0.8);
    }

    #[test]
    fn test_are_duplicates() {
        let differ = ResponseDiffer::new(DifferConfig::default());
        let response1 = "<html><body>Test</body></html>";
        let response2 = "<html><body>Test</body></html>";
        assert!(differ.are_duplicates(response1, response2));
    }
}
