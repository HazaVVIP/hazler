use url::Url;

/// Advanced URL normalizer for discovery and deduplication
#[derive(Clone)]
pub struct AdvancedUrlNormalizer;

impl AdvancedUrlNormalizer {
    /// Create a new URL normalizer
    pub fn new() -> Self {
        Self
    }

    /// Normalize URL with various strategies (generates variants)
    pub fn normalize(&self, url: &Url) -> Vec<Url> {
        let mut variants = Vec::new();

        // 1. Base URL (current behavior)
        let mut normalized = url.clone();
        normalized.set_fragment(None);
        variants.push(normalized.clone());

        // 2. Remove trailing slash
        if let Some(path) = normalized.path().strip_suffix('/') {
            if !path.is_empty() {
                let mut no_slash = normalized.clone();
                no_slash.set_path(path);
                variants.push(no_slash);
            }
        } else if !normalized.path().ends_with('/') {
            // Add trailing slash
            let mut with_slash = normalized.clone();
            with_slash.set_path(&format!("{}/", normalized.path()));
            variants.push(with_slash);
        }

        // 3. Remove query parameters (discover base endpoint)
        if normalized.query().is_some() {
            let mut no_query = normalized.clone();
            no_query.set_query(None);
            variants.push(no_query);
        }

        // 4. Common file extensions
        let path = normalized.path();
        if !path.contains('.') && !path.ends_with('/') {
            // Try common API extensions
            for ext in &["json", "xml", "html", "txt"] {
                let mut with_ext = normalized.clone();
                with_ext.set_path(&format!("{}.{}", path.trim_end_matches('/'), ext));
                variants.push(with_ext);
            }
        }

        // 5. Remove file extension (discover directory)
        if let Some(idx) = path.rfind('.') {
            // Check if dot is in the last path segment (not in directory name)
            let last_slash_idx = path.rfind('/').unwrap_or(0);
            if idx > last_slash_idx {
                let mut no_ext = normalized.clone();
                no_ext.set_path(&path[..idx]);
                variants.push(no_ext);
            }
        }

        // Deduplicate
        variants.sort_by(|a, b| a.as_str().cmp(b.as_str()));
        variants.dedup();

        variants
    }

    /// Generate common API path variations
    pub fn generate_api_variations(&self, url: &Url) -> Vec<Url> {
        let mut variants = Vec::new();
        let path = url.path();

        // If path looks like it might be an API endpoint
        if path.contains("/api/") || path.contains("/v1/") || path.contains("/v2/") {
            // Try different versions
            for version in &["v1", "v2", "v3"] {
                let versioned_path = path
                    .replace("/v1/", &format!("/{}/", version))
                    .replace("/v2/", &format!("/{}/", version))
                    .replace("/v3/", &format!("/{}/", version));

                if versioned_path != path {
                    let mut versioned = url.clone();
                    versioned.set_path(&versioned_path);
                    variants.push(versioned);
                }
            }

            // Try different formats
            for format in &["json", "xml", "yaml"] {
                let mut with_format = url.clone();
                if let Some(query) = url.query() {
                    with_format.set_query(Some(&format!("{}&format={}", query, format)));
                } else {
                    with_format.set_query(Some(&format!("format={}", format)));
                }
                variants.push(with_format);
            }
        }

        variants
    }

    /// Canonicalize URL for deduplication
    pub fn canonicalize(&self, url: &Url) -> String {
        let mut canonical = url.clone();

        // Remove fragment
        canonical.set_fragment(None);

        // Sort query parameters
        if let Some(query) = canonical.query() {
            let mut params: Vec<(&str, &str)> = query
                .split('&')
                .filter_map(|p| {
                    let mut parts = p.splitn(2, '=');
                    Some((parts.next()?, parts.next().unwrap_or("")))
                })
                .collect();
            params.sort();
            let sorted_query: String = params
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("&");
            canonical.set_query(Some(&sorted_query));
        }

        // Lowercase scheme and host
        let mut result = canonical.scheme().to_lowercase();
        result.push_str("://");
        result.push_str(&canonical.host_str().unwrap_or("").to_lowercase());

        if let Some(port) = canonical.port() {
            // Only include port if non-standard
            let standard_port =
                (canonical.scheme() == "http" && port == 80)
                    || (canonical.scheme() == "https" && port == 443);
            if !standard_port {
                result.push(':');
                result.push_str(&port.to_string());
            }
        }

        result.push_str(canonical.path());

        if let Some(query) = canonical.query() {
            result.push('?');
            result.push_str(query);
        }

        result
    }
}

impl Default for AdvancedUrlNormalizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_normalization() {
        let normalizer = AdvancedUrlNormalizer::new();
        let url = Url::parse("https://example.com/path/?query=1#frag").unwrap();
        let variants = normalizer.normalize(&url);

        // Should generate multiple variants
        assert!(variants.len() > 1);
        assert!(variants.iter().any(|u| u.query().is_none()));
        assert!(variants.iter().any(|u| u.fragment().is_none()));
    }

    #[test]
    fn test_trailing_slash_variants() {
        let normalizer = AdvancedUrlNormalizer::new();
        let url = Url::parse("https://example.com/path/").unwrap();
        let variants = normalizer.normalize(&url);

        // Should have variant without trailing slash
        assert!(variants.iter().any(|u| u.path() == "/path"));
    }

    #[test]
    fn test_extension_variants() {
        let normalizer = AdvancedUrlNormalizer::new();
        let url = Url::parse("https://example.com/api/users").unwrap();
        let variants = normalizer.normalize(&url);

        // Should generate variants with extensions
        assert!(variants
            .iter()
            .any(|u| u.path() == "/api/users.json" || u.path() == "/api/users.xml"));
    }

    #[test]
    fn test_api_version_variations() {
        let normalizer = AdvancedUrlNormalizer::new();
        let url = Url::parse("https://example.com/api/v1/users").unwrap();
        let variants = normalizer.generate_api_variations(&url);

        // Should generate v2 and v3 variants
        assert!(variants.iter().any(|u| u.path() == "/api/v2/users"));
        assert!(variants.iter().any(|u| u.path() == "/api/v3/users"));
    }

    #[test]
    fn test_canonicalize() {
        let normalizer = AdvancedUrlNormalizer::new();
        let url1 = Url::parse("https://Example.com:443/path?b=2&a=1#frag").unwrap();
        let url2 = Url::parse("https://example.com/path?a=1&b=2").unwrap();

        let canon1 = normalizer.canonicalize(&url1);
        let canon2 = normalizer.canonicalize(&url2);

        // Should be the same after canonicalization
        assert_eq!(canon1, canon2);
    }

    #[test]
    fn test_canonicalize_non_standard_port() {
        let normalizer = AdvancedUrlNormalizer::new();
        let url = Url::parse("https://example.com:8080/path").unwrap();
        let canonical = normalizer.canonicalize(&url);

        // Should keep non-standard port
        assert!(canonical.contains(":8080"));
    }
}
