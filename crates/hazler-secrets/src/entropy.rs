//! Shannon entropy-based secret detection
//!
//! This module implements entropy analysis to detect high-entropy strings
//! that are likely to be secrets (API keys, passwords, tokens, etc.) even
//! when they don't match known patterns.
//!
//! ## Algorithm
//!
//! Shannon entropy measures the randomness/unpredictability of a string.
//! Real secrets tend to have high entropy (> 4.5 bits per character) because
//! they are randomly generated, whereas natural-language text and variable
//! names have lower entropy.
//!
//! ## Example
//!
//! ```
//! use hazler_secrets::entropy::{calculate_entropy, EntropyScanner};
//!
//! let entropy = calculate_entropy("Xk7mP9qR8vB2nL4s");
//! assert!(entropy >= 4.0);
//!
//! let scanner = EntropyScanner::new();
//! let findings = scanner.scan("const customApiKey = \"Xk7mP9qR8vB2nL4sZqWe3rTyUiOp\";", "app.js");
//! assert!(!findings.is_empty());
//! ```

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::context::{extract_inline_context, is_likely_placeholder, is_likely_test_context};
use crate::scanner::Severity;

/// Default minimum entropy threshold (bits per character) for flagging a string.
pub const DEFAULT_ENTROPY_THRESHOLD: f64 = 4.5;

/// Default minimum token length to consider for entropy analysis.
pub const MIN_TOKEN_LENGTH: usize = 16;

/// Default maximum token length to consider for entropy analysis.
pub const MAX_TOKEN_LENGTH: usize = 256;

/// An entropy-based finding in scanned text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntropyFinding {
    /// The type label for the finding
    pub secret_type: String,
    /// Severity level
    pub severity: Severity,
    /// Description of why this was flagged
    pub description: String,
    /// Calculated Shannon entropy (bits per character)
    pub entropy: f64,
    /// The raw high-entropy value (may be redacted)
    pub value: String,
    /// Surrounding context from the source line
    pub context: String,
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number of the token start (1-indexed)
    pub column: usize,
    /// Estimated likelihood (0–100) that the value is a real secret
    pub likelihood: u8,
    /// File or URL where found
    pub location: String,
}

/// Calculate the Shannon entropy (in bits per character) of a string.
///
/// Returns 0.0 for empty strings.
pub fn calculate_entropy(s: &str) -> f64 {
    if s.is_empty() {
        return 0.0;
    }

    // Single pass: build frequency map and count characters simultaneously.
    let mut freq: HashMap<char, usize> = HashMap::new();
    let mut len: usize = 0;
    for c in s.chars() {
        *freq.entry(c).or_insert(0) += 1;
        len += 1;
    }

    let len_f = len as f64;
    -freq
        .values()
        .map(|&count| {
            let p = count as f64 / len_f;
            p * p.log2()
        })
        .sum::<f64>()
}

/// Scanner that detects high-entropy strings as potential secrets.
#[derive(Debug, Clone)]
pub struct EntropyScanner {
    /// Minimum entropy threshold; strings at or above this value are flagged.
    pub threshold: f64,
    /// Minimum token length to analyse.
    pub min_length: usize,
    /// Maximum token length to analyse.
    pub max_length: usize,
}

impl EntropyScanner {
    /// Create a new `EntropyScanner` with default settings.
    pub fn new() -> Self {
        Self {
            threshold: DEFAULT_ENTROPY_THRESHOLD,
            min_length: MIN_TOKEN_LENGTH,
            max_length: MAX_TOKEN_LENGTH,
        }
    }

    /// Create an `EntropyScanner` with a custom threshold and token-length range.
    pub fn with_config(threshold: f64, min_length: usize, max_length: usize) -> Self {
        Self {
            threshold,
            min_length,
            max_length,
        }
    }

    /// Scan `text` for high-entropy strings, returning a list of [`EntropyFinding`]s.
    ///
    /// # Arguments
    ///
    /// * `text`     – The source text to scan.
    /// * `location` – The file path or URL being scanned (used in findings).
    pub fn scan(&self, text: &str, location: &str) -> Vec<EntropyFinding> {
        let mut findings = Vec::new();

        for (line_idx, line) in text.lines().enumerate() {
            for token in self.extract_tokens(line) {
                let entropy = calculate_entropy(token.value);
                if entropy < self.threshold {
                    continue;
                }

                // False-positive reduction
                if is_likely_placeholder(token.value) {
                    continue;
                }
                let context =
                    extract_inline_context(line, token.start, token.start + token.value.len(), 50);
                if is_likely_test_context(&context) {
                    continue;
                }

                let likelihood = self.estimate_likelihood(token.value, entropy);
                let severity = if likelihood >= 85 {
                    Severity::High
                } else {
                    Severity::Medium
                };

                findings.push(EntropyFinding {
                    secret_type: "High-Entropy String".to_string(),
                    severity,
                    description: format!(
                        "High-entropy string detected (entropy: {:.2} bits); likely a secret",
                        entropy
                    ),
                    entropy,
                    value: Self::redact(token.value),
                    context,
                    line: line_idx + 1,
                    column: token.start + 1,
                    likelihood,
                    location: location.to_string(),
                });
            }
        }

        findings
    }

    // ── private helpers ──────────────────────────────────────────────────────

    /// Extract candidate tokens from a single line.
    ///
    /// Tokens are contiguous runs of alphanumeric characters plus `+`, `/`,
    /// `=`, `_`, `-` (covers Base64 and common key formats).
    fn extract_tokens<'a>(&self, line: &'a str) -> Vec<Token<'a>> {
        let mut tokens = Vec::new();
        let bytes = line.as_bytes();
        let mut start: Option<usize> = None;

        for (i, &b) in bytes.iter().enumerate() {
            let in_token = b.is_ascii_alphanumeric()
                || matches!(b, b'+' | b'/' | b'=' | b'_' | b'-');

            match (start, in_token) {
                (None, true) => start = Some(i),
                (Some(s), false) => {
                    let slice = &line[s..i];
                    if slice.len() >= self.min_length && slice.len() <= self.max_length {
                        tokens.push(Token {
                            value: slice,
                            start: s,
                        });
                    }
                    start = None;
                }
                _ => {}
            }
        }
        // Handle token that reaches end of line
        if let Some(s) = start {
            let slice = &line[s..];
            if slice.len() >= self.min_length && slice.len() <= self.max_length {
                tokens.push(Token {
                    value: slice,
                    start: s,
                });
            }
        }

        tokens
    }

    /// Estimate likelihood (0–100) that a high-entropy token is a real secret.
    ///
    /// Higher entropy and mix of character classes both increase likelihood.
    fn estimate_likelihood(&self, value: &str, entropy: f64) -> u8 {
        let mut score: f64 = 0.0;

        // Base score from entropy (max 6.0 bits → 100 %)
        let entropy_score = ((entropy - self.threshold) / (6.0 - self.threshold)).clamp(0.0, 1.0);
        score += entropy_score * 60.0;

        // Character-class diversity bonus – single pass over the value.
        let mut has_upper = false;
        let mut has_lower = false;
        let mut has_digit = false;
        let mut has_special = false;
        for c in value.chars() {
            if c.is_ascii_uppercase() {
                has_upper = true;
            } else if c.is_ascii_lowercase() {
                has_lower = true;
            } else if c.is_ascii_digit() {
                has_digit = true;
            } else if matches!(c, '+' | '/' | '=' | '_' | '-') {
                has_special = true;
            }
            // Early exit once all four classes are confirmed.
            if has_upper && has_lower && has_digit && has_special {
                break;
            }
        }

        let class_count = [has_upper, has_lower, has_digit, has_special]
            .iter()
            .filter(|&&x| x)
            .count();
        score += (class_count as f64 / 4.0) * 30.0;

        // Length bonus (longer tokens are more likely to be real secrets)
        let length_score = ((value.len() as f64 - self.min_length as f64)
            / (64.0 - self.min_length as f64))
            .clamp(0.0, 1.0);
        score += length_score * 10.0;

        score.round().clamp(0.0, 100.0) as u8
    }

    /// Redact the middle of a token, keeping the first and last 4 characters.
    ///
    /// Uses a single forward pass with a ring buffer to locate both byte
    /// boundaries simultaneously, avoiding any `Vec<char>` allocation.
    fn redact(value: &str) -> String {
        const KEEP: usize = 4;
        const RING: usize = KEEP + 1; // ring buffer size

        // Ring buffer storing the byte index of the most-recently-seen
        // `RING` character starts.  After the loop, ring[(char_count - k) % RING]
        // holds the byte index of the k-th character from the end (0-indexed from end).
        let mut ring = [0usize; RING];
        let mut char_count = 0usize;
        let mut prefix_end = value.len(); // byte index just after the KEEP-th char

        for (byte_idx, _ch) in value.char_indices() {
            ring[char_count % RING] = byte_idx;
            char_count += 1;
            if char_count == KEEP + 1 {
                // byte_idx is the start of the (KEEP+1)-th char, so the first
                // KEEP chars end here.
                prefix_end = byte_idx;
            }
        }

        if char_count <= KEEP * 2 {
            return "***REDACTED***".to_string();
        }

        // The suffix starts at char index (char_count - KEEP) (0-indexed).
        // That char's byte index is in the ring slot:
        //   (char_count - KEEP) % RING
        let suffix_start = ring[(char_count - KEEP) % RING];
        format!(
            "{}...[REDACTED]...{}",
            &value[..prefix_end],
            &value[suffix_start..]
        )
    }
}

impl Default for EntropyScanner {
    fn default() -> Self {
        Self::new()
    }
}

/// A candidate token extracted from a line of text.
struct Token<'a> {
    value: &'a str,
    start: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── calculate_entropy ──────────────────────────────────────────────────

    #[test]
    fn test_entropy_empty_string() {
        assert_eq!(calculate_entropy(""), 0.0);
    }

    #[test]
    fn test_entropy_single_char() {
        // Single repeated character → entropy = 0
        assert_eq!(calculate_entropy("aaaa"), 0.0);
    }

    #[test]
    fn test_entropy_two_chars() {
        // "ab" repeated → entropy should be 1.0 bit
        let e = calculate_entropy("abab");
        assert!((e - 1.0).abs() < 1e-9);
    }

    #[test]
    fn test_entropy_random_string() {
        // Random-looking base64 string should have high entropy
        let e = calculate_entropy("Xk7mP9qR8vB2nL4sZqWe3rTyUiOp");
        assert!(e > 4.0, "Expected entropy > 4.0, got {}", e);
    }

    #[test]
    fn test_entropy_natural_language() {
        // Natural-language text has lower entropy
        let e = calculate_entropy("the quick brown fox");
        assert!(e < 4.5, "Expected entropy < 4.5, got {}", e);
    }

    #[test]
    fn test_entropy_aws_key() {
        let e = calculate_entropy("AKIA1234567890ABCDEF");
        // AWS key format – should be reasonably high
        assert!(e > 3.5, "Expected entropy > 3.5, got {}", e);
    }

    // ── EntropyScanner ─────────────────────────────────────────────────────

    #[test]
    fn test_scanner_detects_high_entropy() {
        let scanner = EntropyScanner::new();
        // A realistic-looking random API key
        let code = r#"const customApiKey = "Xk7mP9qR8vB2nL4sZqWe3rTyUiOpAs12";"#;
        let findings = scanner.scan(code, "app.js");
        assert!(
            !findings.is_empty(),
            "Expected at least one finding for high-entropy string"
        );
        assert_eq!(findings[0].secret_type, "High-Entropy String");
        assert!(findings[0].entropy >= DEFAULT_ENTROPY_THRESHOLD);
    }

    #[test]
    fn test_scanner_skips_low_entropy() {
        let scanner = EntropyScanner::new();
        let code = "const name = \"hello world this is plain text\";";
        let findings = scanner.scan(code, "app.js");
        assert!(
            findings.is_empty(),
            "Expected no findings for low-entropy text"
        );
    }

    #[test]
    fn test_scanner_skips_placeholder() {
        let scanner = EntropyScanner::new();
        // "changeme" and repeating chars should be skipped
        let code = "const key = \"aaaaaaaaaaaaaaaa\";";
        let findings = scanner.scan(code, "app.js");
        assert!(findings.is_empty(), "Expected no findings for placeholder");
    }

    #[test]
    fn test_scanner_respects_min_length() {
        let scanner = EntropyScanner::new();
        // Short high-entropy string should be below min_length threshold
        let code = "const x = \"Xk7mP9q\";"; // only 7 chars
        let findings = scanner.scan(code, "app.js");
        assert!(
            findings.is_empty(),
            "Expected no findings for short token (below min_length)"
        );
    }

    #[test]
    fn test_scanner_line_and_column() {
        let scanner = EntropyScanner::new();
        let code = "line1\nconst k = \"Xk7mP9qR8vB2nL4sZqWe3rTyUiOpAs12\";\nline3";
        let findings = scanner.scan(code, "test.js");
        assert!(!findings.is_empty());
        assert_eq!(findings[0].line, 2, "Finding should be on line 2");
    }

    #[test]
    fn test_scanner_severity_high_for_very_high_entropy() {
        let scanner = EntropyScanner::new();
        // Base64-like string with all character classes → very high entropy
        let code =
            r#"const token = "aBcDeFgHiJkLmNoPqRsTuVwXyZ0123456789+/abcDEFG";"#;
        let findings = scanner.scan(code, "test.js");
        assert!(!findings.is_empty());
        // High-entropy findings with diverse character classes should be High
        assert!(matches!(
            findings[0].severity,
            Severity::High | Severity::Medium
        ));
    }

    #[test]
    fn test_redact() {
        let redacted = EntropyScanner::redact("Xk7mP9qR8vB2nL4s");
        assert!(redacted.starts_with("Xk7m"));
        assert!(redacted.contains("[REDACTED]"));
    }

    #[test]
    fn test_custom_threshold() {
        // Lower the threshold so that medium-entropy strings are caught
        let scanner = EntropyScanner::with_config(3.5, 8, 256);
        let code = r#"const key = "AbCdEfGhIjKlMnOp";"#;
        let findings = scanner.scan(code, "test.js");
        assert!(!findings.is_empty(), "Should detect with lower threshold");
    }

    #[test]
    fn test_multiline_scan() {
        let scanner = EntropyScanner::new();
        let code = r#"
const a = "lowentropystring";
const b = "Xk7mP9qR8vB2nL4sZqWe3rTyUiOpAs12";
const c = "anotherlowentropystring";
"#;
        let findings = scanner.scan(code, "test.js");
        // Only line with high entropy token should be flagged
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].line, 3);
    }
}
