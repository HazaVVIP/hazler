use crate::error::{Error, Result};
use reqwest::Client;
use std::time::Duration;
use tracing::{debug, warn};
use url::Url;

/// Maximum response body size in bytes (10 MB)
const MAX_BODY_SIZE: usize = 10 * 1024 * 1024;

/// HTTP client wrapper for making requests
#[derive(Clone)]
pub struct HttpClient {
    client: Client,
    max_body_size: usize,
}

impl HttpClient {
    /// Create a new HTTP client with custom configuration
    pub fn new(user_agent: &str, timeout: Duration) -> Result<Self> {
        let client = Client::builder()
            .user_agent(user_agent)
            .timeout(timeout)
            .redirect(reqwest::redirect::Policy::limited(10))
            .build()
            .map_err(Error::RequestFailed)?;

        Ok(Self {
            client,
            max_body_size: MAX_BODY_SIZE,
        })
    }

    /// Create a default HTTP client
    pub fn new_default() -> Result<Self> {
        Self::new("Hazler/0.1.0", Duration::from_secs(10))
    }

    /// Fetch a URL and return the response
    pub async fn fetch(&self, url: &Url) -> Result<HttpResponse> {
        debug!("Fetching URL: {}", url);

        let response = self.client.get(url.as_str()).send().await.map_err(|e| {
            warn!("Failed to fetch {}: {}", url, e);
            Error::RequestFailed(e)
        })?;

        let status_code = response.status().as_u16();
        let headers = response
            .headers()
            .iter()
            .map(|(k, v)| {
                let value = v.to_str().unwrap_or("[non-UTF8 header value]");
                (k.to_string(), value.to_string())
            })
            .collect();

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        // Check content length before downloading
        if let Some(content_length) = response.content_length() {
            if content_length as usize > self.max_body_size {
                warn!(
                    "Response from {} exceeds max body size ({} > {} bytes), truncating",
                    url, content_length, self.max_body_size
                );
                return Ok(HttpResponse {
                    url: url.clone(),
                    status_code,
                    headers,
                    content_type,
                    body: format!(
                        "[Response body too large: {} bytes, max {} bytes]",
                        content_length, self.max_body_size
                    ),
                });
            }
        }

        let body = response.text().await.map_err(Error::RequestFailed)?;

        // Double-check after download (in case Content-Length was missing)
        if body.len() > self.max_body_size {
            warn!(
                "Response from {} exceeds max body size ({} > {} bytes), truncating",
                url,
                body.len(),
                self.max_body_size
            );
            // Truncate at byte boundary, ensuring valid UTF-8
            // Find the last valid UTF-8 character boundary at or before max_body_size
            let mut truncate_at = self.max_body_size;
            while truncate_at > 0 && !body.is_char_boundary(truncate_at) {
                truncate_at -= 1;
            }
            let truncated = &body[..truncate_at];
            
            return Ok(HttpResponse {
                url: url.clone(),
                status_code,
                headers,
                content_type,
                body: format!(
                    "{}[... truncated at {} bytes]",
                    truncated, self.max_body_size
                ),
            });
        }

        debug!(
            "Fetched {} - status: {}, size: {} bytes",
            url,
            status_code,
            body.len()
        );

        Ok(HttpResponse {
            url: url.clone(),
            status_code,
            headers,
            content_type,
            body,
        })
    }
}

/// HTTP response data
#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub url: Url,
    pub status_code: u16,
    pub headers: std::collections::HashMap<String, String>,
    pub content_type: Option<String>,
    pub body: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_client() {
        let client = HttpClient::new("TestAgent/1.0", Duration::from_secs(5));
        assert!(client.is_ok());
    }
}
