//! Secret scanner implementation

use crate::entropy::{EntropyFinding, EntropyScanner};
use crate::patterns;
use regex::Regex;
use serde::{Deserialize, Serialize};

/// Severity level for findings
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
}

impl From<&str> for Severity {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "critical" => Severity::Critical,
            "high" => Severity::High,
            "medium" => Severity::Medium,
            "low" => Severity::Low,
            _ => Severity::Medium,
        }
    }
}

/// A secret or sensitive information finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// Type of secret found
    pub secret_type: String,
    /// Severity level
    pub severity: Severity,
    /// Description of the finding
    pub description: String,
    /// Line number where found (1-indexed)
    pub line: usize,
    /// Column number where found (1-indexed)
    pub column: usize,
    /// Context (surrounding text)
    pub context: String,
    /// Matched text (may be redacted for sensitive values)
    pub matched_text: String,
    /// File or URL where found
    pub location: String,
}

/// Secret scanner for detecting sensitive information
#[derive(Clone)]
pub struct SecretScanner {
    patterns: Vec<(String, Regex, String, String)>,
    entropy_scanner: EntropyScanner,
}

impl SecretScanner {
    /// Create a new secret scanner
    pub fn new() -> Self {
        let patterns = patterns::compile_patterns().expect("Failed to compile secret patterns");

        SecretScanner {
            patterns,
            entropy_scanner: EntropyScanner::new(),
        }
    }

    /// Scan text for secrets and sensitive information
    ///
    /// # Arguments
    ///
    /// * `text` - The text to scan
    /// * `location` - The file path or URL being scanned
    ///
    /// # Returns
    ///
    /// A vector of findings
    pub fn scan(&self, text: &str, location: &str) -> Vec<Finding> {
        let mut findings = Vec::new();
        // Track (line, column) positions already reported to avoid duplicates
        let mut seen_positions = std::collections::HashSet::new();

        for (line_idx, line) in text.lines().enumerate() {
            for (name, pattern, severity, description) in &self.patterns {
                // Use captures_iter for global search to find all matches in minified code
                for captures in pattern.captures_iter(line) {
                    let matched = captures.get(0).unwrap();
                    let matched_text = matched.as_str();

                    // Deduplicate: skip if another pattern already reported a finding
                    // at exactly the same position in the same file
                    let pos_key = (line_idx, matched.start());
                    if !seen_positions.insert(pos_key) {
                        continue;
                    }

                    // Get context (up to 50 chars before and after)
                    let start = matched.start().saturating_sub(50);
                    let end = (matched.end() + 50).min(line.len());
                    let context = &line[start..end];

                    // Redact sensitive parts of the matched text
                    let redacted = Self::redact_sensitive(matched_text, name);

                    findings.push(Finding {
                        secret_type: name.clone(),
                        severity: Severity::from(severity.as_str()),
                        description: description.clone(),
                        line: line_idx + 1,
                        column: matched.start() + 1,
                        context: context.to_string(),
                        matched_text: redacted,
                        location: location.to_string(),
                    });
                }
            }
        }

        findings
    }

    /// Redact sensitive information from matched text.
    ///
    /// Both the high-sensitivity (keep 4) and standard (keep 6) paths use a
    /// single forward pass with a ring buffer so that the byte boundaries for
    /// the prefix and suffix are located without allocating a `Vec<char>` and
    /// without iterating the string twice.
    fn redact_sensitive(text: &str, pattern_name: &str) -> String {
        // For high-sensitivity patterns, redact most of the value
        match pattern_name {
            name if name.contains("Secret")
                || name.contains("Private")
                || name.contains("Password") =>
            {
                // Keep first 4 and last 4 characters.
                const KEEP: usize = 4;
                const RING: usize = KEEP + 1;
                let mut ring = [0usize; RING];
                let mut char_count = 0usize;
                let mut prefix_end = text.len();
                for (byte_idx, _ch) in text.char_indices() {
                    ring[char_count % RING] = byte_idx;
                    char_count += 1;
                    if char_count == KEEP + 1 {
                        prefix_end = byte_idx;
                    }
                }
                if char_count > KEEP * 2 {
                    let suffix_start = ring[(char_count - KEEP) % RING];
                    format!("{}...{}", &text[..prefix_end], &text[suffix_start..])
                } else {
                    "***REDACTED***".to_string()
                }
            }
            _ => {
                // For other patterns, show a bit more (keep first 6 and last 6).
                const KEEP: usize = 6;
                const RING: usize = KEEP + 1;
                let mut ring = [0usize; RING];
                let mut char_count = 0usize;
                let mut prefix_end = text.len();
                for (byte_idx, _ch) in text.char_indices() {
                    ring[char_count % RING] = byte_idx;
                    char_count += 1;
                    if char_count == KEEP + 1 {
                        prefix_end = byte_idx;
                    }
                }
                if char_count > KEEP * 2 {
                    let suffix_start = ring[(char_count - KEEP) % RING];
                    format!("{}...{}", &text[..prefix_end], &text[suffix_start..])
                } else {
                    text.to_string()
                }
            }
        }
    }

    /// Scan text using both regex patterns and entropy analysis.
    ///
    /// Returns regex-based [`Finding`]s alongside entropy-based [`EntropyFinding`]s.
    /// Entropy findings complement regex findings by catching secrets that do not
    /// match any known pattern.
    pub fn scan_with_entropy(
        &self,
        text: &str,
        location: &str,
    ) -> (Vec<Finding>, Vec<EntropyFinding>) {
        let pattern_findings = self.scan(text, location);
        let entropy_findings = self.entropy_scanner.scan(text, location);
        (pattern_findings, entropy_findings)
    }

    /// Scan and return only high and critical severity findings
    pub fn scan_high_severity(&self, text: &str, location: &str) -> Vec<Finding> {
        self.scan(text, location)
            .into_iter()
            .filter(|f| matches!(f.severity, Severity::Critical | Severity::High))
            .collect()
    }

    /// Get statistics about findings by severity
    pub fn get_stats(findings: &[Finding]) -> FindingStats {
        let mut stats = FindingStats::default();

        for finding in findings {
            match finding.severity {
                Severity::Critical => stats.critical += 1,
                Severity::High => stats.high += 1,
                Severity::Medium => stats.medium += 1,
                Severity::Low => stats.low += 1,
            }
        }

        stats.total = findings.len();
        stats
    }
}

impl Default for SecretScanner {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about findings
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FindingStats {
    pub total: usize,
    pub critical: usize,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scanner_creation() {
        let scanner = SecretScanner::new();
        assert!(!scanner.patterns.is_empty());
    }

    #[test]
    fn test_aws_key_detection() {
        let scanner = SecretScanner::new();
        let code = "const AWS_KEY = 'AKIA1234567890ABCDEF';";
        let findings = scanner.scan(code, "test.js");

        assert!(!findings.is_empty());
        let aws_finding = findings.iter().find(|f| f.secret_type.contains("AWS"));
        assert!(aws_finding.is_some());
    }

    #[test]
    fn test_github_token_detection() {
        let scanner = SecretScanner::new();
        let code = "token = 'ghp_1234567890abcdefABCDEF1234567890ab'";
        let findings = scanner.scan(code, "config.js");

        assert!(!findings.is_empty());
        assert!(findings[0].secret_type.contains("GitHub"));
    }

    #[test]
    fn test_jwt_detection() {
        let scanner = SecretScanner::new();
        let code = "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";
        let findings = scanner.scan(code, "api.js");

        assert!(!findings.is_empty());
        assert!(findings.iter().any(|f| f.secret_type.contains("JWT")));
    }

    #[test]
    fn test_private_key_detection() {
        let scanner = SecretScanner::new();
        let code = "-----BEGIN RSA PRIVATE KEY-----\nMIIEpAIBAAKCAQEA...";
        let findings = scanner.scan(code, "key.pem");

        assert!(!findings.is_empty());
        assert!(findings[0].severity == Severity::Critical);
    }

    #[test]
    fn test_internal_ip_detection() {
        let scanner = SecretScanner::new();
        let code = "const API_URL = 'http://192.168.1.100:8080/api';";
        let findings = scanner.scan(code, "config.js");

        assert!(!findings.is_empty());
        assert!(findings
            .iter()
            .any(|f| f.secret_type.contains("Internal IP")));
    }

    #[test]
    fn test_high_severity_filtering() {
        let scanner = SecretScanner::new();
        let code = r#"
            const AWS_KEY = 'AKIA1234567890ABCDEF';
            const SERVER_ADDR = '192.168.1.100';
        "#;
        let all_findings = scanner.scan(code, "test.js");
        let high_findings = scanner.scan_high_severity(code, "test.js");

        // Should have at least two findings (AWS key + internal IP)
        assert!(all_findings.len() >= 2);
        // high_findings should only have AWS key (critical/high), not internal IP (medium)
        assert!(high_findings.len() < all_findings.len());
    }

    #[test]
    fn test_stats_calculation() {
        let findings = vec![
            Finding {
                secret_type: "AWS Key".to_string(),
                severity: Severity::Critical,
                description: "Test".to_string(),
                line: 1,
                column: 1,
                context: "".to_string(),
                matched_text: "".to_string(),
                location: "test".to_string(),
            },
            Finding {
                secret_type: "API Key".to_string(),
                severity: Severity::High,
                description: "Test".to_string(),
                line: 2,
                column: 1,
                context: "".to_string(),
                matched_text: "".to_string(),
                location: "test".to_string(),
            },
            Finding {
                secret_type: "Email".to_string(),
                severity: Severity::Low,
                description: "Test".to_string(),
                line: 3,
                column: 1,
                context: "".to_string(),
                matched_text: "".to_string(),
                location: "test".to_string(),
            },
        ];

        let stats = SecretScanner::get_stats(&findings);
        assert_eq!(stats.total, 3);
        assert_eq!(stats.critical, 1);
        assert_eq!(stats.high, 1);
        assert_eq!(stats.low, 1);
    }

    #[test]
    fn test_minified_code_multiple_secrets() {
        let scanner = SecretScanner::new();
        // Minified JavaScript with multiple secrets on one line (test values)
        let aws_test = "AKIA1234567890ABCDEF";
        let gh_test = "ghp_1234567890abcdefABCDEF1234567890ab";
        let stripe_test = format!("sk_live_{}", "12345678901234567890abcd");
        let minified = format!(
            r#"const a="{}",b="{}",c="{}";"#,
            aws_test, gh_test, stripe_test
        );
        let findings = scanner.scan(&minified, "minified.js");

        // Should find exactly 3 secrets (AWS key, GitHub token, Stripe key)
        assert_eq!(
            findings.len(),
            3,
            "Expected exactly 3 findings, got {}",
            findings.len()
        );

        // Verify each secret type is found
        assert!(findings.iter().any(|f| f.secret_type.contains("AWS")));
        assert!(findings.iter().any(|f| f.secret_type.contains("GitHub")));
        assert!(findings.iter().any(|f| f.secret_type.contains("Stripe")));
    }

    #[test]
    fn test_secrets_with_quotes() {
        let scanner = SecretScanner::new();
        // Test secrets surrounded by quotes (common in minified code)
        // Use variable names that won't trigger additional patterns
        let code = r#"const a="AKIA1234567890ABCDEF",b='ghp_1234567890abcdefABCDEF1234567890ab'"#;
        let findings = scanner.scan(code, "test.js");

        // Should find at least AWS and GitHub tokens (may find more generic patterns)
        assert!(
            findings.len() >= 2,
            "Expected at least 2 findings, got {}",
            findings.len()
        );
        assert!(findings.iter().any(|f| f.secret_type.contains("AWS")));
        assert!(findings.iter().any(|f| f.secret_type.contains("GitHub")));
    }

    #[test]
    fn test_no_false_positives_on_empty_string() {
        let scanner = SecretScanner::new();
        let findings = scanner.scan("", "empty.js");
        assert!(findings.is_empty());
    }

    #[test]
    fn test_no_false_positives_on_plain_text() {
        let scanner = SecretScanner::new();
        // Plain English prose should not trigger secret patterns
        let text = "The quick brown fox jumps over the lazy dog.";
        let findings = scanner.scan(text, "text.txt");
        assert!(findings.is_empty(), "Should not find secrets in plain text");
    }

    #[test]
    fn test_scan_with_entropy_returns_both() {
        let scanner = SecretScanner::new();
        // AWS key is both a regex match AND a high-entropy string
        let code = "const key = 'AKIA1234567890ABCDEF';";
        let (pattern_findings, _entropy_findings) = scanner.scan_with_entropy(code, "test.js");
        assert!(!pattern_findings.is_empty());
    }

    #[test]
    fn test_redact_sensitive_short_value() {
        // Values shorter than 2*KEEP should be fully redacted
        let result = SecretScanner::redact_sensitive("short", "AWS Secret");
        assert_eq!(result, "***REDACTED***");
    }

    #[test]
    fn test_severity_from_string() {
        assert_eq!(Severity::from("critical"), Severity::Critical);
        assert_eq!(Severity::from("HIGH"), Severity::High);
        assert_eq!(Severity::from("medium"), Severity::Medium);
        assert_eq!(Severity::from("low"), Severity::Low);
        // Unknown falls back to Medium
        assert_eq!(Severity::from("unknown_level"), Severity::Medium);
    }

    #[test]
    fn test_location_recorded_in_finding() {
        let scanner = SecretScanner::new();
        let location = "https://example.com/app.js";
        let code = "const key = 'AKIA1234567890ABCDEF';";
        let findings = scanner.scan(code, location);
        assert!(!findings.is_empty());
        assert_eq!(findings[0].location, location);
    }

    #[test]
    fn test_line_and_column_numbers() {
        let scanner = SecretScanner::new();
        let code = "line1\nconst key = 'AKIA1234567890ABCDEF';";
        let findings = scanner.scan(code, "test.js");
        assert!(!findings.is_empty());
        // Secret is on line 2
        assert_eq!(findings[0].line, 2);
    }
}
