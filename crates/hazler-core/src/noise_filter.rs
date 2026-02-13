//! Noise Filter Module
//!
//! This module implements smart filtering of repetitive response patterns
//! to reduce false positives from WAF blocks and other noise.
//!
//! The filter tracks combinations of (status_code, content_length) and drops
//! URLs that match patterns occurring more than a threshold number of times.

use std::collections::HashMap;

/// Key for tracking response patterns
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ResponsePattern {
    pub status_code: u16,
    pub content_length: usize,
}

impl ResponsePattern {
    /// Create a new response pattern
    pub fn new(status_code: u16, content_length: usize) -> Self {
        Self {
            status_code,
            content_length,
        }
    }
}

/// Noise filter for detecting and filtering repetitive response patterns
#[derive(Debug, Clone)]
pub struct NoiseFilter {
    /// Map of response patterns to their occurrence count
    pattern_counts: HashMap<ResponsePattern, usize>,
    /// Threshold for considering a pattern as noise (default: 5)
    threshold: usize,
    /// Whether the filter is enabled
    enabled: bool,
}

impl NoiseFilter {
    /// Create a new noise filter with default threshold (5)
    pub fn new() -> Self {
        Self {
            pattern_counts: HashMap::new(),
            threshold: 5,
            enabled: true,
        }
    }

    /// Create a noise filter with a custom threshold
    pub fn with_threshold(threshold: usize) -> Self {
        Self {
            pattern_counts: HashMap::new(),
            threshold,
            enabled: true,
        }
    }

    /// Create a disabled noise filter (pass-through mode)
    pub fn disabled() -> Self {
        Self {
            pattern_counts: HashMap::new(),
            threshold: 5,
            enabled: false,
        }
    }

    /// Record a response pattern and return true if it should be filtered (is noise)
    ///
    /// # Arguments
    ///
    /// * `status_code` - HTTP status code of the response
    /// * `content_length` - Content length of the response body
    ///
    /// # Returns
    ///
    /// Returns `true` if this pattern has occurred more than the threshold times
    /// and should be filtered out, `false` otherwise.
    pub fn should_filter(&mut self, status_code: u16, content_length: usize) -> bool {
        if !self.enabled {
            return false;
        }

        let pattern = ResponsePattern::new(status_code, content_length);

        // Increment the count for this pattern
        let count = self.pattern_counts.entry(pattern.clone()).or_insert(0);
        *count += 1;

        // Filter if count exceeds threshold
        *count > self.threshold
    }

    /// Record a response pattern without filtering decision
    ///
    /// Use this to track patterns without immediately filtering them.
    /// You can later query with `is_noise` to check if a pattern is noise.
    pub fn record_pattern(&mut self, status_code: u16, content_length: usize) {
        if !self.enabled {
            return;
        }

        let pattern = ResponsePattern::new(status_code, content_length);
        let count = self.pattern_counts.entry(pattern).or_insert(0);
        *count += 1;
    }

    /// Check if a response pattern is considered noise without recording it
    ///
    /// # Returns
    ///
    /// Returns `true` if the pattern count exceeds the threshold
    pub fn is_noise(&self, status_code: u16, content_length: usize) -> bool {
        if !self.enabled {
            return false;
        }

        let pattern = ResponsePattern::new(status_code, content_length);
        self.pattern_counts
            .get(&pattern)
            .map(|&count| count > self.threshold)
            .unwrap_or(false)
    }

    /// Get the count for a specific pattern
    pub fn get_count(&self, status_code: u16, content_length: usize) -> usize {
        let pattern = ResponsePattern::new(status_code, content_length);
        self.pattern_counts.get(&pattern).copied().unwrap_or(0)
    }

    /// Get total number of unique patterns tracked
    pub fn pattern_count(&self) -> usize {
        self.pattern_counts.len()
    }

    /// Get total number of responses tracked
    pub fn total_responses(&self) -> usize {
        self.pattern_counts.values().sum()
    }

    /// Get statistics about filtered patterns
    pub fn get_stats(&self) -> NoiseFilterStats {
        let mut stats = NoiseFilterStats::default();

        for count in self.pattern_counts.values() {
            if *count > self.threshold {
                stats.filtered_patterns += 1;
                stats.filtered_responses += count;
            } else {
                stats.normal_patterns += 1;
                stats.normal_responses += count;
            }
        }

        stats
    }

    /// Clear all tracked patterns (useful for testing or resetting state)
    pub fn clear(&mut self) {
        self.pattern_counts.clear();
    }
}

impl Default for NoiseFilter {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about noise filtering
#[derive(Debug, Clone, Default)]
pub struct NoiseFilterStats {
    /// Number of unique patterns considered noise
    pub filtered_patterns: usize,
    /// Total responses filtered as noise
    pub filtered_responses: usize,
    /// Number of unique normal patterns
    pub normal_patterns: usize,
    /// Total normal responses
    pub normal_responses: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_noise_filter_creation() {
        let filter = NoiseFilter::new();
        assert_eq!(filter.threshold, 5);
        assert!(filter.enabled);
        assert_eq!(filter.pattern_count(), 0);
    }

    #[test]
    fn test_custom_threshold() {
        let filter = NoiseFilter::with_threshold(10);
        assert_eq!(filter.threshold, 10);
    }

    #[test]
    fn test_disabled_filter() {
        let mut filter = NoiseFilter::disabled();
        assert!(!filter.enabled);

        // Should never filter when disabled
        for _ in 0..10 {
            assert!(!filter.should_filter(403, 512));
        }
    }

    #[test]
    fn test_filter_threshold() {
        let mut filter = NoiseFilter::with_threshold(3);

        // First 3 occurrences should not be filtered
        assert!(!filter.should_filter(403, 512));
        assert!(!filter.should_filter(403, 512));
        assert!(!filter.should_filter(403, 512));

        // 4th and beyond should be filtered
        assert!(filter.should_filter(403, 512));
        assert!(filter.should_filter(403, 512));
    }

    #[test]
    fn test_different_patterns() {
        let mut filter = NoiseFilter::with_threshold(2);

        // Different patterns should be tracked separately
        assert!(!filter.should_filter(403, 512)); // Pattern 1, count 1
        assert!(!filter.should_filter(404, 1024)); // Pattern 2, count 1
        assert!(!filter.should_filter(403, 512)); // Pattern 1, count 2

        // Pattern 1 should now be filtered (count > 2)
        assert!(filter.should_filter(403, 512)); // Pattern 1, count 3

        // Pattern 2 should still not be filtered
        assert!(!filter.should_filter(404, 1024)); // Pattern 2, count 2
    }

    #[test]
    fn test_record_pattern() {
        let mut filter = NoiseFilter::with_threshold(2);

        // Record patterns without filtering
        filter.record_pattern(500, 256);
        filter.record_pattern(500, 256);
        filter.record_pattern(500, 256);

        // Now check if it's noise
        assert!(filter.is_noise(500, 256));
        assert!(!filter.is_noise(200, 1024));
    }

    #[test]
    fn test_get_count() {
        let mut filter = NoiseFilter::new();

        assert_eq!(filter.get_count(403, 512), 0);

        filter.record_pattern(403, 512);
        assert_eq!(filter.get_count(403, 512), 1);

        filter.record_pattern(403, 512);
        assert_eq!(filter.get_count(403, 512), 2);
    }

    #[test]
    fn test_stats() {
        let mut filter = NoiseFilter::with_threshold(2);

        // Add some normal responses
        filter.record_pattern(200, 1024);
        filter.record_pattern(200, 1024);

        // Add some noise responses
        filter.record_pattern(403, 512);
        filter.record_pattern(403, 512);
        filter.record_pattern(403, 512);
        filter.record_pattern(403, 512);

        let stats = filter.get_stats();
        assert_eq!(stats.normal_patterns, 1); // 200 OK pattern
        assert_eq!(stats.normal_responses, 2);
        assert_eq!(stats.filtered_patterns, 1); // 403 pattern
        assert_eq!(stats.filtered_responses, 4);
    }

    #[test]
    fn test_clear() {
        let mut filter = NoiseFilter::new();

        filter.record_pattern(403, 512);
        filter.record_pattern(404, 256);
        assert_eq!(filter.pattern_count(), 2);

        filter.clear();
        assert_eq!(filter.pattern_count(), 0);
    }

    #[test]
    fn test_waf_blocking_scenario() {
        let mut filter = NoiseFilter::with_threshold(5);

        // Simulate WAF blocking with same response
        for i in 1..=10 {
            let should_filter = filter.should_filter(403, 512);

            if i <= 5 {
                assert!(!should_filter, "Response {} should not be filtered", i);
            } else {
                assert!(should_filter, "Response {} should be filtered as noise", i);
            }
        }

        // Normal response should not be affected
        assert!(!filter.should_filter(200, 2048));
    }

    #[test]
    fn test_multiple_waf_patterns() {
        let mut filter = NoiseFilter::with_threshold(3);

        // Simulate different WAF responses
        for _ in 0..5 {
            filter.record_pattern(403, 512); // Pattern 1
            filter.record_pattern(429, 256); // Pattern 2
        }

        // Both patterns should be noise
        assert!(filter.is_noise(403, 512));
        assert!(filter.is_noise(429, 256));

        // Legitimate response should not be noise
        assert!(!filter.is_noise(200, 4096));
    }
}
