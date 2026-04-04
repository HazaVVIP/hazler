use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

/// A verified, valid endpoint emitted in real-time through the output channel.
///
/// Only pages that pass all validity checks (correct HTTP status, non-error
/// response body, not noise-filtered, above minimum body length) are emitted
/// as `ValidEndpoint` values.
#[derive(Debug, Clone)]
pub struct ValidEndpoint {
    /// The URL of the endpoint
    pub url: Url,
    /// HTTP status code
    pub status_code: u16,
    /// Content-Type of the response (empty string when unknown)
    pub content_type: String,
}

/// Severity level for findings
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
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

/// Represents a crawled web page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    /// The URL of the page
    pub url: Url,
    /// HTTP status code
    pub status_code: u16,
    /// Response body (HTML content)
    pub body: String,
    /// Response headers
    pub headers: HashMap<String, String>,
    /// Content type
    pub content_type: Option<String>,
    /// Links found on the page
    pub links: Vec<Url>,
    /// Depth in the crawl tree
    pub depth: usize,
    /// Secret findings (if secrets scanning is enabled)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub secrets: Vec<Finding>,
    /// Whether this page was suppressed by the noise filter (repetitive response pattern)
    #[serde(skip)]
    pub was_noise_filtered: bool,
}

impl Page {
    pub fn new(url: Url, status_code: u16, body: String, depth: usize) -> Self {
        Self {
            url,
            status_code,
            body,
            headers: HashMap::new(),
            content_type: None,
            links: Vec::new(),
            depth,
            secrets: Vec::new(),
            was_noise_filtered: false,
        }
    }
}

/// Result of a crawl operation
#[derive(Debug, Serialize, Deserialize)]
pub struct CrawlResult {
    /// All successfully crawled pages
    pub pages: Vec<Page>,
    /// Total number of pages crawled
    pub total_pages: usize,
    /// Total number of URLs discovered
    pub total_urls: usize,
    /// Errors encountered
    pub errors: Vec<String>,
    /// Total secret findings (if secrets scanning is enabled)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub secret_findings: Option<FindingStats>,
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

impl CrawlResult {
    pub fn new() -> Self {
        Self {
            pages: Vec::new(),
            total_pages: 0,
            total_urls: 0,
            errors: Vec::new(),
            secret_findings: None,
        }
    }
}

impl Default for CrawlResult {
    fn default() -> Self {
        Self::new()
    }
}
