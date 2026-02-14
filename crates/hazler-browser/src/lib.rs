//! Hazler Browser - Headless browser support for JavaScript-heavy sites
//!
//! This crate provides headless browser capabilities using Chrome/Chromium
//! via the Chrome DevTools Protocol, enabling Hazler to crawl modern
//! single-page applications (SPAs) that require JavaScript execution.
//!
//! Key features:
//! - Network request interception using CDP events
//! - Automatic capture of API endpoints, headers, and payloads
//! - Detection of authentication tokens and sensitive data
//! - Perfect for finding IDOR vulnerabilities and API leaks

pub mod browser;
pub mod error;
pub mod types;

pub use browser::Browser;
pub use error::{BrowserError, Result};
pub use types::{BrowserConfig, Cookie, NetworkRequest, PageLoadResult};
