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
///     .timeout_secs(30)
///     .aggressive(true);
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
    /// Enable aggressive endpoint discovery mode
    pub aggressive_discovery: bool,
    /// Enable stealth mode for WAF evasion
    pub stealth_mode: bool,
    /// Proxy URL for requests
    pub proxy_url: Option<String>,
    /// Enable secrets and sensitive data scanning
    pub secrets_scanning: bool,
    /// Enable strict domain mode (only exact domain, no subdomains)
    pub strict_domain: bool,
    /// Allow crawling subdomains
    pub allow_subdomains: bool,
    /// Enable headless browser for JavaScript-heavy sites
    pub use_headless_browser: bool,
    /// Path to save screenshots when using headless browser
    pub screenshot_path: Option<String>,
    /// Disable images in headless browser for faster loading
    pub disable_images: bool,
    /// Enable GraphQL introspection queries
    pub graphql_introspect: bool,
    /// Enable source map parsing (enabled by default)
    pub parse_source_maps: bool,
    /// Authentication configuration file path
    pub auth_config_file: Option<String>,
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
            aggressive_discovery: false,
            stealth_mode: true, // Enable stealth mode by default
            proxy_url: None,
            secrets_scanning: true, // Enable secrets scanning by default
            strict_domain: false,
            allow_subdomains: false,
            use_headless_browser: false,
            screenshot_path: None,
            disable_images: false,
            graphql_introspect: false,
            parse_source_maps: true, // Enable source map parsing by default
            auth_config_file: None,
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

    /// Enable or disable aggressive endpoint discovery mode.
    ///
    /// When enabled:
    /// - Applies regex patterns to JavaScript files
    /// - Generates URL variations
    /// - Discovers API endpoints more thoroughly
    ///
    /// Warning: This may generate more requests.
    ///
    /// # Examples
    ///
    /// ```
    /// use hazler_core::Config;
    ///
    /// let config = Config::new().aggressive(true);
    /// assert_eq!(config.aggressive_discovery, true);
    /// ```
    pub fn aggressive(mut self, enabled: bool) -> Self {
        self.aggressive_discovery = enabled;
        self
    }

    /// Enable or disable stealth mode for WAF evasion.
    ///
    /// When enabled:
    /// - Randomizes request patterns
    /// - Implements adaptive rate limiting
    /// - Maintains session state
    /// - Uses realistic browser headers
    ///
    /// # Examples
    ///
    /// ```
    /// use hazler_core::Config;
    ///
    /// let config = Config::new().stealth(true);
    /// assert_eq!(config.stealth_mode, true);
    /// ```
    pub fn stealth(mut self, enabled: bool) -> Self {
        self.stealth_mode = enabled;
        self
    }

    /// Set proxy URL for requests.
    ///
    /// Supports HTTP, HTTPS, and SOCKS5 proxies.
    ///
    /// # Examples
    ///
    /// ```
    /// use hazler_core::Config;
    ///
    /// let config = Config::new().proxy("socks5://localhost:1080".to_string());
    /// ```
    pub fn proxy(mut self, proxy_url: String) -> Self {
        self.proxy_url = Some(proxy_url);
        self
    }

    /// Enable or disable secrets and sensitive data scanning.
    ///
    /// When enabled, scans responses for:
    /// - API keys and tokens
    /// - Credentials and passwords
    /// - Internal information leakage
    ///
    /// # Examples
    ///
    /// ```
    /// use hazler_core::Config;
    ///
    /// let config = Config::new().secrets_scanning(true);
    /// assert_eq!(config.secrets_scanning, true);
    /// ```
    pub fn secrets_scanning(mut self, enabled: bool) -> Self {
        self.secrets_scanning = enabled;
        self
    }

    /// Enable strict domain mode - only crawl the exact domain (no subdomains).
    ///
    /// When enabled, the crawler will only visit URLs from the exact domain
    /// specified in the starting URL. Subdomains will be excluded.
    ///
    /// # Examples
    ///
    /// ```
    /// use hazler_core::Config;
    ///
    /// let config = Config::new().strict_domain(true);
    /// assert_eq!(config.strict_domain, true);
    /// ```
    pub fn strict_domain(mut self, enabled: bool) -> Self {
        self.strict_domain = enabled;
        self
    }

    /// Allow crawling subdomains of the target domain.
    ///
    /// When enabled, the crawler will visit subdomains of the starting domain.
    /// For example, if the starting URL is example.com, the crawler will also
    /// visit sub.example.com, api.example.com, etc.
    ///
    /// Note: This option is ignored if strict_domain is enabled.
    ///
    /// # Examples
    ///
    /// ```
    /// use hazler_core::Config;
    ///
    /// let config = Config::new().allow_subdomains(true);
    /// assert_eq!(config.allow_subdomains, true);
    /// ```
    pub fn allow_subdomains(mut self, enabled: bool) -> Self {
        self.allow_subdomains = enabled;
        self
    }

    /// Enable headless browser for crawling JavaScript-heavy sites.
    ///
    /// When enabled, uses Chrome/Chromium via CDP to render pages with JavaScript.
    /// This allows crawling modern SPAs (React, Vue, Angular, etc.) that require
    /// JavaScript execution to display content.
    ///
    /// Note: Requires the "browser" feature to be enabled.
    ///
    /// # Examples
    ///
    /// ```
    /// use hazler_core::Config;
    ///
    /// let config = Config::new().headless_browser(true);
    /// assert_eq!(config.use_headless_browser, true);
    /// ```
    pub fn headless_browser(mut self, enabled: bool) -> Self {
        self.use_headless_browser = enabled;
        self
    }

    /// Set the path to save screenshots when using headless browser.
    ///
    /// Screenshots are saved as PNG files with the URL hash as filename.
    ///
    /// # Examples
    ///
    /// ```
    /// use hazler_core::Config;
    ///
    /// let config = Config::new()
    ///     .headless_browser(true)
    ///     .screenshot_path("screenshots/".to_string());
    /// ```
    pub fn screenshot_path(mut self, path: String) -> Self {
        self.screenshot_path = Some(path);
        self
    }

    /// Disable images in headless browser for faster loading.
    ///
    /// When enabled, the browser will not load images, which can significantly
    /// speed up page loading times.
    ///
    /// # Examples
    ///
    /// ```
    /// use hazler_core::Config;
    ///
    /// let config = Config::new()
    ///     .headless_browser(true)
    ///     .disable_images(true);
    /// ```
    pub fn disable_images(mut self, enabled: bool) -> Self {
        self.disable_images = enabled;
        self
    }

    /// Enable GraphQL introspection queries.
    ///
    /// When enabled, automatically runs introspection queries on detected
    /// GraphQL endpoints to extract schema information including types,
    /// queries, mutations, and subscriptions.
    ///
    /// # Examples
    ///
    /// ```
    /// use hazler_core::Config;
    ///
    /// let config = Config::new().graphql_introspect(true);
    /// assert_eq!(config.graphql_introspect, true);
    /// ```
    pub fn graphql_introspect(mut self, enabled: bool) -> Self {
        self.graphql_introspect = enabled;
        self
    }

    /// Enable or disable source map parsing.
    ///
    /// When enabled (default), automatically detects and parses source maps
    /// to reveal original source code structure, potentially exposing
    /// admin panels, API endpoints, and sensitive paths.
    ///
    /// # Examples
    ///
    /// ```
    /// use hazler_core::Config;
    ///
    /// let config = Config::new().parse_source_maps(false);
    /// assert_eq!(config.parse_source_maps, false);
    /// ```
    pub fn parse_source_maps(mut self, enabled: bool) -> Self {
        self.parse_source_maps = enabled;
        self
    }

    /// Set authentication configuration file path
    ///
    /// Loads authentication credentials from a JSON file.
    ///
    /// # Examples
    ///
    /// ```
    /// use hazler_core::Config;
    ///
    /// let config = Config::new().auth_config_file("auth.json".to_string());
    /// ```
    pub fn auth_config_file(mut self, path: String) -> Self {
        self.auth_config_file = Some(path);
        self
    }
}
