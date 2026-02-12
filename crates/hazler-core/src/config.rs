use serde::{Deserialize, Serialize};

/// Configuration for the crawler.
///
/// Provides a fluent builder API for configuring crawler behavior.
///
/// # Examples
///
/// ```
/// use hazler_core::Config;
///
/// let config = Config::new()
///     .max_depth(5)
///     .concurrency(20)
///     .max_pages(1000)
///     .user_agent("MyBot/1.0".to_string())
///     .timeout_secs(30);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Maximum depth to crawl from the starting URL
    pub max_depth: usize,
    /// Maximum number of concurrent HTTP requests
    pub concurrency: usize,
    /// Maximum number of pages to crawl (0 = unlimited)
    pub max_pages: usize,
    /// User agent string sent with HTTP requests
    pub user_agent: String,
    /// Request timeout in seconds
    pub timeout_secs: u64,
    /// Whether to respect robots.txt (not yet implemented)
    pub respect_robots: bool,
    /// Whether to follow HTTP redirects
    pub follow_redirects: bool,
    /// Maximum number of redirects to follow
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
    /// Create a new configuration with default values.
    ///
    /// Default values:
    /// - max_depth: 3
    /// - concurrency: 10
    /// - max_pages: 0 (unlimited)
    /// - user_agent: "Hazler/0.1.0"
    /// - timeout_secs: 10
    /// - respect_robots: true
    /// - follow_redirects: true
    /// - max_redirects: 5
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum crawl depth.
    ///
    /// The starting URL is at depth 0. Links found on the starting page
    /// are at depth 1, and so on.
    ///
    /// # Examples
    ///
    /// ```
    /// use hazler_core::Config;
    ///
    /// let config = Config::new().max_depth(5);
    /// assert_eq!(config.max_depth, 5);
    /// ```
    pub fn max_depth(mut self, depth: usize) -> Self {
        self.max_depth = depth;
        self
    }

    /// Set concurrency level (number of simultaneous requests).
    ///
    /// Higher values increase crawling speed but may overwhelm target
    /// servers or your network connection.
    ///
    /// # Examples
    ///
    /// ```
    /// use hazler_core::Config;
    ///
    /// let config = Config::new().concurrency(20);
    /// assert_eq!(config.concurrency, 20);
    /// ```
    pub fn concurrency(mut self, concurrency: usize) -> Self {
        self.concurrency = concurrency;
        self
    }

    /// Set maximum number of pages to crawl.
    ///
    /// Set to 0 for unlimited crawling (use with caution).
    ///
    /// # Examples
    ///
    /// ```
    /// use hazler_core::Config;
    ///
    /// let config = Config::new().max_pages(100);
    /// assert_eq!(config.max_pages, 100);
    /// ```
    pub fn max_pages(mut self, max_pages: usize) -> Self {
        self.max_pages = max_pages;
        self
    }

    /// Set custom User-Agent string.
    ///
    /// # Examples
    ///
    /// ```
    /// use hazler_core::Config;
    ///
    /// let config = Config::new().user_agent("MyBot/1.0".to_string());
    /// assert_eq!(config.user_agent, "MyBot/1.0");
    /// ```
    pub fn user_agent(mut self, user_agent: String) -> Self {
        self.user_agent = user_agent;
        self
    }

    /// Set request timeout in seconds.
    ///
    /// Requests that take longer than this will be aborted.
    ///
    /// # Examples
    ///
    /// ```
    /// use hazler_core::Config;
    ///
    /// let config = Config::new().timeout_secs(30);
    /// assert_eq!(config.timeout_secs, 30);
    /// ```
    pub fn timeout_secs(mut self, timeout: u64) -> Self {
        self.timeout_secs = timeout;
        self
    }
}
