//! Secret scanner implementation

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
}

impl SecretScanner {
    /// Create a new secret scanner
    pub fn new() -> Self {
        let patterns = patterns::compile_patterns().expect("Failed to compile secret patterns");

        SecretScanner { patterns }
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
        let lines: Vec<&str> = text.lines().collect();

        for (line_idx, line) in lines.iter().enumerate() {
            for (name, pattern, severity, description) in &self.patterns {
                if let Some(captures) = pattern.captures(line) {
                    let matched = captures.get(0).unwrap();
                    let matched_text = matched.as_str();

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

    /// Redact sensitive information from matched text
    fn redact_sensitive(text: &str, pattern_name: &str) -> String {
        // For high-sensitivity patterns, redact most of the value
        match pattern_name {
            name if name.contains("Secret")
                || name.contains("Private")
                || name.contains("Password") =>
            {
                let chars: Vec<char> = text.chars().collect();
                if chars.len() > 8 {
                    format!(
                        "{}...{}",
                        chars.iter().take(4).collect::<String>(),
                        chars.iter().skip(chars.len() - 4).collect::<String>()
                    )
                } else {
                    "***REDACTED***".to_string()
                }
            }
            _ => {
                // For other patterns, show a bit more
                let chars: Vec<char> = text.chars().collect();
                if chars.len() > 12 {
                    format!(
                        "{}...{}",
                        chars.iter().take(6).collect::<String>(),
                        chars.iter().skip(chars.len() - 6).collect::<String>()
                    )
                } else {
                    text.to_string()
                }
            }
        }
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
            const email = 'user@example.com';
        "#;
        let all_findings = scanner.scan(code, "test.js");
        let high_findings = scanner.scan_high_severity(code, "test.js");

        // Should have both findings in all_findings
        assert!(all_findings.len() >= 2);
        // high_findings should only have AWS key (critical/high)
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
}
