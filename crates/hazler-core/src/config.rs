use serde::{Deserialize, Serialize};

/// Configuration for the crawler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Maximum depth to crawl
    pub max_depth: usize,
    /// Maximum number of concurrent requests
    pub concurrency: usize,
    /// Maximum number of pages to crawl (0 = unlimited)
    pub max_pages: usize,
    /// User agent string
    pub user_agent: String,
    /// Request timeout in seconds
    pub timeout_secs: u64,
    /// Respect robots.txt
    pub respect_robots: bool,
    /// Follow redirects
    pub follow_redirects: bool,
    /// Maximum redirects to follow
    pub max_redirects: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            max_depth: 3,
            concurrency: 10,
            max_pages: 0,
            user_agent: "Hazler/0.1.0".to_string(),
            timeout_secs: 10,
            respect_robots: true,
            follow_redirects: true,
            max_redirects: 5,
        }
    }
}

impl Config {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum crawl depth
    pub fn max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    /// Set concurrency level
    pub fn concurrency(mut self, concurrency: usize) -> Self {
        self.concurrency = concurrency;
        self
    }

    /// Set maximum pages to crawl
    pub fn max_pages(mut self, max_pages: usize) -> Self {
        self.max_pages = max_pages;
        self
    }

    /// Set user agent
    pub fn user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = user_agent;
        self
    }

    /// Set request timeout
    pub fn timeout_secs(mut self, timeout: u64) -> Self {
        self.timeout_secs = timeout;
        self
    }
}
