use crate::error::Result;
use crate::framework::{detect_framework, get_compiled_framework_patterns, Framework};
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
    r#"axios\(\s*\{[^}]*url\s*:\s*["']([^"']+)["']"#,
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
    // React Router patterns
    r#"<Route\s+path=["']([^"']+)["']"#,
    r#"useNavigate\s*\(\s*\)\s*\(\s*["']([^"']+)["']"#,
    // Angular routing
    r#"RouterModule\.forRoot\([^)]*path:\s*["']([^"']+)["']"#,
    r#"\.navigate\(\s*\[["']([^"']+)["']"#,
    // Vue Router
    r#"router\.push\(\s*["']([^"']+)["']"#,
    // Next.js API routes
    r#"/api/[^"'\s]+"#,
    // Express-like route definitions
    r#"(app|router)\.(get|post|put|delete|patch)\s*\(\s*["']([^"']+)["']"#,
    // Import statements with URLs
    r#"import\s+.*\s+from\s+["']([^"']+)["']"#,
];

/// Confidence scores for each pattern in `JS_URL_PATTERNS` (0.0 – 1.0).
///
/// A higher score means the match is very likely to be a real, reachable
/// endpoint.  A lower score means the pattern is more speculative (e.g. a
/// general string literal or an import path).
static JS_URL_PATTERN_CONFIDENCE: &[f32] = &[
    0.6,  // absolute URL in string quotes
    0.5,  // relative path in string quotes (very broad)
    0.9,  // fetch() with string literal
    0.75, // fetch() with template literal (variable substitution applied)
    0.9,  // XMLHttpRequest .open()
    0.9,  // axios.method() with string
    0.85, // axios({ url: })
    0.85, // $.ajax({ url: })
    0.85, // $.get/.post()
    0.7,  // api/endpoint/url/path/route assignment
    0.7,  // template literal /api/…
    0.65, // template literal https://…
    0.75, // path: "…"
    0.8,  // route: "…"
    0.8,  // graphql/gql endpoint reference
    0.8,  // WebSocket wss?://
    0.75, // rpc: "…"
    0.8,  // .get("…")
    0.8,  // .post("…")
    0.8,  // .put("…")
    0.8,  // .delete("…")
    0.8,  // .patch("…")
    0.85, // <Route path= (React Router)
    0.85, // useNavigate() (React Router)
    0.85, // RouterModule.forRoot() (Angular)
    0.85, // .navigate([…]) (Angular)
    0.85, // router.push() (Vue Router)
    0.75, // /api/… literal (Next.js style)
    0.9,  // Express app/router.method()
    0.3,  // import … from "…" (usually a module path, rarely an endpoint)
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
        // Verify that the confidence array stays in sync with the pattern array.
        debug_assert_eq!(
            JS_URL_PATTERNS.len(),
            JS_URL_PATTERN_CONFIDENCE.len(),
            "JS_URL_PATTERNS and JS_URL_PATTERN_CONFIDENCE must have the same length"
        );

        let patterns = JS_URL_PATTERNS
            .iter()
            .map(|p| Regex::new(p))
            .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(Self { patterns })
    }

    /// Extract endpoints from JavaScript content
    pub fn extract_endpoints(&self, js_content: &str, base_url: &Url) -> Vec<Url> {
        self.extract_endpoints_with_confidence(js_content, base_url)
            .into_iter()
            .map(|(url, _)| url)
            .collect()
    }

    /// Extract endpoints from JavaScript content together with a confidence score.
    ///
    /// Each returned `(Url, f32)` pair contains the discovered endpoint and a
    /// confidence value in `[0.0, 1.0]` indicating how reliable the extraction
    /// pattern is.  Callers can filter on the confidence value to reduce false
    /// positives from speculative patterns (e.g. import paths).
    ///
    /// Framework-specific patterns are assigned a confidence of 0.85.
    pub fn extract_endpoints_with_confidence(
        &self,
        js_content: &str,
        base_url: &Url,
    ) -> Vec<(Url, f32)> {
        // Use a map so that if the same URL is matched by multiple patterns we
        // keep the *highest* confidence score for it.
        let mut endpoint_confidence: std::collections::HashMap<String, (Url, f32)> =
            std::collections::HashMap::new();

        let insert =
            |map: &mut std::collections::HashMap<String, (Url, f32)>, url: Url, confidence: f32| {
                let key = url.as_str().to_string();
                let entry = map.entry(key).or_insert((url.clone(), confidence));
                if confidence > entry.1 {
                    *entry = (url, confidence);
                }
            };

        // Standard pattern matching
        for (pattern, &confidence) in self.patterns.iter().zip(JS_URL_PATTERN_CONFIDENCE.iter()) {
            for cap in pattern.captures_iter(js_content) {
                // Extract URL from all capture groups (most patterns have 1-2 groups)
                // Group 0 is the full match, groups 1+ are captured values
                for i in 1..cap.len() {
                    if let Some(url_match) = cap.get(i) {
                        let url_str = url_match.as_str();

                        // Try to resolve as absolute or relative URL
                        if let Ok(url) = self.normalize_and_resolve(url_str, base_url) {
                            insert(&mut endpoint_confidence, url, confidence);
                        }
                    }
                }
            }
        }

        // Detect frameworks and apply framework-specific patterns (confidence 0.85)
        let frameworks = detect_framework(js_content);
        for framework in &frameworks {
            if let Some(framework_endpoints) =
                self.extract_framework_endpoints(js_content, base_url, framework)
            {
                for url in framework_endpoints {
                    insert(&mut endpoint_confidence, url, 0.85);
                }
            }
        }

        endpoint_confidence.into_values().collect()
    }

    /// Extract endpoints using framework-specific patterns
    fn extract_framework_endpoints(
        &self,
        js_content: &str,
        base_url: &Url,
        framework: &Framework,
    ) -> Option<Vec<Url>> {
        // Use pre-compiled regexes to avoid re-compiling on every call.
        let patterns = get_compiled_framework_patterns(framework);
        if patterns.is_empty() {
            return None;
        }

        let mut endpoints = Vec::new();

        for pattern in patterns {
            for cap in pattern.captures_iter(js_content) {
                for i in 1..cap.len() {
                    if let Some(url_match) = cap.get(i) {
                        let url_str = url_match.as_str();
                        if let Ok(url) = self.normalize_and_resolve(url_str, base_url) {
                            endpoints.push(url);
                        }
                    }
                }
            }
        }

        Some(endpoints)
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
        Self::new().unwrap_or_else(|e| panic!("Failed to create default JavaScriptParser: {}", e))
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
        Self::new().unwrap_or_else(|e| panic!("Failed to create default FrameFileParser: {}", e))
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
