use crate::error::{BrowserError, Result};
use crate::types::{BrowserConfig, Cookie, PageLoadResult};
use chromiumoxide::browser::{Browser as ChromeBrowser, BrowserConfig as ChromeConfig};
use chromiumoxide::page::Page;
use futures::StreamExt;
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
        
        // Set headless mode
        if config.headless {
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

        Ok(Self {
            chrome,
            config,
        })
    }

    /// Load a page and capture all network activity
    pub async fn load_page(&self, url: &Url) -> Result<PageLoadResult> {
        info!("Loading page with headless browser: {}", url);

        // Create a new page
        let page = self
            .chrome
            .new_page(url.as_str())
            .await
            .map_err(|e| BrowserError::PageCreationError(format!("Failed to create page: {}", e)))?;

        // Wait for page to load with timeout
        let timeout = std::time::Duration::from_secs(self.config.timeout_secs);
        
        match tokio::time::timeout(timeout, page.wait_for_navigation()).await {
            Ok(Ok(_)) => {
                debug!("Page navigation completed");
            }
            Ok(Err(e)) => {
                warn!("Navigation error (continuing anyway): {}", e);
            }
            Err(_) => {
                warn!("Navigation timeout after {} seconds", self.config.timeout_secs);
            }
        }

        // Give additional time for dynamic content to load
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;

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

        // Get status code (default to 200 if successful)
        let status_code = 200;

        Ok(PageLoadResult {
            url: final_url,
            status_code,
            links,
            title,
            screenshot_data,
            cookies,
        })
    }

    /// Extract all links from the page
    async fn extract_links(&self, page: &Page) -> Result<Vec<String>> {
        let js_code = r#"
            Array.from(document.querySelectorAll('a[href]'))
                .map(a => a.href)
                .filter(href => href && href.length > 0);
        "#;

        let result = page
            .evaluate(js_code)
            .await
            .map_err(|e| BrowserError::JsExecutionError(format!("Failed to extract links: {}", e)))?;

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
            .map_err(|e| BrowserError::ScreenshotError(format!("Failed to take screenshot: {}", e)))?;

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
    async fn test_browser_creation() {
        let config = BrowserConfig::default();
        let result = Browser::new(config).await;
        assert!(result.is_ok());
    }
}
