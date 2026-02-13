use url::Url;

/// Validates if a URL is within the crawl scope
#[derive(Debug, Clone)]
pub struct ScopeValidator {
    /// Base domains to stay within
    base_domains: Vec<String>,
    /// Whether to allow subdomains
    allow_subdomains: bool,
}

impl ScopeValidator {
    /// Create a new scope validator for a given URL
    pub fn new(base_url: &Url) -> Self {
        let domain = base_url.host_str().unwrap_or_default().to_string();

        Self {
            base_domains: vec![domain],
            allow_subdomains: false,
        }
    }

    /// Create a validator with multiple base domains
    pub fn with_domains(domains: Vec<String>) -> Self {
        Self {
            base_domains: domains,
            allow_subdomains: false,
        }
    }

    /// Set whether to allow subdomains
    pub fn allow_subdomains(mut self, allow: bool) -> Self {
        self.allow_subdomains = allow;
        self
    }

    /// Check if a URL is within scope
    pub fn is_in_scope(&self, url: &Url) -> bool {
        let url_host = match url.host_str() {
            Some(host) => host,
            None => return false,
        };

        for base_domain in &self.base_domains {
            if url_host == base_domain {
                return true;
            }

            if self.allow_subdomains && url_host.ends_with(&format!(".{}", base_domain)) {
                return true;
            }
        }

        false
    }

    /// Normalize a URL (remove fragment, sort query params, etc.)
    pub fn normalize_url(&self, url: &Url) -> Url {
        let mut normalized = url.clone();
        normalized.set_fragment(None);
        normalized
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_same_domain() {
        let base = Url::parse("https://example.com").unwrap();
        let validator = ScopeValidator::new(&base);

        let url = Url::parse("https://example.com/page").unwrap();
        assert!(validator.is_in_scope(&url));
    }

    #[test]
    fn test_different_domain() {
        let base = Url::parse("https://example.com").unwrap();
        let validator = ScopeValidator::new(&base);

        let url = Url::parse("https://other.com/page").unwrap();
        assert!(!validator.is_in_scope(&url));
    }

    #[test]
    fn test_subdomain_disallowed() {
        let base = Url::parse("https://example.com").unwrap();
        let validator = ScopeValidator::new(&base);

        let url = Url::parse("https://sub.example.com/page").unwrap();
        assert!(!validator.is_in_scope(&url));
    }

    #[test]
    fn test_subdomain_allowed() {
        let base = Url::parse("https://example.com").unwrap();
        let validator = ScopeValidator::new(&base).allow_subdomains(true);

        let url = Url::parse("https://sub.example.com/page").unwrap();
        assert!(validator.is_in_scope(&url));
    }
}
