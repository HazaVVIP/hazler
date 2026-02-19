use crate::error::{BrowserError, Result};
use crate::types::{BrowserConfig, Cookie, NetworkRequest, PageLoadResult};
use chromiumoxide::browser::{Browser as ChromeBrowser, BrowserConfig as ChromeConfig};
use chromiumoxide::cdp::browser_protocol::network::EventRequestWillBeSent;
use chromiumoxide::page::Page;
use futures::StreamExt;
use serde_json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};
use url::Url;

/// Headless browser client for crawling JavaScript-heavy sites
pub struct Browser {
    chrome: ChromeBrowser,
    config: BrowserConfig,
}

impl Browser {
    /// Create a new browser instance
    pub async fn new(config: BrowserConfig) -> Result<Self> {
        info!("Launching headless browser...");

        let mut chrome_config = ChromeConfig::builder();

        // Set headless mode (default is headless, with_head() makes it non-headless)
        if !config.headless {
            chrome_config = chrome_config.with_head();
        }

        // Set window size
        chrome_config = chrome_config.window_size(config.window_width, config.window_height);

        // Set user agent if provided
        if let Some(ref user_agent) = config.user_agent {
            chrome_config = chrome_config.arg(format!("--user-agent={}", user_agent));
        }

        // Additional Chrome flags for stealth and performance
        chrome_config = chrome_config
            .arg("--disable-blink-features=AutomationControlled")
            .arg("--disable-dev-shm-usage")
            .arg("--no-sandbox")
            .arg("--disable-setuid-sandbox")
            .arg("--disable-gpu");

        // Disable images if configured
        if config.disable_images {
            chrome_config = chrome_config.arg("--blink-settings=imagesEnabled=false");
        }

        let chrome_config = chrome_config.build().map_err(|e| {
            BrowserError::LaunchError(format!("Failed to build browser config: {}", e))
        })?;

        let (chrome, mut handler) = ChromeBrowser::launch(chrome_config)
            .await
            .map_err(|e| BrowserError::LaunchError(format!("Failed to launch browser: {}", e)))?;

        // Spawn a task to handle browser events
        tokio::spawn(async move {
            while let Some(event) = handler.next().await {
                if let Err(e) = event {
                    error!("Browser handler error: {}", e);
                }
            }
        });

        info!("Browser launched successfully");

        Ok(Self { chrome, config })
    }

    /// Load a page and capture all network activity
    pub async fn load_page(&self, url: &Url) -> Result<PageLoadResult> {
        info!("Loading page with headless browser: {}", url);

        // Create a blank page first (to avoid navigation issues in new_page)
        let page = self.chrome.new_page("about:blank").await.map_err(|e| {
            BrowserError::PageCreationError(format!("Failed to create page: {}", e))
        })?;

        // Enable network domain to capture network events
        page.execute(chromiumoxide::cdp::browser_protocol::network::EnableParams::default())
            .await
            .map_err(|e| {
                BrowserError::InterceptionError(format!("Failed to enable network: {}", e))
            })?;

        // Storage for captured network requests
        let network_requests = Arc::new(Mutex::new(Vec::new()));
        let network_requests_clone = network_requests.clone();

        // Set up network request listener
        let mut request_events = page
            .event_listener::<EventRequestWillBeSent>()
            .await
            .map_err(|e| {
                BrowserError::InterceptionError(format!("Failed to create event listener: {}", e))
            })?;

        // Spawn task to capture network requests
        tokio::spawn(async move {
            while let Some(event) = request_events.next().await {
                let request = &event.request;
                let resource_type = event
                    .r#type
                    .as_ref()
                    .map(|t| format!("{:?}", t))
                    .unwrap_or_else(|| "Unknown".to_string());

                // Convert headers to HashMap
                // Serialize Headers to JSON and then parse as HashMap
                let mut headers: HashMap<String, String> = HashMap::new();
                if let Ok(headers_json) = serde_json::to_value(&request.headers) {
                    if let Ok(headers_map) =
                        serde_json::from_value::<HashMap<String, serde_json::Value>>(headers_json)
                    {
                        for (k, v) in headers_map {
                            let value_str = match v {
                                serde_json::Value::String(s) => s,
                                _ => v.to_string(),
                            };
                            headers.insert(k, value_str);
                        }
                    }
                }

                // Get post data if available
                let post_data = if request.has_post_data.unwrap_or(false) {
                    request
                        .post_data_entries
                        .as_ref()
                        .and_then(|entries| entries.first())
                        .and_then(|entry| entry.bytes.as_ref())
                        .map(|b| String::from_utf8_lossy(b.as_ref()).to_string())
                } else {
                    None
                };

                let network_request = NetworkRequest {
                    url: request.url.clone(),
                    method: request.method.clone(),
                    headers,
                    post_data,
                    resource_type: resource_type.clone(),
                    request_id: format!("{:?}", event.request_id),
                    timestamp: *event.timestamp.inner(),
                };

                // Log interesting API requests
                if network_request.url.contains("/api/")
                    || network_request.url.contains("/graphql")
                    || network_request.url.contains("/v1/")
                    || network_request.url.contains("/v2/")
                    || resource_type.contains("XHR")
                    || resource_type.contains("Fetch")
                {
                    info!(
                        "🔍 API Request detected: {} {}",
                        network_request.method, network_request.url
                    );

                    // Log authentication headers if present
                    if let Some(auth) = network_request.headers.get("authorization") {
                        info!(
                            "  🔑 Authorization header found: {}",
                            if auth.len() > 20 {
                                format!("{}...", &auth[..20])
                            } else {
                                auth.clone()
                            }
                        );
                    }

                    // Log payload for POST/PUT/PATCH requests
                    if let Some(ref payload) = network_request.post_data {
                        if payload.len() < 500 {
                            info!("  📦 Payload: {}", payload);
                        } else {
                            info!("  📦 Payload: {} bytes", payload.len());
                        }
                    }
                }

                network_requests_clone.lock().await.push(network_request);
            }
        });

        // Navigate to the target URL with retry logic
        let max_retries = 3;
        let mut retry_count = 0;
        let timeout = std::time::Duration::from_secs(self.config.timeout_secs);
        
        loop {
            retry_count += 1;
            debug!("Navigation attempt {} of {}", retry_count, max_retries);
            
            // Navigate to the URL
            match page.goto(url.as_str()).await {
                Ok(_) => {
                    debug!("Navigation initiated successfully");
                }
                Err(e) => {
                    warn!("Failed to navigate to {}: {}", url, e);
                    if retry_count >= max_retries {
                        return Err(BrowserError::NavigationError(format!(
                            "Failed to navigate after {} attempts: {}",
                            max_retries, e
                        )));
                    }
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    continue;
                }
            }
            
            // Wait for page to load with timeout
            match tokio::time::timeout(timeout, page.wait_for_navigation()).await {
                Ok(Ok(_)) => {
                    debug!("Page navigation completed");
                }
                Ok(Err(e)) => {
                    warn!("Navigation error: {}", e);
                    if retry_count >= max_retries {
                        warn!("Continuing despite navigation error after {} attempts", max_retries);
                    } else {
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                        continue;
                    }
                }
                Err(_) => {
                    warn!(
                        "Navigation timeout after {} seconds",
                        self.config.timeout_secs
                    );
                    if retry_count >= max_retries {
                        warn!("Continuing despite timeout after {} attempts", max_retries);
                    } else {
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                        continue;
                    }
                }
            }
            
            // Check if we landed on an error page
            if let Ok(Some(current_url)) = page.url().await {
                if current_url.starts_with("chrome-error://") {
                    warn!("Landed on chrome error page: {}", current_url);
                    if retry_count >= max_retries {
                        return Err(BrowserError::NavigationError(format!(
                            "Navigation resulted in error page after {} attempts: {}",
                            max_retries, current_url
                        )));
                    }
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    continue;
                }
            }
            
            // Navigation successful
            break;
        }

        // Give additional time for dynamic content to load and network requests to complete
        info!("Waiting for dynamic content and API calls...");
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;

        // Extract links from the page
        let links = self.extract_links(&page).await.unwrap_or_default();

        // Get page title
        let title = self.get_page_title(&page).await.ok();

        // Get cookies
        let cookies = self.get_cookies(&page).await.unwrap_or_default();

        // Take screenshot if enabled
        let screenshot_data = if self.config.screenshot_path.is_some() {
            self.take_screenshot(&page).await.ok()
        } else {
            None
        };

        // Get final URL (after redirects)
        let final_url = page
            .url()
            .await
            .map_err(|e| BrowserError::NavigationError(format!("Failed to get final URL: {}", e)))?
            .ok_or_else(|| BrowserError::NavigationError("No URL found".to_string()))?;

        let final_url = Url::parse(&final_url)
            .map_err(|e| BrowserError::NavigationError(format!("Invalid URL: {}", e)))?;

        // Final validation: ensure we didn't end up on an error page
        if final_url.scheme() == "chrome-error" || final_url.as_str().starts_with("chrome-error://") {
            return Err(BrowserError::NavigationError(format!(
                "Navigation failed - ended up on error page: {}",
                final_url
            )));
        }

        // Get captured network requests
        let captured_requests = network_requests.lock().await.clone();

        info!(
            "Captured {} network requests (including {} API calls)",
            captured_requests.len(),
            captured_requests
                .iter()
                .filter(|r| r.url.contains("/api/")
                    || r.url.contains("/graphql")
                    || r.resource_type.contains("XHR")
                    || r.resource_type.contains("Fetch"))
                .count()
        );

        // Get status code (default to 200 if successful)
        let status_code = 200;

        Ok(PageLoadResult {
            url: final_url,
            status_code,
            links,
            title,
            screenshot_data,
            cookies,
            network_requests: captured_requests,
        })
    }

    /// Extract all links from the page
    async fn extract_links(&self, page: &Page) -> Result<Vec<String>> {
        let js_code = r#"
            Array.from(document.querySelectorAll('a[href]'))
                .map(a => a.href)
                .filter(href => href && href.length > 0);
        "#;

        let result = page.evaluate(js_code).await.map_err(|e| {
            BrowserError::JsExecutionError(format!("Failed to extract links: {}", e))
        })?;

        let links: Vec<String> = result
            .into_value()
            .map_err(|e| BrowserError::JsExecutionError(format!("Failed to parse links: {}", e)))?;

        debug!("Extracted {} links from page", links.len());
        Ok(links)
    }

    /// Get the page title
    async fn get_page_title(&self, page: &Page) -> Result<String> {
        let js_code = "document.title";

        let result = page
            .evaluate(js_code)
            .await
            .map_err(|e| BrowserError::JsExecutionError(format!("Failed to get title: {}", e)))?;

        let title: String = result
            .into_value()
            .map_err(|e| BrowserError::JsExecutionError(format!("Failed to parse title: {}", e)))?;

        Ok(title)
    }

    /// Get all cookies from the page
    async fn get_cookies(&self, page: &Page) -> Result<Vec<Cookie>> {
        let cookies = page
            .get_cookies()
            .await
            .map_err(|e| BrowserError::CookieError(format!("Failed to get cookies: {}", e)))?;

        let result = cookies
            .into_iter()
            .map(|c| {
                let same_site_str = c.same_site.as_ref().map(|s| format!("{:?}", s));
                Cookie {
                    name: c.name,
                    value: c.value,
                    domain: Some(c.domain),
                    path: Some(c.path),
                    expires: Some(c.expires as i64),
                    http_only: c.http_only,
                    secure: c.secure,
                    same_site: same_site_str,
                }
            })
            .collect();

        Ok(result)
    }

    /// Take a screenshot of the page
    async fn take_screenshot(&self, page: &Page) -> Result<Vec<u8>> {
        let screenshot = page
            .screenshot(chromiumoxide::page::ScreenshotParams::default())
            .await
            .map_err(|e| {
                BrowserError::ScreenshotError(format!("Failed to take screenshot: {}", e))
            })?;

        Ok(screenshot)
    }

    /// Close the browser
    pub async fn close(mut self) -> Result<()> {
        info!("Closing browser");
        self.chrome
            .close()
            .await
            .map_err(|e| BrowserError::BrowserError(format!("Failed to close browser: {}", e)))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires Chrome/Chromium to be installed
    async fn test_browser_creation() {
        let config = BrowserConfig::default();
        let result = Browser::new(config).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_browser_config_default() {
        let config = BrowserConfig::default();
        assert!(config.headless);
        assert_eq!(config.timeout_secs, 30);
        assert_eq!(config.window_width, 1920);
        assert_eq!(config.window_height, 1080);
    }

    #[test]
    fn test_network_request_structure() {
        let req = NetworkRequest {
            url: "https://api.example.com/users".to_string(),
            method: "GET".to_string(),
            headers: std::collections::HashMap::new(),
            post_data: None,
            resource_type: "XHR".to_string(),
            request_id: "123".to_string(),
            timestamp: 0.0,
        };

        assert_eq!(req.url, "https://api.example.com/users");
        assert_eq!(req.method, "GET");
        assert_eq!(req.resource_type, "XHR");
    }
}
