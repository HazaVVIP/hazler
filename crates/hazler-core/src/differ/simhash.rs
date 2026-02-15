//! SimHash implementation for fuzzy document hashing
//!
//! SimHash is a locality-sensitive hashing technique that generates similar hashes
//! for similar documents. It's particularly useful for near-duplicate detection.
//!
//! ## Algorithm
//!
//! 1. Extract features (words/tokens) from the document
//! 2. Hash each feature to get a 64-bit hash
//! 3. For each hash bit position, accumulate +1 if bit is 1, -1 if bit is 0
//! 4. Final hash: set bit to 1 if accumulator > 0, else 0
//! 5. Similarity is measured by Hamming distance between hashes

use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// SimHash value (64-bit fingerprint)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SimHash(pub u64);

impl SimHash {
    /// Create a new SimHash from a u64 value
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    /// Calculate Hamming distance between two SimHash values
    pub fn hamming_distance(&self, other: &SimHash) -> u32 {
        (self.0 ^ other.0).count_ones()
    }

    /// Calculate similarity score (0.0 to 1.0)
    /// Based on inverse of normalized Hamming distance
    pub fn similarity(&self, other: &SimHash) -> f64 {
        let distance = self.hamming_distance(other);
        1.0 - (distance as f64 / 64.0)
    }

    /// Check if two hashes are similar within a threshold
    pub fn is_similar(&self, other: &SimHash, threshold: u32) -> bool {
        self.hamming_distance(other) <= threshold
    }
}

/// SimHash calculator for generating hashes from text
pub struct SimHashCalculator {
    // Configuration could be added here for customization
}

impl SimHashCalculator {
    /// Create a new SimHash calculator
    pub fn new() -> Self {
        Self {}
    }

    /// Calculate SimHash for a given text
    pub fn calculate(&self, text: &str) -> SimHash {
        // Extract features (words)
        let features = self.extract_features(text);

        if features.is_empty() {
            return SimHash::new(0);
        }

        // Initialize accumulator vector for 64 bits
        let mut v = [0i32; 64];

        // For each feature
        for feature in features {
            // Hash the feature
            let hash = self.hash_feature(&feature);

            // Update accumulator based on hash bits
            for i in 0..64 {
                let bit = (hash >> i) & 1;
                if bit == 1 {
                    v[i] += 1;
                } else {
                    v[i] -= 1;
                }
            }
        }

        // Generate final hash
        let mut simhash: u64 = 0;
        for i in 0..64 {
            if v[i] > 0 {
                simhash |= 1u64 << i;
            }
        }

        SimHash::new(simhash)
    }

    /// Extract features (words) from text
    fn extract_features(&self, text: &str) -> Vec<String> {
        text.split_whitespace()
            .map(|word| {
                // Normalize: lowercase and remove punctuation
                word.to_lowercase()
                    .chars()
                    .filter(|c| c.is_alphanumeric())
                    .collect()
            })
            .filter(|word: &String| !word.is_empty())
            .collect()
    }

    /// Hash a single feature to u64
    fn hash_feature(&self, feature: &str) -> u64 {
        let mut hasher = DefaultHasher::new();
        feature.hash(&mut hasher);
        hasher.finish()
    }
}

impl Default for SimHashCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simhash_identical() {
        let calc = SimHashCalculator::new();
        let text = "The quick brown fox jumps over the lazy dog";
        let hash1 = calc.calculate(text);
        let hash2 = calc.calculate(text);
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.similarity(&hash2), 1.0);
    }

    #[test]
    fn test_simhash_similar() {
        let calc = SimHashCalculator::new();
        let text1 = "The quick brown fox jumps over the lazy dog";
        let text2 = "The quick brown fox jumps over the lazy cat";
        let hash1 = calc.calculate(text1);
        let hash2 = calc.calculate(text2);

        let similarity = hash1.similarity(&hash2);
        assert!(
            similarity > 0.8,
            "Similar texts should have high similarity"
        );
    }

    #[test]
    fn test_simhash_different() {
        let calc = SimHashCalculator::new();
        let text1 = "The quick brown fox";
        let text2 = "Hello world from Rust";
        let hash1 = calc.calculate(text1);
        let hash2 = calc.calculate(text2);

        let similarity = hash1.similarity(&hash2);
        assert!(
            similarity < 0.5,
            "Different texts should have low similarity"
        );
    }

    #[test]
    fn test_hamming_distance() {
        let hash1 = SimHash::new(0b1010);
        let hash2 = SimHash::new(0b1000);
        assert_eq!(hash1.hamming_distance(&hash2), 1);
    }

    #[test]
    fn test_hamming_distance_identical() {
        let hash1 = SimHash::new(12345);
        let hash2 = SimHash::new(12345);
        assert_eq!(hash1.hamming_distance(&hash2), 0);
    }

    #[test]
    fn test_is_similar() {
        let hash1 = SimHash::new(0b1010);
        let hash2 = SimHash::new(0b1000);
        assert!(hash1.is_similar(&hash2, 1));
        assert!(!hash1.is_similar(&hash2, 0));
    }

    #[test]
    fn test_empty_text() {
        let calc = SimHashCalculator::new();
        let hash = calc.calculate("");
        assert_eq!(hash.0, 0);
    }

    #[test]
    fn test_feature_extraction() {
        let calc = SimHashCalculator::new();
        let features = calc.extract_features("Hello, World! Test 123");
        assert!(features.contains(&"hello".to_string()));
        assert!(features.contains(&"world".to_string()));
        assert!(features.contains(&"test".to_string()));
        assert!(features.contains(&"123".to_string()));
    }

    #[test]
    fn test_similarity_range() {
        let hash1 = SimHash::new(0);
        let hash2 = SimHash::new(u64::MAX);
        let similarity = hash1.similarity(&hash2);
        assert!(similarity >= 0.0 && similarity <= 1.0);
    }

    #[test]
    fn test_html_simhash() {
        let calc = SimHashCalculator::new();
        let html1 = "<html><body><h1>Welcome</h1><p>Hello World</p></body></html>";
        let html2 = "<html><body><h1>Welcome</h1><p>Hello Universe</p></body></html>";
        let hash1 = calc.calculate(html1);
        let hash2 = calc.calculate(html2);

        let similarity = hash1.similarity(&hash2);
        assert!(similarity > 0.7, "Similar HTML should have high similarity");
    }
}
