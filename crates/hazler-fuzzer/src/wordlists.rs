//! Built-in wordlists for fuzzing

use once_cell::sync::Lazy;

/// Common API endpoint paths
pub static COMMON_ENDPOINTS: Lazy<Vec<&'static str>> = Lazy::new(|| {
    vec![
        // User management
        "users",
        "user",
        "profile",
        "account",
        "accounts",
        "login",
        "logout",
        "register",
        "signup",
        "signin",
        // Admin endpoints
        "admin",
        "administrator",
        "management",
        "dashboard",
        "panel",
        "control",
        // API endpoints
        "api",
        "v1",
        "v2",
        "v3",
        "rest",
        "graphql",
        // Data endpoints
        "data",
        "info",
        "details",
        "list",
        "search",
        "query",
        // File operations
        "upload",
        "download",
        "file",
        "files",
        "media",
        "images",
        "documents",
        // Configuration
        "config",
        "settings",
        "options",
        "preferences",
        // Security
        "auth",
        "token",
        "oauth",
        "key",
        "secret",
        // Common resources
        "posts",
        "post",
        "comments",
        "comment",
        "items",
        "item",
        "products",
        "product",
        "orders",
        "order",
    ]
});

/// Common parameter names for fuzzing
pub static COMMON_PARAMS: Lazy<Vec<&'static str>> = Lazy::new(|| {
    vec![
        // Identifiers
        "id",
        "user_id",
        "userId",
        "uid",
        "account_id",
        "accountId",
        "customer_id",
        "customerId",
        "order_id",
        "orderId",
        "product_id",
        "productId",
        // Pagination
        "page",
        "limit",
        "offset",
        "per_page",
        "perPage",
        "count",
        "size",
        // Sorting
        "sort",
        "order",
        "orderBy",
        "sort_by",
        "sortBy",
        "direction",
        "dir",
        // Filtering
        "filter",
        "search",
        "q",
        "query",
        "keyword",
        "term",
        "name",
        "type",
        "status",
        "category",
        "tag",
        // Actions
        "action",
        "method",
        "operation",
        "cmd",
        "command",
        // Security
        "token",
        "api_key",
        "apiKey",
        "key",
        "secret",
        "auth",
        "access_token",
        "accessToken",
        "refresh_token",
        "refreshToken",
        // File operations
        "file",
        "filename",
        "path",
        "url",
        "uri",
        "redirect",
        "redirect_uri",
        "redirectUri",
        "callback",
        "return_url",
        "returnUrl",
        // Data format
        "format",
        "output",
        "response_type",
        "responseType",
        // Debug/Testing
        "debug",
        "test",
        "dev",
        "preview",
        "demo",
    ]
});

/// Common file extensions
pub static FILE_EXTENSIONS: Lazy<Vec<&'static str>> = Lazy::new(|| {
    vec![
        // Data formats
        "json",
        "xml",
        "yaml",
        "yml",
        "csv",
        "txt",
        // Web formats
        "html",
        "htm",
        "xhtml",
        // Server-side scripts
        "php",
        "asp",
        "aspx",
        "jsp",
        "jspx",
        // Other formats
        "pdf",
        "doc",
        "docx",
        "xls",
        "xlsx",
        // Backup/Config
        "bak",
        "backup",
        "old",
        "config",
        "conf",
        "cfg",
        "ini",
    ]
});

/// Wordlist collection
pub struct Wordlists;

impl Wordlists {
    /// Get common endpoint wordlist
    pub fn endpoints() -> &'static [&'static str] {
        &COMMON_ENDPOINTS
    }

    /// Get common parameter wordlist
    pub fn params() -> &'static [&'static str] {
        &COMMON_PARAMS
    }

    /// Get file extension wordlist
    pub fn extensions() -> &'static [&'static str] {
        &FILE_EXTENSIONS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_endpoints_not_empty() {
        let endpoints = Wordlists::endpoints();
        assert!(!endpoints.is_empty(), "Endpoints wordlist should not be empty");
        assert!(endpoints.len() > 50, "Should have at least 50 endpoints");
    }

    #[test]
    fn test_params_not_empty() {
        let params = Wordlists::params();
        assert!(!params.is_empty(), "Params wordlist should not be empty");
        assert!(params.len() > 50, "Should have at least 50 params");
    }

    #[test]
    fn test_extensions_not_empty() {
        let extensions = Wordlists::extensions();
        assert!(!extensions.is_empty(), "Extensions wordlist should not be empty");
        assert!(extensions.len() > 20, "Should have at least 20 extensions");
    }

    #[test]
    fn test_common_endpoints_include_api() {
        let endpoints = Wordlists::endpoints();
        assert!(endpoints.contains(&"api"), "Should include 'api' endpoint");
        assert!(endpoints.contains(&"users"), "Should include 'users' endpoint");
        assert!(endpoints.contains(&"admin"), "Should include 'admin' endpoint");
    }

    #[test]
    fn test_common_params_include_id() {
        let params = Wordlists::params();
        assert!(params.contains(&"id"), "Should include 'id' parameter");
        assert!(params.contains(&"user_id"), "Should include 'user_id' parameter");
        assert!(params.contains(&"token"), "Should include 'token' parameter");
    }
}
