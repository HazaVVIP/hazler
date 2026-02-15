use serde::{Deserialize, Serialize};
use url::Url;

/// Configuration for the headless browser
#[derive(Debug, Clone)]
pub struct BrowserConfig {
    /// Whether to run in headless mode (default: true)
    pub headless: bool,

    /// Request timeout in seconds (default: 30)
    pub timeout_secs: u64,

    /// Window width (default: 1920)
    pub window_width: u32,

    /// Window height (default: 1080)
    pub window_height: u32,

    /// Whether to enable request interception
    pub intercept_requests: bool,

    /// Path to save screenshots (optional)
    pub screenshot_path: Option<String>,

    /// User agent string (optional)
    pub user_agent: Option<String>,

    /// Whether to disable images for faster loading
    pub disable_images: bool,

    /// Whether to disable JavaScript (default: false)
    pub disable_javascript: bool,
}

impl Default for BrowserConfig {
    fn default() -> Self {
        Self {
            headless: true,
            timeout_secs: 30,
            window_width: 1920,
            window_height: 1080,
            intercept_requests: false,
            screenshot_path: None,
            user_agent: None,
            disable_images: false,
            disable_javascript: false,
        }
    }
}

/// Result of a browser page load
#[derive(Debug, Clone)]
pub struct PageLoadResult {
    /// The final URL (after redirects)
    pub url: Url,

    /// HTTP status code
    pub status_code: u16,

    /// Extracted links from the page
    pub links: Vec<String>,

    /// Page title
    pub title: Option<String>,

    /// Screenshot data (if enabled)
    pub screenshot_data: Option<Vec<u8>>,

    /// Cookies from the page
    pub cookies: Vec<Cookie>,

    /// All intercepted network requests (including API calls)
    pub network_requests: Vec<NetworkRequest>,
}

/// Represents a captured network request from browser
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRequest {
    /// Request URL
    pub url: String,

    /// HTTP method
    pub method: String,

    /// Request headers
    pub headers: std::collections::HashMap<String, String>,

    /// Request body/payload (if any)
    pub post_data: Option<String>,

    /// Resource type (Document, Stylesheet, Script, XHR, Fetch, etc.)
    pub resource_type: String,

    /// Request ID
    pub request_id: String,

    /// Timestamp
    pub timestamp: f64,
}

/// Cookie representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cookie {
    pub name: String,
    pub value: String,
    pub domain: Option<String>,
    pub path: Option<String>,
    pub expires: Option<i64>,
    pub http_only: bool,
    pub secure: bool,
    pub same_site: Option<String>,
}
