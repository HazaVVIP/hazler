use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Authentication method to use for HTTP requests
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthMethod {
    /// No authentication
    None,
    /// HTTP Basic Authentication (username:password)
    Basic { username: String, password: String },
    /// Bearer token authentication (Authorization: Bearer <token>)
    Bearer { token: String },
    /// Cookie-based authentication
    Cookie { cookies: HashMap<String, String> },
    /// Custom header authentication
    Header { name: String, value: String },
    /// API Key authentication (can be in query, header, or cookie)
    ApiKey {
        key: String,
        location: ApiKeyLocation,
        name: String,
    },
    /// OAuth 2.0 authentication
    OAuth2 {
        access_token: String,
        token_type: Option<String>,
        refresh_token: Option<String>,
        expires_in: Option<u64>,
    },
}

/// Location where API key should be placed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApiKeyLocation {
    /// API key in query parameter
    Query,
    /// API key in request header
    Header,
    /// API key in cookie
    Cookie,
}

/// Configuration for form-based authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormAuth {
    /// URL of the login page/endpoint
    pub login_url: String,
    /// Username field name in the form
    pub username_field: String,
    /// Password field name in the form
    pub password_field: String,
    /// Username value
    pub username: String,
    /// Password value
    pub password: String,
    /// Additional form fields (e.g., CSRF tokens, hidden fields)
    pub extra_fields: HashMap<String, String>,
    /// Whether to follow redirects after login
    pub follow_redirects: bool,
}

/// Session management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Enable session management (cookie persistence)
    pub enabled: bool,
    /// Session refresh interval in seconds (0 = no refresh)
    pub refresh_interval: u64,
    /// Token refresh endpoint URL (for OAuth2 or custom token refresh)
    pub refresh_url: Option<String>,
    /// Refresh token or credentials
    pub refresh_credentials: Option<AuthMethod>,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            refresh_interval: 0,
            refresh_url: None,
            refresh_credentials: None,
        }
    }
}

impl AuthMethod {
    /// Check if authentication requires session management
    pub fn requires_session(&self) -> bool {
        matches!(
            self,
            AuthMethod::Cookie { .. }
                | AuthMethod::OAuth2 { .. }
                | AuthMethod::Bearer { .. }
        )
    }

    /// Check if this is an OAuth2 method that might need token refresh
    pub fn supports_refresh(&self) -> bool {
        matches!(self, AuthMethod::OAuth2 { refresh_token, .. } if refresh_token.is_some())
    }

    /// Get the authentication value for logging (sanitized)
    pub fn sanitized_display(&self) -> String {
        match self {
            AuthMethod::None => "None".to_string(),
            AuthMethod::Basic { username, .. } => format!("Basic (user: {})", username),
            AuthMethod::Bearer { .. } => "Bearer (token: ***)".to_string(),
            AuthMethod::Cookie { cookies } => {
                format!("Cookie ({} cookies)", cookies.len())
            }
            AuthMethod::Header { name, .. } => format!("Header ({})", name),
            AuthMethod::ApiKey { location, name, .. } => {
                format!("ApiKey ({:?} in {})", location, name)
            }
            AuthMethod::OAuth2 { token_type, .. } => {
                format!(
                    "OAuth2 (type: {})",
                    token_type.as_deref().unwrap_or("Bearer")
                )
            }
        }
    }
}

impl Default for AuthMethod {
    fn default() -> Self {
        AuthMethod::None
    }
}

/// Authentication configuration container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Primary authentication method
    pub method: AuthMethod,
    /// Session configuration
    pub session: SessionConfig,
    /// Form-based authentication (if applicable)
    pub form_auth: Option<FormAuth>,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            method: AuthMethod::None,
            session: SessionConfig::default(),
            form_auth: None,
        }
    }
}

impl AuthConfig {
    /// Create a new authentication configuration
    pub fn new(method: AuthMethod) -> Self {
        Self {
            method,
            session: SessionConfig::default(),
            form_auth: None,
        }
    }

    /// Create Basic Auth configuration
    pub fn basic(username: String, password: String) -> Self {
        Self::new(AuthMethod::Basic { username, password })
    }

    /// Create Bearer token configuration
    pub fn bearer(token: String) -> Self {
        Self::new(AuthMethod::Bearer { token })
    }

    /// Create Cookie-based authentication
    pub fn cookie(cookies: HashMap<String, String>) -> Self {
        Self::new(AuthMethod::Cookie { cookies })
    }

    /// Create custom header authentication
    pub fn header(name: String, value: String) -> Self {
        Self::new(AuthMethod::Header { name, value })
    }

    /// Create API key authentication
    pub fn api_key(key: String, location: ApiKeyLocation, name: String) -> Self {
        Self::new(AuthMethod::ApiKey {
            key,
            location,
            name,
        })
    }

    /// Create OAuth2 authentication
    pub fn oauth2(access_token: String, refresh_token: Option<String>) -> Self {
        Self::new(AuthMethod::OAuth2 {
            access_token,
            token_type: Some("Bearer".to_string()),
            refresh_token,
            expires_in: None,
        })
    }

    /// Set session configuration
    pub fn with_session(mut self, session: SessionConfig) -> Self {
        self.session = session;
        self
    }

    /// Set form authentication
    pub fn with_form_auth(mut self, form_auth: FormAuth) -> Self {
        self.form_auth = Some(form_auth);
        self
    }

    /// Load authentication config from JSON
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Save authentication config to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_auth() {
        let auth = AuthConfig::basic("user".to_string(), "pass".to_string());
        assert!(matches!(
            auth.method,
            AuthMethod::Basic {
                username,
                password
            } if username == "user" && password == "pass"
        ));
    }

    #[test]
    fn test_bearer_auth() {
        let auth = AuthConfig::bearer("token123".to_string());
        assert!(matches!(
            auth.method,
            AuthMethod::Bearer { token } if token == "token123"
        ));
    }

    #[test]
    fn test_cookie_auth() {
        let mut cookies = HashMap::new();
        cookies.insert("session".to_string(), "abc123".to_string());
        let auth = AuthConfig::cookie(cookies.clone());
        assert!(matches!(
            auth.method,
            AuthMethod::Cookie { cookies: c } if c == cookies
        ));
    }

    #[test]
    fn test_header_auth() {
        let auth = AuthConfig::header("X-API-Key".to_string(), "secret".to_string());
        assert!(matches!(
            auth.method,
            AuthMethod::Header { name, value } 
            if name == "X-API-Key" && value == "secret"
        ));
    }

    #[test]
    fn test_api_key_auth() {
        let auth = AuthConfig::api_key(
            "key123".to_string(),
            ApiKeyLocation::Header,
            "X-API-Key".to_string(),
        );
        assert!(matches!(
            auth.method,
            AuthMethod::ApiKey {
                key,
                location: ApiKeyLocation::Header,
                name
            } if key == "key123" && name == "X-API-Key"
        ));
    }

    #[test]
    fn test_oauth2_auth() {
        let auth = AuthConfig::oauth2("access123".to_string(), Some("refresh456".to_string()));
        assert!(matches!(
            auth.method,
            AuthMethod::OAuth2 {
                access_token,
                refresh_token: Some(rt),
                ..
            } if access_token == "access123" && rt == "refresh456"
        ));
    }

    #[test]
    fn test_requires_session() {
        let cookie_auth = AuthMethod::Cookie {
            cookies: HashMap::new(),
        };
        assert!(cookie_auth.requires_session());

        let basic_auth = AuthMethod::Basic {
            username: "user".to_string(),
            password: "pass".to_string(),
        };
        assert!(!basic_auth.requires_session());
    }

    #[test]
    fn test_supports_refresh() {
        let oauth_with_refresh = AuthMethod::OAuth2 {
            access_token: "token".to_string(),
            token_type: Some("Bearer".to_string()),
            refresh_token: Some("refresh".to_string()),
            expires_in: None,
        };
        assert!(oauth_with_refresh.supports_refresh());

        let oauth_no_refresh = AuthMethod::OAuth2 {
            access_token: "token".to_string(),
            token_type: Some("Bearer".to_string()),
            refresh_token: None,
            expires_in: None,
        };
        assert!(!oauth_no_refresh.supports_refresh());
    }

    #[test]
    fn test_sanitized_display() {
        let basic = AuthMethod::Basic {
            username: "user".to_string(),
            password: "secret".to_string(),
        };
        let display = basic.sanitized_display();
        assert!(display.contains("user"));
        assert!(!display.contains("secret"));

        let bearer = AuthMethod::Bearer {
            token: "supersecret".to_string(),
        };
        let display = bearer.sanitized_display();
        assert!(!display.contains("supersecret"));
        assert!(display.contains("***"));
    }

    #[test]
    fn test_json_serialization() {
        let auth = AuthConfig::bearer("token123".to_string());
        let json = auth.to_json().unwrap();
        let deserialized = AuthConfig::from_json(&json).unwrap();

        assert!(matches!(
            deserialized.method,
            AuthMethod::Bearer { token } if token == "token123"
        ));
    }
}
