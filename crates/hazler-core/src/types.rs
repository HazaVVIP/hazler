use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use url::Url;

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
}

impl CrawlResult {
    pub fn new() -> Self {
        Self {
            pages: Vec::new(),
            total_pages: 0,
            total_urls: 0,
            errors: Vec::new(),
        }
    }
}

impl Default for CrawlResult {
    fn default() -> Self {
        Self::new()
    }
}
