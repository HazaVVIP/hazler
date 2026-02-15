use crate::auth::{ApiKeyLocation, AuthConfig, AuthMethod, FormAuth};
use crate::error::{Error, Result};
use crate::user_agents::{UserAgentDatabase, generate_chrome_client_hints};
use reqwest::{Client, header};
use std::collections::HashMap;
use std::sync::Arc;
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
    user_agent_db: Arc<UserAgentDatabase>,
    rotate_user_agent: bool,
    add_chrome_hints: bool,
    auth_config: Option<AuthConfig>,
}

impl HttpClient {
    /// Create a new HTTP client with custom configuration
    pub fn new(user_agent: &str, timeout: Duration) -> Result<Self> {
        let client = Client::builder()
            .user_agent(user_agent)
            .timeout(timeout)
            .redirect(reqwest::redirect::Policy::limited(10))
            .cookie_store(true) // Enable cookie jar for session management
            .build()
            .map_err(Error::RequestFailed)?;

        Ok(Self {
            client,
            max_body_size: MAX_BODY_SIZE,
            user_agent_db: Arc::new(UserAgentDatabase::new()),
            rotate_user_agent: false,
            add_chrome_hints: false,
            auth_config: None,
        })
    }
    
    /// Enable User-Agent rotation for WAF evasion
    pub fn with_user_agent_rotation(mut self, enable: bool) -> Self {
        self.rotate_user_agent = enable;
        self
    }
    
    /// Enable Chrome client hints for better fingerprinting
    pub fn with_chrome_hints(mut self, enable: bool) -> Self {
        self.add_chrome_hints = enable;
        self
    }

    /// Set authentication configuration
    pub fn with_auth(mut self, auth_config: AuthConfig) -> Self {
        debug!(
            "Authentication configured: {}",
            auth_config.method.sanitized_display()
        );
        self.auth_config = Some(auth_config);
        self
    }

    /// Create a default HTTP client
    pub fn new_default() -> Result<Self> {
        Self::new("Hazler/0.1.0", Duration::from_secs(10))
    }

    /// Apply authentication to a request
    fn apply_auth(
        &self,
        mut request: reqwest::RequestBuilder,
        url: &Url,
        auth_method: &AuthMethod,
    ) -> Result<reqwest::RequestBuilder> {
        match auth_method {
            AuthMethod::None => {}
            AuthMethod::Basic { username, password } => {
                // HTTP Basic Authentication
                request = request.basic_auth(username, Some(password));
                debug!("Applied Basic authentication for user: {}", username);
            }
            AuthMethod::Bearer { token } => {
                // Bearer token authentication
                request = request.bearer_auth(token);
                debug!("Applied Bearer token authentication");
            }
            AuthMethod::Cookie { cookies } => {
                // Cookie-based authentication
                // Note: reqwest handles cookies automatically with cookie_store enabled
                // We set them here for initial requests
                let cookie_str = cookies
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<_>>()
                    .join("; ");
                request = request.header(header::COOKIE, cookie_str);
                debug!("Applied {} cookies", cookies.len());
            }
            AuthMethod::Header { name, value } => {
                // Custom header authentication
                request = request.header(name, value);
                debug!("Applied custom header: {}", name);
            }
            AuthMethod::ApiKey {
                key,
                location,
                name,
            } => {
                // API Key authentication
                match location {
                    ApiKeyLocation::Header => {
                        request = request.header(name, key);
                        debug!("Applied API key in header: {}", name);
                    }
                    ApiKeyLocation::Query => {
                        // Modify URL to add query parameter
                        let mut url_with_key = url.clone();
                        url_with_key
                            .query_pairs_mut()
                            .append_pair(name, key);
                        request = self.client.get(url_with_key.as_str());
                        debug!("Applied API key in query parameter: {}", name);
                    }
                    ApiKeyLocation::Cookie => {
                        let cookie_str = format!("{}={}", name, key);
                        request = request.header(header::COOKIE, cookie_str);
                        debug!("Applied API key in cookie: {}", name);
                    }
                }
            }
            AuthMethod::OAuth2 {
                access_token,
                token_type,
                ..
            } => {
                // OAuth 2.0 authentication
                let token_type = token_type.as_deref().unwrap_or("Bearer");
                let auth_value = format!("{} {}", token_type, access_token);
                request = request.header(header::AUTHORIZATION, auth_value);
                debug!("Applied OAuth2 authentication ({})", token_type);
            }
        }

        Ok(request)
    }

    /// Perform form-based authentication
    pub async fn form_login(&self, form_auth: &FormAuth) -> Result<()> {
        debug!("Performing form-based login to: {}", form_auth.login_url);

        let login_url = Url::parse(&form_auth.login_url)
            .map_err(|e| Error::InvalidUrl(e.to_string()))?;

        // Build form data
        let mut form_data = HashMap::new();
        form_data.insert(
            form_auth.username_field.clone(),
            form_auth.username.clone(),
        );
        form_data.insert(
            form_auth.password_field.clone(),
            form_auth.password.clone(),
        );

        // Add extra fields (e.g., CSRF tokens)
        for (key, value) in &form_auth.extra_fields {
            form_data.insert(key.clone(), value.clone());
        }

        // Submit the form
        let response = self
            .client
            .post(login_url.as_str())
            .form(&form_data)
            .send()
            .await
            .map_err(Error::RequestFailed)?;

        let status = response.status();
        if status.is_success() || (status.is_redirection() && form_auth.follow_redirects) {
            debug!("Form login successful (status: {})", status);
            // Cookies are automatically stored in the cookie jar
            Ok(())
        } else {
            warn!("Form login failed with status: {}", status);
            Err(Error::AuthenticationFailed(format!(
                "Form login failed with status: {}",
                status
            )))
        }
    }

    /// Fetch a URL and return the response
    pub async fn fetch(&self, url: &Url) -> Result<HttpResponse> {
        debug!("Fetching URL: {}", url);

        // Build request with optional User-Agent rotation and Chrome hints
        let mut request = self.client.get(url.as_str());
        
        // Apply User-Agent rotation if enabled
        if self.rotate_user_agent {
            let user_agent = self.user_agent_db.get_random();
            request = request.header(header::USER_AGENT, user_agent);
            debug!("Using rotated User-Agent: {}", user_agent);
            
            // Add Chrome client hints if enabled and UA is Chrome
            if self.add_chrome_hints {
                if let Some(hints) = generate_chrome_client_hints(user_agent) {
                    for (name, value) in hints {
                        request = request.header(name, value);
                    }
                    debug!("Added Chrome client hints");
                }
            }
        }
        
        // Apply authentication if configured
        if let Some(auth_config) = &self.auth_config {
            request = self.apply_auth(request, url, &auth_config.method)?;
        }
        
        let response = request.send().await.map_err(|e| {
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
