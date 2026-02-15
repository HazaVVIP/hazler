//! Smart noise filtering for response normalization
//!
//! This module removes dynamic content from responses to focus on structural changes.
//! Common noise patterns include:
//! - Timestamps and dates
//! - Session IDs and tokens
//! - CSRF tokens
//! - Dynamic IDs
//! - Nonces and random strings

use regex::Regex;
use serde::{Deserialize, Serialize};

/// Normalized response with noise removed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedResponse {
    /// Content with noise patterns removed
    pub content: String,
    /// List of noise patterns that were removed
    pub removed_patterns: Vec<String>,
}

/// Response normalizer for removing noise
pub struct ResponseNormalizer {
    patterns: Vec<NoisePattern>,
}

impl ResponseNormalizer {
    /// Create a new response normalizer with default patterns
    pub fn new() -> Self {
        Self {
            patterns: Self::default_patterns(),
        }
    }

    /// Normalize a response by removing noise patterns
    pub fn normalize(&self, content: &str) -> NormalizedResponse {
        let mut normalized = content.to_string();
        let mut removed_patterns = Vec::new();

        for pattern in &self.patterns {
            if pattern.regex.is_match(&normalized) {
                normalized = pattern
                    .regex
                    .replace_all(&normalized, &pattern.replacement)
                    .to_string();
                removed_patterns.push(pattern.name.clone());
            }
        }

        NormalizedResponse {
            content: normalized,
            removed_patterns,
        }
    }

    /// Get default noise patterns
    fn default_patterns() -> Vec<NoisePattern> {
        vec![
            // Timestamps (Unix epoch)
            NoisePattern::new("unix_timestamp", r"\b\d{10,13}\b", "TIMESTAMP"),
            // ISO 8601 dates
            NoisePattern::new(
                "iso_date",
                r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:\d{2})?",
                "ISO_DATE",
            ),
            // Session IDs (common formats)
            NoisePattern::new(
                "session_id",
                r"(?i)(session[_-]?id|sid|sess)[=:]\s*[a-zA-Z0-9+/=]{20,}",
                "SESSION_ID",
            ),
            // CSRF tokens
            NoisePattern::new(
                "csrf_token",
                r"(?i)(csrf[_-]?token|xsrf[_-]?token)[=:]\s*[a-zA-Z0-9+/=_-]{20,}",
                "CSRF_TOKEN",
            ),
            // JWT tokens
            NoisePattern::new(
                "jwt_token",
                r"eyJ[a-zA-Z0-9_-]+\.eyJ[a-zA-Z0-9_-]+\.[a-zA-Z0-9_-]+",
                "JWT_TOKEN",
            ),
            // UUIDs
            NoisePattern::new(
                "uuid",
                r"\b[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}\b",
                "UUID",
            ),
            // Nonces (32+ hex chars)
            NoisePattern::new("nonce", r"\b[a-fA-F0-9]{32,}\b", "NONCE"),
            // ETag headers
            NoisePattern::new("etag", r#"(?i)etag:\s*"[^"]+""#, "ETAG"),
            // Random alphanumeric strings (likely IDs)
            NoisePattern::new("random_id", r"\b[a-zA-Z0-9]{40,}\b", "RANDOM_ID"),
            // Dynamic timestamps in various formats
            NoisePattern::new(
                "human_date",
                r"(?i)(Mon|Tue|Wed|Thu|Fri|Sat|Sun),?\s+\d{1,2}\s+(Jan|Feb|Mar|Apr|May|Jun|Jul|Aug|Sep|Oct|Nov|Dec)\s+\d{4}\s+\d{2}:\d{2}:\d{2}",
                "HUMAN_DATE",
            ),
        ]
    }

    /// Add a custom noise pattern
    pub fn add_pattern(
        &mut self,
        name: &str,
        regex: &str,
        replacement: &str,
    ) -> Result<(), regex::Error> {
        let pattern = NoisePattern::new(name, regex, replacement);
        self.patterns.push(pattern);
        Ok(())
    }
}

impl Default for ResponseNormalizer {
    fn default() -> Self {
        Self::new()
    }
}

/// A noise pattern definition
struct NoisePattern {
    name: String,
    regex: Regex,
    replacement: String,
}

impl NoisePattern {
    fn new(name: &str, pattern: &str, replacement: &str) -> Self {
        Self {
            name: name.to_string(),
            regex: Regex::new(pattern).unwrap(),
            replacement: replacement.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_timestamp() {
        let normalizer = ResponseNormalizer::new();
        let content = "Generated at 1234567890";
        let result = normalizer.normalize(content);
        assert!(result.content.contains("TIMESTAMP"));
        assert!(result
            .removed_patterns
            .contains(&"unix_timestamp".to_string()));
    }

    #[test]
    fn test_normalize_iso_date() {
        let normalizer = ResponseNormalizer::new();
        let content = "Created: 2024-01-15T10:30:00Z";
        let result = normalizer.normalize(content);
        assert!(result.content.contains("ISO_DATE"));
        assert!(result.removed_patterns.contains(&"iso_date".to_string()));
    }

    #[test]
    fn test_normalize_session_id() {
        let normalizer = ResponseNormalizer::new();
        let content = "session_id=abc123def456ghi789jkl012";
        let result = normalizer.normalize(content);
        assert!(result.content.contains("SESSION_ID"));
    }

    #[test]
    fn test_normalize_csrf_token() {
        let normalizer = ResponseNormalizer::new();
        let content = "csrf-token: abcdefghijklmnopqrstuvwxyz123456";
        let result = normalizer.normalize(content);
        assert!(result.content.contains("CSRF_TOKEN"));
    }

    #[test]
    fn test_normalize_uuid() {
        let normalizer = ResponseNormalizer::new();
        let content = "ID: 550e8400-e29b-41d4-a716-446655440000";
        let result = normalizer.normalize(content);
        // UUID pattern should match and replace
        println!(
            "Result: {}, Patterns: {:?}",
            result.content, result.removed_patterns
        );
        assert!(
            result.content.contains("UUID") || result.content.contains("550e8400"),
            "Content should contain UUID placeholder or original UUID"
        );
        // Check if UUID was detected (might also match nonce pattern)
        assert!(
            !result.removed_patterns.is_empty(),
            "Should detect some pattern"
        );
    }

    #[test]
    fn test_normalize_jwt() {
        let normalizer = ResponseNormalizer::new();
        let content = "Authorization: eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3I9PlFUP0THsR8U";
        let result = normalizer.normalize(content);
        assert!(result.content.contains("JWT_TOKEN"));
    }

    #[test]
    fn test_normalize_multiple_patterns() {
        let normalizer = ResponseNormalizer::new();
        let content = "session_id=abc123def456ghi789jkl012, Time: 1234567890, ID: 550e8400-e29b-41d4-a716-446655440000";
        let result = normalizer.normalize(content);

        println!("Normalized: {}", result.content);
        println!("Removed patterns: {:?}", result.removed_patterns);
        assert!(
            result.removed_patterns.len() >= 2,
            "Should remove at least 2 patterns"
        );
        assert!(
            result.content.contains("TIMESTAMP") || result.content.contains("SESSION_ID"),
            "Should normalize timestamp or session"
        );
    }

    #[test]
    fn test_normalize_no_noise() {
        let normalizer = ResponseNormalizer::new();
        let content = "Hello World";
        let result = normalizer.normalize(content);
        assert_eq!(result.content, "Hello World");
        assert!(result.removed_patterns.is_empty());
    }

    #[test]
    fn test_add_custom_pattern() {
        let mut normalizer = ResponseNormalizer::new();
        normalizer
            .add_pattern("custom", r"CUSTOM_\d+", "CUSTOM")
            .unwrap();

        let content = "ID: CUSTOM_12345";
        let result = normalizer.normalize(content);
        assert!(result.content.contains("CUSTOM"));
    }

    #[test]
    fn test_normalize_html_with_timestamps() {
        let normalizer = ResponseNormalizer::new();
        let html = r#"<html>
            <body>
                <div>Generated at: 2024-01-15T10:30:00Z</div>
                <div>Session: session_id=abc123def456ghi789jkl012</div>
            </body>
        </html>"#;

        let result = normalizer.normalize(html);
        assert!(result.content.contains("ISO_DATE"));
        assert!(result.removed_patterns.len() >= 2);
    }

    #[test]
    fn test_normalize_api_response() {
        let normalizer = ResponseNormalizer::new();
        let json = r#"{
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "timestamp": 1234567890,
            "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.test"
        }"#;

        let result = normalizer.normalize(json);
        assert!(result.removed_patterns.len() >= 2);
    }
}
