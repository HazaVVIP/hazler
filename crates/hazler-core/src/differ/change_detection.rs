//! Change detection for before/after response comparison
//!
//! This module detects and categorizes changes between response versions.

use crate::differ::simhash::SimHash;
use serde::{Deserialize, Serialize};

/// Type of change detected
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeType {
    /// Content added
    Addition,
    /// Content removed
    Deletion,
    /// Content modified
    Modification,
    /// Structure changed
    Structural,
    /// No significant change
    None,
}

/// A detected change between responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseChange {
    /// URL of the response
    pub url: String,
    /// Type of change
    pub change_type: ChangeType,
    /// Similarity score (0.0 to 1.0)
    pub similarity: f64,
    /// Size difference in bytes
    pub size_diff: i64,
    /// Description of the change
    pub description: String,
    /// Whether the change is significant
    pub is_significant: bool,
}

/// Change detector for comparing responses
pub struct ChangeDetector {
    /// Threshold for considering a change significant (0.0 to 1.0)
    significance_threshold: f64,
    /// Minimum size difference to consider significant (bytes)
    min_size_diff: usize,
}

impl ChangeDetector {
    /// Create a new change detector
    pub fn new(significance_threshold: f64, min_size_diff: usize) -> Self {
        Self {
            significance_threshold,
            min_size_diff,
        }
    }

    /// Detect changes between two responses
    pub fn detect_change(
        &self,
        url: &str,
        before: &str,
        after: &str,
        before_hash: &SimHash,
        after_hash: &SimHash,
    ) -> ResponseChange {
        let similarity = before_hash.similarity(after_hash);
        let size_before = before.len();
        let size_after = after.len();
        let size_diff = size_after as i64 - size_before as i64;

        let change_type =
            self.determine_change_type(similarity, size_diff, size_before, size_after);
        let is_significant = self.is_change_significant(similarity, size_diff.abs() as usize);
        let description = self.generate_description(&change_type, similarity, size_diff);

        ResponseChange {
            url: url.to_string(),
            change_type,
            similarity,
            size_diff,
            description,
            is_significant,
        }
    }

    /// Determine the type of change
    fn determine_change_type(
        &self,
        similarity: f64,
        size_diff: i64,
        size_before: usize,
        size_after: usize,
    ) -> ChangeType {
        if similarity > 0.99 {
            return ChangeType::None;
        }

        if similarity < 0.7 {
            return ChangeType::Structural;
        }

        if size_before == 0 && size_after > 0 {
            return ChangeType::Addition;
        }

        if size_before > 0 && size_after == 0 {
            return ChangeType::Deletion;
        }

        let size_change_ratio = size_diff.abs() as f64 / size_before.max(1) as f64;

        if size_change_ratio > 0.3 {
            if size_diff > 0 {
                ChangeType::Addition
            } else {
                ChangeType::Deletion
            }
        } else {
            ChangeType::Modification
        }
    }

    /// Check if a change is significant
    fn is_change_significant(&self, similarity: f64, size_diff: usize) -> bool {
        let similarity_change = 1.0 - similarity;
        similarity_change >= self.significance_threshold || size_diff >= self.min_size_diff
    }

    /// Generate a human-readable description of the change
    fn generate_description(
        &self,
        change_type: &ChangeType,
        similarity: f64,
        size_diff: i64,
    ) -> String {
        match change_type {
            ChangeType::None => {
                format!(
                    "No significant change (similarity: {:.1}%)",
                    similarity * 100.0
                )
            }
            ChangeType::Addition => {
                format!(
                    "Content added (+{} bytes, similarity: {:.1}%)",
                    size_diff,
                    similarity * 100.0
                )
            }
            ChangeType::Deletion => {
                format!(
                    "Content removed ({} bytes, similarity: {:.1}%)",
                    size_diff,
                    similarity * 100.0
                )
            }
            ChangeType::Modification => {
                format!(
                    "Content modified ({}{}bytes, similarity: {:.1}%)",
                    if size_diff > 0 { "+" } else { "" },
                    size_diff,
                    similarity * 100.0
                )
            }
            ChangeType::Structural => {
                format!(
                    "Structural change detected (similarity: {:.1}%)",
                    similarity * 100.0
                )
            }
        }
    }

    /// Detect changes across multiple response pairs
    pub fn detect_changes(
        &self,
        comparisons: Vec<(&str, &str, &str, &SimHash, &SimHash)>,
    ) -> Vec<ResponseChange> {
        comparisons
            .into_iter()
            .map(|(url, before, after, before_hash, after_hash)| {
                self.detect_change(url, before, after, before_hash, after_hash)
            })
            .collect()
    }

    /// Filter significant changes from a list
    pub fn filter_significant(&self, changes: Vec<ResponseChange>) -> Vec<ResponseChange> {
        changes
            .into_iter()
            .filter(|change| change.is_significant)
            .collect()
    }
}

impl Default for ChangeDetector {
    fn default() -> Self {
        Self::new(0.15, 100) // 15% difference or 100 bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::differ::simhash::SimHashCalculator;

    #[test]
    fn test_no_change() {
        let detector = ChangeDetector::default();
        let calculator = SimHashCalculator::new();
        let content = "Hello World";
        let hash = calculator.calculate(content);

        let change = detector.detect_change("http://example.com", content, content, &hash, &hash);
        assert_eq!(change.change_type, ChangeType::None);
        assert!(!change.is_significant);
    }

    #[test]
    fn test_addition() {
        let detector = ChangeDetector::default();
        let calculator = SimHashCalculator::new();

        let before = "Hello";
        let after = "Hello World with lots more content added here for testing purposes";
        let hash_before = calculator.calculate(before);
        let hash_after = calculator.calculate(after);

        let change = detector.detect_change(
            "http://example.com",
            before,
            after,
            &hash_before,
            &hash_after,
        );
        // Should detect addition or structural change
        assert!(
            matches!(
                change.change_type,
                ChangeType::Addition | ChangeType::Modification | ChangeType::Structural
            ),
            "Change type should be Addition, Modification, or Structural but got {:?}",
            change.change_type
        );
        assert!(change.size_diff > 0);
    }

    #[test]
    fn test_deletion() {
        let detector = ChangeDetector::default();
        let calculator = SimHashCalculator::new();

        let before = "Hello World with lots of content to remove";
        let after = "Hello";
        let hash_before = calculator.calculate(before);
        let hash_after = calculator.calculate(after);

        let change = detector.detect_change(
            "http://example.com",
            before,
            after,
            &hash_before,
            &hash_after,
        );
        assert!(matches!(
            change.change_type,
            ChangeType::Deletion | ChangeType::Modification
        ));
        assert!(change.size_diff < 0);
    }

    #[test]
    fn test_modification() {
        let detector = ChangeDetector::default();
        let calculator = SimHashCalculator::new();

        let before = "Hello World";
        let after = "Hello Universe";
        let hash_before = calculator.calculate(before);
        let hash_after = calculator.calculate(after);

        let change = detector.detect_change(
            "http://example.com",
            before,
            after,
            &hash_before,
            &hash_after,
        );
        assert!(matches!(
            change.change_type,
            ChangeType::Modification | ChangeType::None
        ));
    }

    #[test]
    fn test_structural_change() {
        let detector = ChangeDetector::default();
        let calculator = SimHashCalculator::new();

        let before = "Apple Banana Cherry Date";
        let after = "Zebra Yak Xerus Walrus";
        let hash_before = calculator.calculate(before);
        let hash_after = calculator.calculate(after);

        let change = detector.detect_change(
            "http://example.com",
            before,
            after,
            &hash_before,
            &hash_after,
        );
        let similarity = hash_before.similarity(&hash_after);

        // Structural change if very different
        if similarity < 0.7 {
            assert_eq!(change.change_type, ChangeType::Structural);
        }
    }

    #[test]
    fn test_significant_change() {
        let detector = ChangeDetector::new(0.1, 50);
        let calculator = SimHashCalculator::new();

        let before = "Original content";
        let after = "Completely different content with many more words and characters";
        let hash_before = calculator.calculate(before);
        let hash_after = calculator.calculate(after);

        let change = detector.detect_change(
            "http://example.com",
            before,
            after,
            &hash_before,
            &hash_after,
        );
        assert!(change.is_significant);
    }

    #[test]
    fn test_filter_significant() {
        let detector = ChangeDetector::default();
        let calculator = SimHashCalculator::new();

        let changes = vec![
            detector.detect_change(
                "url1",
                "same",
                "same",
                &calculator.calculate("same"),
                &calculator.calculate("same"),
            ),
            detector.detect_change(
                "url2",
                "before",
                "after with significant change content added here",
                &calculator.calculate("before"),
                &calculator.calculate("after with significant change content added here"),
            ),
        ];

        let significant = detector.filter_significant(changes);
        // At least one should be significant
        assert!(significant.len() >= 1);
    }

    #[test]
    fn test_description_generation() {
        let detector = ChangeDetector::default();
        let calculator = SimHashCalculator::new();

        let before = "test";
        let after = "test";
        let hash = calculator.calculate(before);

        let change = detector.detect_change("url", before, after, &hash, &hash);
        assert!(change.description.contains("No significant change"));
    }
}
