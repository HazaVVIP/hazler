//! Hazler Browser - Headless browser support for JavaScript-heavy sites
//!
//! This crate provides headless browser capabilities using Chrome/Chromium
//! via the Chrome DevTools Protocol, enabling Hazler to crawl modern
//! single-page applications (SPAs) that require JavaScript execution.

pub mod browser;
pub mod error;
pub mod types;

pub use browser::Browser;
pub use error::{BrowserError, Result};
pub use types::{BrowserConfig, Cookie, PageLoadResult};
