use crate::error::{Error, Result};
use reqwest::Client;
use std::time::Duration;
use tracing::{debug, warn};
use url::Url;

/// HTTP client wrapper for making requests
#[derive(Clone)]
pub struct HttpClient {
    client: Client,
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

        Ok(Self { client })
    }

    /// Create a default HTTP client
    pub fn default() -> Result<Self> {
        Self::new("Hazler/0.1.0", Duration::from_secs(10))
    }

    /// Fetch a URL and return the response
    pub async fn fetch(&self, url: &Url) -> Result<HttpResponse> {
        debug!("Fetching URL: {}", url);

        let response = self
            .client
            .get(url.as_str())
            .send()
            .await
            .map_err(|e| {
                warn!("Failed to fetch {}: {}", url, e);
                Error::RequestFailed(e)
            })?;

        let status_code = response.status().as_u16();
        let headers = response
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or_default().to_string()))
            .collect();

        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_string());

        let body = response.text().await.map_err(Error::RequestFailed)?;

        debug!("Fetched {} - status: {}, size: {} bytes", url, status_code, body.len());

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
