//! Secret detection patterns
//!
//! This module contains regex patterns for detecting various types of secrets
//! and sensitive information in web content.

use once_cell::sync::Lazy;
use regex::Regex;

/// A secret pattern definition
#[derive(Debug, Clone)]
pub struct SecretPattern {
    pub name: &'static str,
    pub pattern: &'static str,
    pub severity: &'static str,
    pub description: &'static str,
}

/// All secret patterns used for detection
pub static SECRET_PATTERNS: Lazy<Vec<SecretPattern>> = Lazy::new(|| {
    vec![
        // AWS Access Keys
        SecretPattern {
            name: "AWS Access Key ID",
            pattern: r"AKIA[0-9A-Z]{16}",
            severity: "critical",
            description: "AWS Access Key ID found",
        },
        SecretPattern {
            name: "AWS Secret Access Key",
            pattern: r"aws[_-]?secret[_-]?access[_-]?key['\x22]?\s*[:=]\s*['\x22]?([A-Za-z0-9/+=]{40})",
            severity: "critical",
            description: "AWS Secret Access Key found",
        },
        // GitHub Tokens
        SecretPattern {
            name: "GitHub Personal Access Token",
            pattern: r"ghp_[A-Za-z0-9]{30,}",
            severity: "critical",
            description: "GitHub Personal Access Token found",
        },
        SecretPattern {
            name: "GitHub OAuth Token",
            pattern: r"gho_[A-Za-z0-9]{30,}",
            severity: "critical",
            description: "GitHub OAuth Access Token found",
        },
        SecretPattern {
            name: "GitHub App Token",
            pattern: r"(ghu|ghs)_[A-Za-z0-9]{30,}",
            severity: "critical",
            description: "GitHub App Token found",
        },
        SecretPattern {
            name: "GitHub Refresh Token",
            pattern: r"ghr_[A-Za-z0-9]{30,}",
            severity: "critical",
            description: "GitHub Refresh Token found",
        },
        // Stripe Keys
        SecretPattern {
            name: "Stripe Live Secret Key",
            pattern: r"sk_live_[0-9a-zA-Z]{24,}",
            severity: "critical",
            description: "Stripe Live Secret Key found",
        },
        SecretPattern {
            name: "Stripe Live Publishable Key",
            pattern: r"pk_live_[0-9a-zA-Z]{24,}",
            severity: "high",
            description: "Stripe Live Publishable Key found",
        },
        SecretPattern {
            name: "Stripe Restricted Key",
            pattern: r"rk_live_[0-9a-zA-Z]{24,}",
            severity: "critical",
            description: "Stripe Restricted Key found",
        },
        // Google API Keys
        SecretPattern {
            name: "Google API Key",
            pattern: r"AIza[0-9A-Za-z\-_]{35}",
            severity: "high",
            description: "Google API Key found",
        },
        SecretPattern {
            name: "Google Cloud Service Account",
            pattern: r#""type":\s*"service_account""#,
            severity: "critical",
            description: "Google Cloud Service Account JSON found",
        },
        // Generic API Keys
        SecretPattern {
            name: "Generic API Key",
            pattern: r"(?i)(api[_-]?key|apikey)['\x22]?\s*[:=]\s*['\x22]?([A-Za-z0-9_-]{20,})",
            severity: "high",
            description: "Generic API key found",
        },
        SecretPattern {
            name: "Generic Secret Key",
            pattern: r"(?i)(secret[_-]?key|secretkey)['\x22]?\s*[:=]\s*['\x22]?([A-Za-z0-9_-]{20,})",
            severity: "high",
            description: "Generic secret key found",
        },
        SecretPattern {
            name: "Generic Access Token",
            pattern: r"(?i)(access[_-]?token|accesstoken)['\x22]?\s*[:=]\s*['\x22]?([A-Za-z0-9_-]{20,})",
            severity: "high",
            description: "Generic access token found",
        },
        // JWT Tokens
        SecretPattern {
            name: "JWT Token",
            pattern: r"eyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}",
            severity: "medium",
            description: "JWT token found",
        },
        // Private Keys
        SecretPattern {
            name: "RSA Private Key",
            pattern: r"-----BEGIN RSA PRIVATE KEY-----",
            severity: "critical",
            description: "RSA private key found",
        },
        SecretPattern {
            name: "SSH Private Key",
            pattern: r"-----BEGIN OPENSSH PRIVATE KEY-----",
            severity: "critical",
            description: "SSH private key found",
        },
        SecretPattern {
            name: "PGP Private Key",
            pattern: r"-----BEGIN PGP PRIVATE KEY BLOCK-----",
            severity: "critical",
            description: "PGP private key found",
        },
        // Database Connection Strings
        SecretPattern {
            name: "Database Connection String",
            pattern: r"(?i)(mysql|postgres|mongodb|redis)://[^\s]+:[^\s]+@[^\s]+",
            severity: "critical",
            description: "Database connection string with credentials found",
        },
        SecretPattern {
            name: "JDBC Connection String",
            pattern: r"jdbc:[^\s]+password=[^\s&;]+",
            severity: "critical",
            description: "JDBC connection string with password found",
        },
        // Passwords
        SecretPattern {
            name: "Password in Code",
            pattern: r"(?i)password['\x22]?\s*[:=]\s*['\x22]([^\x22']{8,})['\x22]",
            severity: "high",
            description: "Password found in code",
        },
        // Slack Tokens
        SecretPattern {
            name: "Slack Token",
            pattern: r"xox[baprs]-[0-9]{10,13}-[0-9]{10,13}-[A-Za-z0-9]{24,}",
            severity: "critical",
            description: "Slack token found",
        },
        SecretPattern {
            name: "Slack Webhook",
            pattern: r"https://hooks\.slack\.com/services/T[A-Z0-9]{8,}/B[A-Z0-9]{8,}/[A-Za-z0-9]{24,}",
            severity: "high",
            description: "Slack webhook URL found",
        },
        // Azure
        SecretPattern {
            name: "Azure Storage Account Key",
            pattern: r"(?i)DefaultEndpointsProtocol=https;AccountName=[^;]+;AccountKey=[A-Za-z0-9+/=]{88}",
            severity: "critical",
            description: "Azure Storage Account connection string found",
        },
        // Twilio
        SecretPattern {
            name: "Twilio API Key",
            pattern: r"SK[0-9a-fA-F]{32}",
            severity: "critical",
            description: "Twilio API Key found",
        },
        // SendGrid
        SecretPattern {
            name: "SendGrid API Key",
            pattern: r"SG\.[A-Za-z0-9_-]{22}\.[A-Za-z0-9_-]{43}",
            severity: "critical",
            description: "SendGrid API Key found",
        },
        // Mailgun
        SecretPattern {
            name: "Mailgun API Key",
            pattern: r"key-[0-9a-zA-Z]{32}",
            severity: "high",
            description: "Mailgun API Key found",
        },
        // MailChimp
        SecretPattern {
            name: "MailChimp API Key",
            pattern: r"[0-9a-f]{32}-us[0-9]{1,2}",
            severity: "high",
            description: "MailChimp API Key found",
        },
        // NPM Tokens
        SecretPattern {
            name: "NPM Access Token",
            pattern: r"npm_[A-Za-z0-9]{36}",
            severity: "high",
            description: "NPM access token found",
        },
        // PyPI Tokens
        SecretPattern {
            name: "PyPI Upload Token",
            pattern: r"pypi-[A-Za-z0-9-_]{32,}",
            severity: "high",
            description: "PyPI upload token found",
        },
        // Internal Information
        SecretPattern {
            name: "Internal IP Address",
            pattern: r"\b(?:10\.|172\.(?:1[6-9]|2[0-9]|3[01])\.|192\.168\.)\d{1,3}\.\d{1,3}\b",
            severity: "medium",
            description: "Internal IP address found",
        },
        // Credentials in Config Files (only when inline credentials are present, not just a file reference)
        SecretPattern {
            name: "Environment Variable with Secret",
            pattern: r"(?:^|['\x22;\s])((?i:SECRET|PASSWORD|API[_-]?KEY|TOKEN|AUTH)[A-Za-z0-9_]*)=['\x22]?([A-Za-z0-9+/=_\-]{16,})['\x22]?",
            severity: "high",
            description: "Potential secret in environment variable assignment found",
        },
    ]
});

/// Compile all regex patterns
pub fn compile_patterns() -> Result<Vec<(String, Regex, String, String)>, regex::Error> {
    SECRET_PATTERNS
        .iter()
        .map(|p| {
            Regex::new(p.pattern).map(|regex| {
                (
                    p.name.to_string(),
                    regex,
                    p.severity.to_string(),
                    p.description.to_string(),
                )
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_patterns() {
        let patterns = compile_patterns();
        assert!(patterns.is_ok());
        let patterns = patterns.unwrap();
        assert!(!patterns.is_empty());
        assert!(patterns.len() > 30); // We have many patterns
    }

    #[test]
    fn test_aws_key_pattern() {
        let pattern = Regex::new(r"AKIA[0-9A-Z]{16}").unwrap();
        assert!(pattern.is_match("AKIA1234567890ABCDEF"));
        assert!(!pattern.is_match("NOTAKIA1234567890ABC"));
    }

    #[test]
    fn test_github_token_pattern() {
        let pattern = Regex::new(r"ghp_[A-Za-z0-9]{30,}").unwrap();
        assert!(pattern.is_match("ghp_1234567890abcdefABCDEF1234567890ab"));
        assert!(!pattern.is_match("ghp_short"));
    }

    #[test]
    fn test_jwt_pattern() {
        let pattern =
            Regex::new(r"eyJ[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}\.[A-Za-z0-9_-]{10,}").unwrap();
        assert!(pattern.is_match("eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIiwibmFtZSI6IkpvaG4gRG9lIiwiaWF0IjoxNTE2MjM5MDIyfQ.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c"));
    }
}
