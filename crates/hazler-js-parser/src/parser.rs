use crate::error::Result;
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashSet;
use url::Url;

/// JavaScript URL patterns for endpoint discovery
static JS_URL_PATTERNS: &[&str] = &[
    // URL in quotes
    r#"["']https?://[^"'\s]+["']"#,
    r#"["'](/[a-zA-Z0-9/_\-\.]+)["']"#,
    // Fetch API calls
    r#"fetch\s*\(\s*["']([^"']+)["']"#,
    r#"fetch\s*\(\s*`([^`]+)`"#,
    // XMLHttpRequest
    r#"\.open\s*\(\s*["'][^"']*["']\s*,\s*["']([^"']+)["']"#,
    // Axios calls
    r#"axios\.(get|post|put|delete|patch)\s*\(\s*["']([^"']+)["']"#,
    // jQuery AJAX
    r#"\$\.ajax\s*\(\s*\{[^}]*url\s*:\s*["']([^"']+)["']"#,
    r#"\$\.(get|post)\s*\(\s*["']([^"']+)["']"#,
    // API endpoint definitions
    r#"(api|endpoint|url|path|route)\s*[:=]\s*["']([^"']+)["']"#,
    // Template literals
    r#"`/api/[^`]+`"#,
    r#"`https?://[^`]+`"#,
    // Relative paths in router configs
    r#"path\s*:\s*["']([^"']+)["']"#,
    r#"route\s*:\s*["']([^"']+)["']"#,
    // GraphQL endpoints
    r#"(graphql|gql)\s*["']([^"']+)["']"#,
    // WebSocket endpoints
    r#"["'](wss?://[^"'\s]+)["']"#,
    // JSON-RPC endpoints
    r#"rpc\s*:\s*["']([^"']+)["']"#,
    // More API patterns
    r#"\.get\s*\(\s*["']([^"']+)["']"#,
    r#"\.post\s*\(\s*["']([^"']+)["']"#,
    r#"\.put\s*\(\s*["']([^"']+)["']"#,
    r#"\.delete\s*\(\s*["']([^"']+)["']"#,
    r#"\.patch\s*\(\s*["']([^"']+)["']"#,
];

/// Template variable pattern for replacement
static TEMPLATE_VAR_RE: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"\$\{[^}]+\}").expect("Failed to compile template variable regex"));

/// JavaScript parser for extracting endpoints from JavaScript code
#[derive(Clone)]
pub struct JavaScriptParser {
    patterns: Vec<Regex>,
}

impl JavaScriptParser {
    /// Create a new JavaScript parser
    pub fn new() -> Result<Self> {
        let patterns = JS_URL_PATTERNS
            .iter()
            .map(|p| Regex::new(p))
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(Self { patterns })
    }

    /// Extract endpoints from JavaScript content
    pub fn extract_endpoints(&self, js_content: &str, base_url: &Url) -> Vec<Url> {
        let mut endpoints = HashSet::new();

        for pattern in &self.patterns {
            for cap in pattern.captures_iter(js_content) {
                // Extract URL from capture groups
                for i in 1..cap.len() {
                    if let Some(url_match) = cap.get(i) {
                        let url_str = url_match.as_str();

                        // Try to resolve as absolute or relative URL
                        if let Ok(url) = self.normalize_and_resolve(url_str, base_url) {
                            endpoints.insert(url);
                        }
                    }
                }
            }
        }

        endpoints.into_iter().collect()
    }

    /// Normalize and resolve a URL string against a base URL
    fn normalize_and_resolve(&self, url_str: &str, base_url: &Url) -> Result<Url> {
        // Remove quotes and backticks
        let cleaned = url_str.trim_matches(|c| c == '"' || c == '\'' || c == '`');

        // Handle template literals with variables
        let cleaned = self.replace_template_vars(cleaned);

        // Try absolute URL first
        if let Ok(url) = Url::parse(&cleaned) {
            return Ok(url);
        }

        // Try relative URL
        Ok(base_url.join(&cleaned)?)
    }

    /// Replace template variables with placeholder values
    fn replace_template_vars(&self, url: &str) -> String {
        let mut result = url.to_string();

        // Replace ${variable} patterns with placeholders
        result = TEMPLATE_VAR_RE.replace_all(&result, "0").to_string();

        // Replace common placeholder patterns with example values
        result = result
            .replace("{id}", "1")
            .replace("{userId}", "1")
            .replace("{user_id}", "1")
            .replace("{uuid}", "00000000-0000-0000-0000-000000000000")
            .replace("{slug}", "example")
            .replace("{name}", "example")
            .replace(":id", "1")
            .replace(":userId", "1")
            .replace(":user_id", "1")
            .replace(":uuid", "00000000-0000-0000-0000-000000000000")
            .replace(":slug", "example")
            .replace(":name", "example");

        result
    }
}

impl Default for JavaScriptParser {
    fn default() -> Self {
        Self::new().expect("Failed to create default JavaScriptParser")
    }
}

/// Parser for .frame files
#[derive(Clone)]
pub struct FrameFileParser {
    js_parser: JavaScriptParser,
}

impl FrameFileParser {
    /// Create a new Frame file parser
    pub fn new() -> Result<Self> {
        Ok(Self {
            js_parser: JavaScriptParser::new()?,
        })
    }

    /// Extract endpoints from .frame file content
    pub fn extract_endpoints(&self, frame_content: &str, base_url: &Url) -> Vec<Url> {
        let mut endpoints = Vec::new();

        // .frame files might contain JSON-like structures
        // Try to parse as JSON first
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(frame_content) {
            endpoints.extend(self.extract_from_json(&json, base_url));
        }

        // Also apply JavaScript patterns
        endpoints.extend(self.js_parser.extract_endpoints(frame_content, base_url));

        // Deduplicate
        let unique: HashSet<_> = endpoints.into_iter().collect();
        unique.into_iter().collect()
    }

    /// Recursively extract URLs from JSON structure
    fn extract_from_json(&self, json: &serde_json::Value, base_url: &Url) -> Vec<Url> {
        let mut endpoints = Vec::new();

        match json {
            serde_json::Value::Object(map) => {
                for (key, value) in map {
                    // Look for keys that suggest URLs
                    if key.contains("url")
                        || key.contains("endpoint")
                        || key.contains("path")
                        || key.contains("route")
                        || key.contains("href")
                        || key.contains("link")
                    {
                        if let Some(url_str) = value.as_str() {
                            if let Ok(url) = base_url.join(url_str) {
                                endpoints.push(url);
                            }
                        }
                    }
                    // Recurse into nested objects
                    endpoints.extend(self.extract_from_json(value, base_url));
                }
            }
            serde_json::Value::Array(arr) => {
                for item in arr {
                    endpoints.extend(self.extract_from_json(item, base_url));
                }
            }
            _ => {}
        }

        endpoints
    }
}

impl Default for FrameFileParser {
    fn default() -> Self {
        Self::new().expect("Failed to create default FrameFileParser")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_js_endpoint_extraction() {
        let parser = JavaScriptParser::new().unwrap();
        let js = r#"
            fetch('/api/users');
            axios.get('/api/posts');
            const endpoint = '/api/comments';
        "#;
        let base = Url::parse("https://example.com").unwrap();
        let endpoints = parser.extract_endpoints(js, &base);

        assert!(endpoints.iter().any(|u| u.path() == "/api/users"));
        assert!(endpoints.iter().any(|u| u.path() == "/api/posts"));
        assert!(endpoints.iter().any(|u| u.path() == "/api/comments"));
    }

    #[test]
    fn test_template_variable_replacement() {
        let parser = JavaScriptParser::new().unwrap();
        let js = r#"
            fetch('/api/users/${userId}');
            fetch('/api/items/{id}');
            fetch('/api/posts/:slug');
        "#;
        let base = Url::parse("https://example.com").unwrap();
        let endpoints = parser.extract_endpoints(js, &base);

        // Should replace template variables with placeholder values
        assert!(endpoints.iter().any(|u| u.path() == "/api/users/0"));
        assert!(endpoints.iter().any(|u| u.path() == "/api/items/1"));
        assert!(endpoints.iter().any(|u| u.path() == "/api/posts/example"));
    }

    #[test]
    fn test_frame_file_json_extraction() {
        let parser = FrameFileParser::new().unwrap();
        let frame_content = r#"
        {
            "api": {
                "endpoint": "/api/v1/data",
                "path": "/api/v1/users"
            }
        }
        "#;
        let base = Url::parse("https://example.com").unwrap();
        let endpoints = parser.extract_endpoints(frame_content, &base);

        assert!(endpoints.iter().any(|u| u.path() == "/api/v1/data"));
        assert!(endpoints.iter().any(|u| u.path() == "/api/v1/users"));
    }

    #[test]
    fn test_websocket_extraction() {
        let parser = JavaScriptParser::new().unwrap();
        let js = r#"
            const ws = new WebSocket('wss://example.com/socket');
        "#;
        let base = Url::parse("https://example.com").unwrap();
        let endpoints = parser.extract_endpoints(js, &base);

        assert!(endpoints
            .iter()
            .any(|u| u.as_str() == "wss://example.com/socket"));
    }
}
