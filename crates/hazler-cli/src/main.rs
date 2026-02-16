use clap::Parser;
use colored::Colorize;
use hazler_core::{Config, Crawler};
use hazler_http::{ApiKeyLocation, AuthConfig, AuthMethod, FormAuth};
use std::collections::HashMap;
use std::fs;
use std::process;
use tracing::{error, info, Level};
use url::Url;

mod output;
use output::{generate_report, OutputFormatter};

mod html_report;
use html_report::generate_html_report;

mod pdf_report;
use pdf_report::generate_pdf_report;

mod sqlite_export;
use sqlite_export::export_to_sqlite;

mod webhook;

mod export_formats;
use export_formats::{format_openapi, format_postman};

mod fuzzer_integration;
use fuzzer_integration::apply_fuzzing;

#[derive(Parser, Debug)]
#[command(name = "hazler")]
#[command(author = "Hazler Team")]
#[command(version = "0.1.0")]
#[command(about = "Next-Generation Intelligent Web Crawler", long_about = None)]
struct Args {
    /// Target URL to crawl (use '-' to read URLs from stdin for pipeline mode)
    #[arg(value_name = "URL")]
    url: String,

    /// Maximum crawl depth
    #[arg(short = 'd', long, default_value = "3")]
    max_depth: usize,

    /// Number of concurrent requests
    #[arg(short = 'c', long, default_value = "10")]
    concurrency: usize,

    /// Maximum number of pages to crawl (0 = unlimited)
    #[arg(short = 'p', long, default_value = "0")]
    max_pages: usize,

    /// Custom user agent string
    #[arg(short = 'u', long, default_value = "Hazler/0.1.0")]
    user_agent: String,

    /// Request timeout in seconds
    #[arg(short = 't', long, default_value = "10")]
    timeout: u64,

    /// Output format (json, jsonl, urls, csv, tree, nuclei, ffuf, burp, openapi, or postman)
    #[arg(short = 'o', long, default_value = "tree")]
    output_format: String,

    /// Include response body in output (excluded by default for clean output)
    #[arg(long)]
    include_body: bool,

    /// Select specific fields to output (comma-separated: url,status_code,depth,links)
    #[arg(long)]
    fields: Option<String>,

    /// Export results in various formats
    /// Format: TYPE:FILE where TYPE can be summary, html, pdf, sqlite, openapi, or postman
    /// Can be specified multiple times for multiple exports
    /// Examples:
    ///   --export html:report.html
    ///   --export pdf:report.pdf --export sqlite:data.db
    ///   --export summary:summary.txt
    #[arg(long, value_name = "TYPE:FILE")]
    export: Vec<String>,

    /// Send results to webhook
    /// Webhook type is auto-detected from URL pattern:
    /// - Slack: hooks.slack.com
    /// - Discord: discord.com/api/webhooks
    /// - Generic: all other URLs
    /// Examples:
    ///   --webhook https://hooks.slack.com/services/...
    ///   --webhook https://discord.com/api/webhooks/...
    ///   --webhook https://example.com/webhook --webhook-type generic
    #[arg(long, value_name = "URL")]
    webhook: Option<String>,

    /// Webhook type (optional, auto-detected by default)
    /// Options: slack, discord, generic
    /// Only needed if auto-detection fails or you want to override
    #[arg(long, value_name = "TYPE")]
    webhook_type: Option<String>,

    /// Verbose output
    #[arg(short = 'v', long)]
    verbose: bool,

    /// Enable aggressive endpoint discovery mode
    /// - Applies regex patterns to JavaScript files
    /// - Generates URL variations
    /// - Discovers API endpoints more thoroughly
    ///
    /// Warning: This may generate more requests
    #[arg(long)]
    aggressive: bool,

    /// Enable comprehensive scanning mode (--all)
    /// Activates all analysis features:
    /// - Deep crawling with increased limits
    /// - JavaScript endpoint extraction
    /// - Secret and sensitive data scanning
    /// - Framework detection
    /// - API endpoint mapping
    /// - GraphQL introspection
    /// - Comprehensive reporting
    #[arg(long)]
    all: bool,

    /// Disable stealth mode (enabled by default)
    /// Stealth mode helps evade WAF detection by:
    /// - Randomizing request patterns
    /// - Implementing adaptive rate limiting
    /// - Maintaining session state
    /// - Using realistic browser headers
    #[arg(long)]
    no_stealth: bool,

    /// Disable secret scanning (enabled by default)
    /// Secret scanning detects:
    /// - API keys and tokens
    /// - Credentials and passwords
    /// - Internal information leakage
    #[arg(long)]
    no_secrets: bool,

    /// Proxy URL (e.g., socks5://localhost:1080, http://proxy:8080)
    #[arg(long)]
    proxy: Option<String>,

    /// Domain crawling scope
    /// Controls which domains are crawled:
    /// - strict: Only the exact domain (no subdomains)
    /// - same-domain: Same domain without subdomains (default)
    /// - subdomains: Include all subdomains
    /// Examples:
    ///   --scope strict  (example.com only, not sub.example.com)
    ///   --scope subdomains  (example.com and sub.example.com)
    #[arg(long, default_value = "same-domain", value_name = "MODE")]
    scope: String,

    /// Enable headless browser for JavaScript-heavy sites (SPAs)
    /// Uses Chrome/Chromium via CDP to render pages with JavaScript
    /// Allows crawling modern SPAs (React, Vue, Angular, etc.)
    #[arg(long)]
    browser: bool,

    /// Path to save screenshots when using headless browser
    /// Screenshots are saved as PNG files
    /// Example: --screenshot-path screenshots/
    #[arg(long)]
    screenshot_path: Option<String>,

    /// Disable images in headless browser for faster loading
    /// When enabled, the browser will not load images
    #[arg(long)]
    disable_images: bool,

    /// Enable GraphQL introspection queries
    /// Automatically runs introspection queries on detected GraphQL endpoints
    /// to extract schema information (types, queries, mutations)
    #[arg(long)]
    graphql_introspect: bool,

    /// Disable source map parsing (enabled by default)
    /// Source maps reveal original source code structure and paths
    /// including potentially sensitive admin panels and API endpoints
    #[arg(long)]
    no_source_maps: bool,

    /// Enable smart fuzzing mode
    /// Automatically generates URL variations to discover hidden endpoints:
    /// - Pluralization (user -> users)
    /// - File extensions (.json, .xml, .php)
    /// - API versioning (v1, v2, v3)
    #[arg(long)]
    fuzz: bool,

    /// Enable parameter discovery fuzzing
    /// Tests common parameter names on discovered endpoints
    #[arg(long)]
    fuzz_params: bool,

    /// Enable endpoint fuzzing with wordlists
    /// Tests common endpoint paths and variations
    #[arg(long)]
    fuzz_endpoints: bool,

    /// Fuzzing aggressiveness level (minimal, default, aggressive)
    #[arg(long, default_value = "default")]
    fuzz_level: String,

    /// Save responses as baseline for future comparison
    /// Saves normalized response hashes to a JSON file for later comparison
    /// Example: --baseline baseline.json
    #[arg(long, value_name = "FILE")]
    baseline: Option<String>,

    /// Compare current responses against a baseline
    /// Detects changes in responses by comparing against saved baseline
    /// Example: --compare baseline.json
    #[arg(long, value_name = "FILE")]
    compare: Option<String>,

    /// Threshold for considering responses as similar (0.0 to 1.0)
    /// Default: 0.85 (85% similarity)
    #[arg(long, default_value = "0.85")]
    diff_threshold: f64,

    /// Enable response clustering
    /// Groups similar responses together using K-means or DBSCAN
    #[arg(long)]
    cluster_responses: bool,

    /// Clustering algorithm (kmeans or dbscan)
    #[arg(long, default_value = "kmeans")]
    cluster_algorithm: String,

    /// Number of clusters for K-means
    #[arg(long, default_value = "5")]
    num_clusters: usize,

    /// Resume from saved state file
    /// Continues crawling from where it was left off
    /// Example: --resume hazler-state.json
    #[arg(long, value_name = "FILE")]
    resume: Option<String>,

    /// Auto-save state every N seconds (0 to disable)
    /// Periodically saves crawl state for recovery
    /// Example: --auto-save 30
    #[arg(long, default_value = "60")]
    auto_save: u64,

    /// Maximum retry attempts for failed requests
    /// Uses exponential backoff between retries
    /// Example: --max-retries 3
    #[arg(long, default_value = "3")]
    max_retries: u32,

    /// Enable circuit breaker for failing domains
    /// Temporarily stops requests to domains with repeated failures
    #[arg(long)]
    circuit_breaker: bool,

    /// Requests per second per domain (rate limiting)
    /// Controls the crawl rate to avoid overwhelming servers
    /// Example: --rate-limit 10
    #[arg(long, default_value = "10")]
    rate_limit: f64,

    /// Progress reporting interval in seconds
    /// Shows crawl progress at regular intervals
    /// Example: --progress 5
    #[arg(long, default_value = "5")]
    progress: u64,

    // ===== Authentication Options =====
    /// Basic Auth credentials (username:password)
    /// Example: --auth-basic "user:pass"
    #[arg(long, value_name = "CREDENTIALS")]
    auth_basic: Option<String>,

    /// Bearer token for authentication
    /// Example: --auth-bearer "eyJhbGc..."
    #[arg(long, value_name = "TOKEN")]
    auth_bearer: Option<String>,

    /// Cookie for authentication (name=value format, can be repeated)
    /// Example: --auth-cookie "session=abc123"
    #[arg(long, value_name = "COOKIE")]
    auth_cookie: Vec<String>,

    /// Custom header for authentication (Name:Value format)
    /// Example: --auth-header "X-API-Key:secret"
    #[arg(long, value_name = "HEADER")]
    auth_header: Option<String>,

    /// API key for authentication
    /// Example: --auth-apikey "your-api-key"
    #[arg(long, value_name = "KEY")]
    auth_apikey: Option<String>,

    /// API key location (header, query, or cookie)
    /// Default: header
    #[arg(long, default_value = "header")]
    auth_apikey_location: String,

    /// API key name (header/param/cookie name)
    /// Default: X-API-Key
    #[arg(long, default_value = "X-API-Key")]
    auth_apikey_name: String,

    /// OAuth2 access token
    /// Example: --auth-oauth "access-token"
    #[arg(long, value_name = "TOKEN")]
    auth_oauth: Option<String>,

    /// Load authentication configuration from JSON file
    /// Example: --auth-file credentials.json
    #[arg(long, value_name = "FILE")]
    auth_file: Option<String>,

    /// Form-based login URL
    /// Example: --auth-form-url "https://example.com/login"
    #[arg(long, value_name = "URL")]
    auth_form_url: Option<String>,

    /// Form username field name
    /// Default: username
    #[arg(long, default_value = "username")]
    auth_form_user_field: String,

    /// Form password field name
    /// Default: password
    #[arg(long, default_value = "password")]
    auth_form_pass_field: String,

    /// Form username value
    #[arg(long, value_name = "USERNAME")]
    auth_form_username: Option<String>,

    /// Form password value
    #[arg(long, value_name = "PASSWORD")]
    auth_form_password: Option<String>,
}

/// Export specification parsed from TYPE:FILE format
#[derive(Debug, Clone)]
struct ExportSpec {
    export_type: String,
    file_path: String,
}

/// Parse export specifications from command line arguments
fn parse_export_specs(export_args: &[String]) -> Result<Vec<ExportSpec>, String> {
    let mut specs = Vec::new();
    
    for arg in export_args {
        let parts: Vec<&str> = arg.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err(format!(
                "Invalid export format: '{}'. Expected format: TYPE:FILE",
                arg
            ));
        }
        
        let export_type = parts[0].trim().to_lowercase();
        let file_path = parts[1].trim().to_string();
        
        // Validate export type
        match export_type.as_str() {
            "summary" | "html" | "pdf" | "sqlite" | "openapi" | "postman" => {
                specs.push(ExportSpec {
                    export_type,
                    file_path,
                });
            }
            _ => {
                return Err(format!(
                    "Unknown export type: '{}'. Supported types: summary, html, pdf, sqlite, openapi, postman",
                    export_type
                ));
            }
        }
    }
    
    Ok(specs)
}

/// Webhook type for different webhook integrations
#[derive(Debug, Clone, PartialEq)]
enum WebhookType {
    Slack,
    Discord,
    Generic,
}

/// Domain scoping mode for crawling
#[derive(Debug, Clone, PartialEq)]
enum ScopeMode {
    Strict,        // Only exact domain
    SameDomain,    // Same domain (default, no subdomains)
    Subdomains,    // Include all subdomains
}

/// Detect webhook type from URL pattern
fn detect_webhook_type(url: &str) -> WebhookType {
    if url.contains("hooks.slack.com") {
        WebhookType::Slack
    } else if url.contains("discord.com/api/webhooks") {
        WebhookType::Discord
    } else {
        WebhookType::Generic
    }
}

/// Parse webhook type from string
fn parse_webhook_type(type_str: &str) -> Result<WebhookType, String> {
    match type_str.to_lowercase().as_str() {
        "slack" => Ok(WebhookType::Slack),
        "discord" => Ok(WebhookType::Discord),
        "generic" => Ok(WebhookType::Generic),
        _ => Err(format!(
            "Unknown webhook type: '{}'. Supported types: slack, discord, generic",
            type_str
        )),
    }
}

/// Parse scope mode from string
fn parse_scope_mode(scope_str: &str) -> Result<ScopeMode, String> {
    match scope_str.to_lowercase().as_str() {
        "strict" => Ok(ScopeMode::Strict),
        "same-domain" | "same_domain" => Ok(ScopeMode::SameDomain),
        "subdomains" | "subs" => Ok(ScopeMode::Subdomains),
        _ => Err(format!(
            "Unknown scope mode: '{}'. Supported modes: strict, same-domain, subdomains",
            scope_str
        )),
    }
}

/// Build authentication configuration from CLI arguments
fn build_auth_config(args: &Args) -> Result<Option<AuthConfig>, String> {
    // Load from file if provided
    if let Some(ref auth_file) = args.auth_file {
        let content = fs::read_to_string(auth_file)
            .map_err(|e| format!("Failed to read auth file: {}", e))?;
        let config = AuthConfig::from_json(&content)
            .map_err(|e| format!("Failed to parse auth file: {}", e))?;
        return Ok(Some(config));
    }

    // Build from CLI arguments
    let mut auth_method = None;

    // Check for Basic Auth
    if let Some(ref creds) = args.auth_basic {
        let parts: Vec<&str> = creds.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err("Basic auth must be in format username:password".to_string());
        }
        auth_method = Some(AuthMethod::Basic {
            username: parts[0].to_string(),
            password: parts[1].to_string(),
        });
    }

    // Check for Bearer Token
    if let Some(ref token) = args.auth_bearer {
        if auth_method.is_some() {
            return Err(
                "Multiple authentication methods specified. Please use only one.".to_string(),
            );
        }
        auth_method = Some(AuthMethod::Bearer {
            token: token.clone(),
        });
    }

    // Check for Cookie Auth
    if !args.auth_cookie.is_empty() {
        if auth_method.is_some() {
            return Err(
                "Multiple authentication methods specified. Please use only one.".to_string(),
            );
        }
        let mut cookies = HashMap::new();
        for cookie in &args.auth_cookie {
            let parts: Vec<&str> = cookie.splitn(2, '=').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid cookie format: {}. Use name=value", cookie));
            }
            cookies.insert(parts[0].to_string(), parts[1].to_string());
        }
        auth_method = Some(AuthMethod::Cookie { cookies });
    }

    // Check for Custom Header
    if let Some(ref header) = args.auth_header {
        if auth_method.is_some() {
            return Err(
                "Multiple authentication methods specified. Please use only one.".to_string(),
            );
        }
        let parts: Vec<&str> = header.splitn(2, ':').collect();
        if parts.len() != 2 {
            return Err("Custom header must be in format Name:Value".to_string());
        }
        auth_method = Some(AuthMethod::Header {
            name: parts[0].to_string(),
            value: parts[1].to_string(),
        });
    }

    // Check for API Key
    if let Some(ref key) = args.auth_apikey {
        if auth_method.is_some() {
            return Err(
                "Multiple authentication methods specified. Please use only one.".to_string(),
            );
        }
        let location = match args.auth_apikey_location.to_lowercase().as_str() {
            "header" => ApiKeyLocation::Header,
            "query" => ApiKeyLocation::Query,
            "cookie" => ApiKeyLocation::Cookie,
            _ => return Err("API key location must be: header, query, or cookie".to_string()),
        };
        auth_method = Some(AuthMethod::ApiKey {
            key: key.clone(),
            location,
            name: args.auth_apikey_name.clone(),
        });
    }

    // Check for OAuth2
    if let Some(ref token) = args.auth_oauth {
        if auth_method.is_some() {
            return Err(
                "Multiple authentication methods specified. Please use only one.".to_string(),
            );
        }
        auth_method = Some(AuthMethod::OAuth2 {
            access_token: token.clone(),
            token_type: Some("Bearer".to_string()),
            refresh_token: None,
            expires_in: None,
        });
    }

    // Build form auth if provided
    let form_auth = if let Some(ref url) = args.auth_form_url {
        if args.auth_form_username.is_none() || args.auth_form_password.is_none() {
            return Err(
                "Form auth requires --auth-form-username and --auth-form-password".to_string(),
            );
        }
        Some(FormAuth {
            login_url: url.clone(),
            username_field: args.auth_form_user_field.clone(),
            password_field: args.auth_form_pass_field.clone(),
            username: args.auth_form_username.as_ref().unwrap().clone(),
            password: args.auth_form_password.as_ref().unwrap().clone(),
            extra_fields: HashMap::new(),
            follow_redirects: true,
        })
    } else {
        None
    };

    // Return auth config if any method was specified
    if let Some(method) = auth_method {
        let mut config = AuthConfig::new(method);
        if let Some(form) = form_auth {
            config = config.with_form_auth(form);
        }
        Ok(Some(config))
    } else if form_auth.is_some() {
        Err("Form auth URL specified but no authentication method provided".to_string())
    } else {
        Ok(None)
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // Setup logging
    let log_level = if args.verbose {
        Level::DEBUG
    } else {
        Level::INFO
    };
    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_target(false)
        .init();

    // Parse the URL(s) - support pipeline mode with stdin
    let urls: Vec<Url> = if args.url == "-" {
        // Pipeline mode: read URLs from stdin
        use std::io::{self, BufRead};
        let stdin = io::stdin();
        let mut urls = Vec::new();

        for line in stdin.lock().lines() {
            match line {
                Ok(url_str) => {
                    let url_str = url_str.trim();
                    if url_str.is_empty() || url_str.starts_with('#') {
                        continue; // Skip empty lines and comments
                    }
                    match Url::parse(url_str) {
                        Ok(url) => urls.push(url),
                        Err(e) => {
                            eprintln!("Warning: Skipping invalid URL '{}': {}", url_str, e);
                        }
                    }
                }
                Err(e) => {
                    error!("Error reading from stdin: {}", e);
                    process::exit(1);
                }
            }
        }

        if urls.is_empty() {
            error!("No valid URLs provided via stdin");
            process::exit(1);
        }

        urls
    } else {
        // Normal mode: single URL from command line
        match Url::parse(&args.url) {
            Ok(url) => vec![url],
            Err(e) => {
                error!("Invalid URL '{}': {}", args.url, e);
                process::exit(1);
            }
        }
    };

    // Validate browser-related flags
    if !args.browser && (args.screenshot_path.is_some() || args.disable_images) {
        error!("--screenshot-path and --disable-images require --browser flag to be enabled");
        eprintln!("Use --browser to enable headless browser mode");
        process::exit(1);
    }

    // Apply --all mode defaults if enabled
    let (max_depth, max_pages, aggressive, enable_secrets, enable_stealth) = if args.all {
        // --all mode: comprehensive scanning
        let depth = if args.max_depth == 3 {
            5
        } else {
            args.max_depth
        }; // Increase depth if using default
        let pages = if args.max_pages == 0 {
            0
        } else {
            args.max_pages
        }; // Keep unlimited or user value
        (depth, pages, true, true, !args.no_stealth) // Force aggressive mode, enable secrets
    } else {
        // Normal mode: stealth and secrets are enabled by default (can be disabled with flags)
        (
            args.max_depth,
            args.max_pages,
            args.aggressive,
            !args.no_secrets,
            !args.no_stealth,
        )
    };

    // Parse authentication configuration early (before borrowing args for other values)
    let auth_config = match build_auth_config(&args) {
        Ok(config) => config,
        Err(e) => {
            error!("Authentication configuration error: {}", e);
            process::exit(1);
        }
    };

    // Display authentication info if configured
    if let Some(ref auth) = auth_config {
        info!(
            "Authentication enabled: {}",
            auth.method.sanitized_display()
        );
    }

    // Configure the crawler
    let mut config = Config::new()
        .max_depth(max_depth)
        .concurrency(args.concurrency)
        .max_pages(max_pages)
        .user_agent(args.user_agent.clone()) // Clone needed because args was borrowed earlier for auth config
        .timeout_secs(args.timeout)
        .aggressive(aggressive);

    // Apply stealth mode based on flag (defaults to enabled)
    config = config.stealth(enable_stealth);

    // Apply scope control options
    let scope_mode = match parse_scope_mode(&args.scope) {
        Ok(mode) => mode,
        Err(e) => {
            error!("Invalid scope mode: {}", e);
            process::exit(1);
        }
    };
    
    match scope_mode {
        ScopeMode::Strict => {
            config = config.strict_domain(true);
        }
        ScopeMode::SameDomain => {
            // Default behavior - no additional config needed
        }
        ScopeMode::Subdomains => {
            config = config.allow_subdomains(true);
        }
    }

    // Apply proxy if provided
    if let Some(proxy_url) = args.proxy {
        config = config.proxy(proxy_url);
    }

    // Apply secrets scanning based on flag (defaults to enabled)
    config = config.secrets_scanning(enable_secrets);

    // Apply browser settings
    if args.browser {
        config = config.headless_browser(true);

        if let Some(screenshot_path) = args.screenshot_path {
            config = config.screenshot_path(screenshot_path);
        }

        if args.disable_images {
            config = config.disable_images(true);
        }
    }

    // Apply GraphQL and Source Map settings
    // GraphQL introspection is automatically enabled with --all, or can be enabled independently
    let enable_graphql = args.graphql_introspect || args.all;
    config = config.graphql_introspect(enable_graphql);
    config = config.parse_source_maps(!args.no_source_maps);

    // Create and run crawler (mutable to support browser initialization)
    let mut crawler = match Crawler::new(config) {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to create crawler: {}", e);
            process::exit(1);
        }
    };

    // Initialize browser if enabled
    if args.browser {
        match crawler.init_browser().await {
            Ok(_) => info!("Browser initialized successfully"),
            Err(e) => {
                error!("Failed to initialize browser: {}", e);
                process::exit(1);
            }
        }
    }

    // Crawl all URLs (supports pipeline mode with multiple URLs)
    use hazler_core::CrawlResult;
    let mut combined_result = CrawlResult {
        pages: Vec::new(),
        total_pages: 0,
        total_urls: 0,
        errors: Vec::new(),
        secret_findings: None,
    };

    for (idx, start_url) in urls.iter().enumerate() {
        if urls.len() > 1 {
            info!("Crawling URL {}/{}: {}", idx + 1, urls.len(), start_url);
        }

        match crawler.crawl(start_url.clone()).await {
            Ok(result) => {
                // Merge results into combined result
                combined_result.pages.extend(result.pages);
                combined_result.total_pages += result.total_pages;
                combined_result.total_urls += result.total_urls;
                combined_result.errors.extend(result.errors);

                // Merge secret findings
                if let Some(ref findings) = result.secret_findings {
                    if let Some(ref mut combined_findings) = combined_result.secret_findings {
                        combined_findings.total += findings.total;
                        combined_findings.critical += findings.critical;
                        combined_findings.high += findings.high;
                        combined_findings.medium += findings.medium;
                        combined_findings.low += findings.low;
                    } else {
                        combined_result.secret_findings = Some(findings.clone());
                    }
                }
            }
            Err(e) => {
                error!("Failed to crawl {}: {}", start_url, e);
                combined_result.errors.push(format!("{}: {}", start_url, e));
            }
        }
    }

    // Process the combined results
    let result = combined_result;

    // Apply fuzzing if enabled
    if args.fuzz || args.fuzz_params || args.fuzz_endpoints {
        // Extract unique URLs from crawled pages
        let mut discovered_urls: Vec<Url> =
            result.pages.iter().map(|page| page.url.clone()).collect();

        // Remove duplicates
        discovered_urls.sort_by(|a, b| a.as_str().cmp(b.as_str()));
        discovered_urls.dedup();

        // Apply fuzzing
        let fuzzed_urls = apply_fuzzing(
            &discovered_urls,
            args.fuzz,
            args.fuzz_params,
            args.fuzz_endpoints,
            &args.fuzz_level,
        );

        // Note: In a real implementation, you would crawl these fuzzed URLs
        // For now, we just report them
        if !fuzzed_urls.is_empty() {
            info!("Generated {} fuzzed URLs for testing", fuzzed_urls.len());
            // TODO: Optionally crawl fuzzed URLs or output them separately
        }
    }

    // Handle baseline and comparison if requested
    if args.baseline.is_some() || args.compare.is_some() {
        use hazler_core::{DifferConfig, ResponseDiffer};

        let diff_config = DifferConfig {
            similarity_threshold: args.diff_threshold,
            enable_noise_filtering: true,
            enable_clustering: args.cluster_responses,
            clustering_algorithm: args.cluster_algorithm.clone(),
            num_clusters: args.num_clusters,
            dbscan_epsilon: 0.3,
            dbscan_min_points: 2,
        };

        // Save baseline mode
        if let Some(baseline_path) = &args.baseline {
            let mut differ =
                ResponseDiffer::with_baseline(diff_config.clone(), baseline_path.clone());

            for page in &result.pages {
                if let Err(e) = differ.save_baseline(page.url.as_str(), &page.body) {
                    error!("Failed to save baseline for {}: {}", page.url, e);
                }
            }

            if let Some(manager) = differ.baseline_manager_mut() {
                if let Err(e) = manager.save() {
                    error!("Failed to save baseline file: {}", e);
                } else {
                    eprintln!(
                        "{} Baseline saved: {} ({} responses)",
                        "✓".green().bold(),
                        baseline_path.bright_cyan(),
                        result.pages.len()
                    );
                }
            }
        }

        // Compare mode
        if let Some(compare_path) = &args.compare {
            let mut differ =
                ResponseDiffer::with_baseline(diff_config.clone(), compare_path.clone());

            // Load baseline
            if let Some(manager) = differ.baseline_manager_mut() {
                if let Err(e) = manager.load() {
                    error!("Failed to load baseline file {}: {}", compare_path, e);
                    eprintln!("Make sure the baseline file exists and was created with --baseline");
                    process::exit(1);
                }
            }

            eprintln!("\n{}", "Response Comparison Report".bright_cyan().bold());
            eprintln!("{}", "=".repeat(50));

            let mut changes_found = 0;
            let mut unchanged = 0;

            for page in &result.pages {
                if let Some(similarity) =
                    differ.compare_with_baseline(page.url.as_str(), &page.body)
                {
                    let change_pct = (1.0 - similarity) * 100.0;

                    if similarity < diff_config.similarity_threshold {
                        changes_found += 1;
                        eprintln!(
                            "{} {} ({:.1}% change)",
                            "⚠".yellow().bold(),
                            page.url.as_str().bright_yellow(),
                            change_pct
                        );
                    } else {
                        unchanged += 1;
                        if args.verbose {
                            eprintln!(
                                "{} {} ({:.1}% similar)",
                                "✓".green(),
                                page.url.as_str(),
                                similarity * 100.0
                            );
                        }
                    }
                } else {
                    eprintln!(
                        "{} {} (new - not in baseline)",
                        "📌".bright_blue().bold(),
                        page.url.as_str().bright_blue()
                    );
                }
            }

            eprintln!("\n{}", "Summary".bright_cyan().bold());
            eprintln!("{}", "-".repeat(50));
            eprintln!("Total responses: {}", result.pages.len());
            eprintln!("Changes detected: {}", changes_found);
            eprintln!("Unchanged: {}", unchanged);
            eprintln!(
                "Similarity threshold: {:.0}%",
                diff_config.similarity_threshold * 100.0
            );
        }

        // Clustering mode (if enabled)
        if args.cluster_responses {
            use hazler_core::{DBSCANClusterer, KMeansClusterer, SimHashCalculator};

            eprintln!("\n{}", "Response Clustering".bright_cyan().bold());
            eprintln!("{}", "=".repeat(50));

            let calculator = SimHashCalculator::new();
            let responses: Vec<(String, hazler_core::SimHash)> = result
                .pages
                .iter()
                .map(|page| (page.url.to_string(), calculator.calculate(&page.body)))
                .collect();

            let clusters = match args.cluster_algorithm.as_str() {
                "kmeans" => {
                    let clusterer = KMeansClusterer::new(args.num_clusters);
                    clusterer.cluster(&responses)
                }
                "dbscan" => {
                    let clusterer = DBSCANClusterer::new(0.3, 2);
                    clusterer.cluster(&responses)
                }
                _ => {
                    error!("Invalid clustering algorithm: {}", args.cluster_algorithm);
                    Vec::new()
                }
            };

            for cluster in &clusters {
                eprintln!(
                    "\n{} Cluster {} ({} URLs, cohesion: {:.1}%)",
                    "📊".bold(),
                    cluster.id,
                    cluster.urls.len(),
                    cluster.cohesion * 100.0
                );
                for url in &cluster.urls {
                    eprintln!("  - {}", url);
                }
            }

            eprintln!("\nTotal clusters: {}", clusters.len());
        }
    }

    // Send to webhook if requested
    if let Some(webhook_url) = &args.webhook {
        // Determine webhook type (auto-detect or use explicit type)
        let webhook_type = if let Some(ref type_str) = args.webhook_type {
            match parse_webhook_type(type_str) {
                Ok(wt) => wt,
                Err(e) => {
                    error!("Invalid webhook type: {}", e);
                    process::exit(1);
                }
            }
        } else {
            // Auto-detect webhook type from URL
            detect_webhook_type(webhook_url)
        };

        // Send to appropriate webhook based on type
        let send_result = match webhook_type {
            WebhookType::Slack => {
                webhook::send_to_slack(&result, webhook_url).await
            }
            WebhookType::Discord => {
                webhook::send_to_discord(&result, webhook_url).await
            }
            WebhookType::Generic => {
                webhook::send_to_webhook(&result, webhook_url).await
            }
        };

        match send_result {
            Ok(_) => {
                let type_name = match webhook_type {
                    WebhookType::Slack => "Slack",
                    WebhookType::Discord => "Discord",
                    WebhookType::Generic => "webhook",
                };
                eprintln!("{} Results sent to {}", "✓".green().bold(), type_name);
            }
            Err(e) => {
                error!("Failed to send to webhook: {}", e);
            }
        }
    }

    // Generate report if requested (to stderr, doesn't interfere with output)
    // Note: Stats are always shown at the end (after results output)
    // The --export summary:file option writes the full report to a file

    // Handle exports using the new consolidated --export argument
    if !args.export.is_empty() {
        match parse_export_specs(&args.export) {
            Ok(export_specs) => {
                for spec in export_specs {
                    match spec.export_type.as_str() {
                        "summary" => {
                            // Export summary report to file
                            let report_content = generate_report(&result);
                            match fs::write(&spec.file_path, report_content) {
                                Ok(_) => {
                                    eprintln!(
                                        "{} Summary report exported: {}",
                                        "✓".green().bold(),
                                        spec.file_path.bright_cyan()
                                    );
                                }
                                Err(e) => {
                                    error!("Failed to export summary report: {}", e);
                                }
                            }
                        }
                        "html" => {
                            match generate_html_report(&result, std::path::Path::new(&spec.file_path))
                            {
                                Ok(_) => {
                                    eprintln!(
                                        "{} HTML report generated: {}",
                                        "✓".green().bold(),
                                        spec.file_path.bright_cyan()
                                    );
                                }
                                Err(e) => {
                                    error!("Failed to generate HTML report: {}", e);
                                }
                            }
                        }
                        "pdf" => {
                            match generate_pdf_report(&result, std::path::Path::new(&spec.file_path))
                            {
                                Ok(_) => {
                                    eprintln!(
                                        "{} PDF report generated: {}",
                                        "✓".green().bold(),
                                        spec.file_path.bright_cyan()
                                    );
                                }
                                Err(e) => {
                                    error!("Failed to generate PDF report: {}", e);
                                }
                            }
                        }
                        "sqlite" => {
                            match export_to_sqlite(&result, std::path::Path::new(&spec.file_path)) {
                                Ok(_) => {
                                    eprintln!(
                                        "{} SQLite database exported: {}",
                                        "✓".green().bold(),
                                        spec.file_path.bright_cyan()
                                    );
                                }
                                Err(e) => {
                                    error!("Failed to export to SQLite: {}", e);
                                }
                            }
                        }
                        "openapi" => {
                            let openapi_spec = format_openapi(&result);
                            match fs::write(&spec.file_path, openapi_spec) {
                                Ok(_) => {
                                    eprintln!(
                                        "{} OpenAPI spec exported: {}",
                                        "✓".green().bold(),
                                        spec.file_path.bright_cyan()
                                    );
                                }
                                Err(e) => {
                                    error!("Failed to export OpenAPI spec: {}", e);
                                }
                            }
                        }
                        "postman" => {
                            let postman_collection = format_postman(&result);
                            match fs::write(&spec.file_path, postman_collection) {
                                Ok(_) => {
                                    eprintln!(
                                        "{} Postman collection exported: {}",
                                        "✓".green().bold(),
                                        spec.file_path.bright_cyan()
                                    );
                                }
                                Err(e) => {
                                    error!("Failed to export Postman collection: {}", e);
                                }
                            }
                        }
                        _ => {
                            error!("Unknown export type: {}", spec.export_type);
                        }
                    }
                }
            }
            Err(e) => {
                error!("Failed to parse export specifications: {}", e);
                process::exit(1);
            }
        }
    }

    // Create output formatter (exclude_body is true by default, unless --include-body is specified)
    let exclude_body = !args.include_body;
    let formatter = OutputFormatter::new(exclude_body, args.fields);

    // Output results based on format
    match args.output_format.as_str() {
        "json" => match formatter.format_json(&result) {
            Ok(json) => println!("{}", json),
            Err(e) => {
                error!("Failed to serialize results: {}", e);
                process::exit(1);
            }
        },
        "jsonl" => match formatter.format_jsonl(&result) {
            Ok(lines) => {
                for line in lines {
                    println!("{}", line);
                }
            }
            Err(_e) => {
            }
        },
        "ffuf" => match formatter.format_ffuf(&result) {
            Ok(lines) => {
                for line in lines {
                    println!("{}", line);
                }
            }
            Err(e) => {
                error!("Failed to serialize ffuf results: {}", e);
                process::exit(1);
            }
        },
        "burp" => {
            println!("{}", formatter.format_burp(&result));
        }
        "openapi" => {
            println!("{}", format_openapi(&result));
        }
        "postman" => {
            println!("{}", format_postman(&result));
        }
        _ => {
            error!(
                        "Unknown output format: {}. Valid formats: json, jsonl, urls, csv, tree, nuclei, ffuf, burp, openapi, postman",
                        args.output_format
                    );
            process::exit(1);
        }
    }

    // Print summary to stderr (always shown after results output)
    eprintln!("\n{}", "═".repeat(80).bright_blue());
        eprintln!("{}", "📝 CRAWL SUMMARY".bright_cyan().bold());
        eprintln!("{}", "═".repeat(80).bright_blue());
        eprintln!(
            "{} {}",
            "Total pages crawled:".bright_white(),
            result.total_pages.to_string().green().bold()
        );
        eprintln!(
            "{} {}",
            "Total URLs discovered:".bright_white(),
            result.total_urls.to_string().cyan().bold()
        );
        eprintln!(
            "{} {}",
            "Errors encountered:".bright_white(),
            if !result.errors.is_empty() {
                result.errors.len().to_string().red().bold()
            } else {
                result.errors.len().to_string().green().bold()
            }
        );

        // Show secrets summary if any found
        if let Some(ref stats) = result.secret_findings {
            if stats.total > 0 {
                eprintln!(
                    "\n{} {}",
                    "🔒 Secrets found:".bright_red().bold(),
                    stats.total.to_string().bright_red().bold()
                );
                if stats.critical > 0 {
                    eprintln!("  {} {}", "Critical:".red(), stats.critical);
                }
                if stats.high > 0 {
                    eprintln!("  {} {}", "High:".yellow(), stats.high);
                }
                if stats.medium > 0 {
                    eprintln!("  {} {}", "Medium:".yellow(), stats.medium);
                }
                if stats.low > 0 {
                    eprintln!("  {} {}", "Low:".cyan(), stats.low);
                }
            }
        }

        if !result.errors.is_empty() && args.verbose {
            eprintln!("\n{}", "⚠️  ERRORS".yellow().bold());
            for error in &result.errors {
                eprintln!("  {} {}", "•".red(), error);
            }
        }

        eprintln!("{}\n", "═".repeat(80).bright_blue());
}
