//! # Hazler Secrets Scanner
//!
//! Secret and sensitive data detection module for Hazler.
//!
//! This module provides functionality to scan web content for:
//! - API keys and tokens (AWS, GitHub, Stripe, Google, etc.)
//! - Credentials and passwords
//! - JWT tokens
//! - Private keys
//! - Internal information leakage
//!
//! ## Example
//!
//! ```
//! use hazler_secrets::{SecretScanner, Severity};
//!
//! let scanner = SecretScanner::new();
//! let findings = scanner.scan("const API_KEY = 'AKIA1234567890ABCDEF';", "test.js");
//!
//! for finding in findings {
//!     println!("{:?}: {} at line {}", finding.severity, finding.secret_type, finding.line);
//! }
//! ```

pub mod context;
pub mod entropy;
pub mod error;
pub mod patterns;
pub mod scanner;

pub use entropy::{calculate_entropy, EntropyFinding, EntropyScanner};
pub use error::SecretError;
pub use scanner::{Finding, SecretScanner, Severity};
