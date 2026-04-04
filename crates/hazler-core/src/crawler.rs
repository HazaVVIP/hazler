use crate::config::Config;
use crate::delay::DelayConfig;
use crate::noise_filter::NoiseFilter;
use crate::normalizer::AdvancedUrlNormalizer;
use crate::queue::UrlQueue;
use crate::scope::ScopeValidator;
use crate::types::{CrawlResult, Finding, FindingStats, Page, Severity, ValidEndpoint};
use hazler_http::HttpClient;
use hazler_js_parser::{FrameFileParser, JavaScriptParser, SourceMapParser};
use hazler_parser::{GraphQLParser, HtmlParser};
use hazler_secrets::SecretScanner;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::Semaphore;
use tracing::{debug, error, info, warn};
use url::Url;

#[cfg(feature = "browser")]
use hazler_browser::{Browser, BrowserConfig};

/// Context for crawling a page, containing all necessary dependencies
struct CrawlPageContext {
    http_client: HttpClient,
    parser: HtmlParser,
    js_parser: JavaScriptParser,
    frame_parser: FrameFileParser,
    graphql_parser: GraphQLParser,
    sourcemap_parser: SourceMapParser,
    url_normalizer: AdvancedUrlNormalizer,
    scope_validator: ScopeValidator,
    max_depth: usize,
    aggressive: bool,
    secret_scanner: Option<SecretScanner>,
    noise_filter: Arc<Mutex<NoiseFilter>>,
    delay_config: Option<DelayConfig>,
    #[allow(dead_code)]
    graphql_introspect: bool,
    parse_source_maps: bool,
    /// Minimum confidence threshold for JS-extracted endpoints.
    js_confidence_threshold: f32,
    #[cfg(feature = "browser")]
    browser: Option<Arc<Browser>>,
}

/// Main crawler implementation
pub struct Crawler {
    config: Config,
    http_client: HttpClient,
    parser: HtmlParser,
    js_parser: JavaScriptParser,
    frame_parser: FrameFileParser,
    graphql_parser: GraphQLParser,
    sourcemap_parser: SourceMapParser,
    url_normalizer: AdvancedUrlNormalizer,
    secret_scanner: Option<SecretScanner>,
    /// Optional channel for real-time verified-endpoint delivery.
    endpoint_tx: Option<UnboundedSender<ValidEndpoint>>,
    /// Tracks URLs already emitted through the channel to prevent duplicates.
    emitted_urls: Arc<Mutex<HashSet<String>>>,
    #[cfg(feature = "browser")]
    browser: Option<Arc<Browser>>,
}

impl Crawler {
    /// Create a new crawler with the given configuration
    pub fn new(config: Config) -> anyhow::Result<Self> {
        let mut http_client =
            HttpClient::new(&config.user_agent, Duration::from_secs(config.timeout_secs))?;

        // Enable User-Agent rotation and Chrome hints if stealth mode is enabled
        if config.stealth_mode {
            http_client = http_client
                .with_user_agent_rotation(true)
                .with_chrome_hints(true);
            info!("Stealth mode enabled: User-Agent rotation and Chrome hints activated");
        }

        let parser = HtmlParser::new();
        let js_parser = JavaScriptParser::new()
            .map_err(|e| anyhow::anyhow!("Failed to create JS parser: {}", e))?;
        let frame_parser = FrameFileParser::new()
            .map_err(|e| anyhow::anyhow!("Failed to create frame parser: {}", e))?;
        let graphql_parser = GraphQLParser::new();
        let sourcemap_parser = SourceMapParser::new();
        let url_normalizer = AdvancedUrlNormalizer::new();

        // Initialize secret scanner if secrets scanning is enabled
        let secret_scanner = if config.secrets_scanning {
            Some(SecretScanner::new())
        } else {
            None
        };

        Ok(Self {
            config,
            http_client,
            parser,
            js_parser,
            frame_parser,
            graphql_parser,
            sourcemap_parser,
            url_normalizer,
            secret_scanner,
            endpoint_tx: None,
            emitted_urls: Arc::new(Mutex::new(HashSet::new())),
            #[cfg(feature = "browser")]
            browser: None,
        })
    }

    /// Initialize the headless browser (must be called separately to handle async)
    #[cfg(feature = "browser")]
    pub async fn init_browser(&mut self) -> anyhow::Result<()> {
        if self.config.use_headless_browser {
            info!("Initializing headless browser...");

            let browser_config = BrowserConfig {
                headless: true,
                timeout_secs: self.config.timeout_secs,
                window_width: 1920,
                window_height: 1080,
                intercept_requests: false,
                screenshot_path: self.config.screenshot_path.clone(),
                user_agent: Some(self.config.user_agent.clone()),
                disable_images: self.config.disable_images,
                disable_javascript: false,
            };

            let browser = Browser::new(browser_config)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to initialize browser: {}", e))?;

            self.browser = Some(Arc::new(browser));
            info!("Browser initialized successfully");
        }
        Ok(())
    }

    /// Attach a channel sender for real-time valid-endpoint delivery.
    ///
    /// When a sender is provided every crawled page that passes the validity
    /// checks (correct status, non-error body, above minimum body length, not
    /// noise-filtered) is sent through the channel **before** being added to the
    /// final `CrawlResult`.  Duplicate URLs are suppressed automatically.
    ///
    /// The caller owns the corresponding receiver and can print or process
    /// endpoints as they arrive.  The channel is implicitly closed when the
    /// crawler is dropped.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use hazler_core::{Config, Crawler};
    /// use tokio::sync::mpsc::unbounded_channel;
    /// use url::Url;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    /// let crawler = Crawler::new(Config::new())?.with_endpoint_sender(tx);
    ///
    /// let display = tokio::spawn(async move {
    ///     while let Some(ep) = rx.recv().await {
    ///         println!("{} {}", ep.status_code, ep.url);
    ///     }
    /// });
    ///
    /// let result = crawler.crawl(Url::parse("https://example.com")?).await?;
    /// drop(crawler); // closes the channel so the display task exits
    /// display.await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn with_endpoint_sender(mut self, tx: UnboundedSender<ValidEndpoint>) -> Self {
        self.endpoint_tx = Some(tx);
        self
    }

    /// Start crawling from a given URL
    pub async fn crawl(&self, start_url: Url) -> anyhow::Result<CrawlResult> {
        info!("Starting crawl from: {}", start_url);

        let mut queue = UrlQueue::new();
        let mut result = CrawlResult::new();

        // Configure scope validator based on config settings
        let mut scope_validator = ScopeValidator::new(&start_url);

        // Configure scope behavior:
        // - If strict_domain is enabled: only exact domain (no subdomains)
        // - Otherwise, if allow_subdomains is enabled: include subdomains
        // - Default: only exact domain (allow_subdomains=false)
        if !self.config.strict_domain && self.config.allow_subdomains {
            scope_validator = scope_validator.allow_subdomains(true);
        }

        // Create noise filter for smart rate limiting
        let noise_filter = Arc::new(Mutex::new(NoiseFilter::with_threshold(5)));

        // Configure request timing if stealth mode is enabled
        let delay_config = if self.config.stealth_mode {
            Some(DelayConfig::stealth())
        } else {
            None
        };

        // Add the starting URL
        queue.push(start_url.clone(), 0);

        // Semaphore for concurrency control
        let semaphore = Arc::new(Semaphore::new(self.config.concurrency));
        let mut active_tasks = Vec::new();

        while !queue.is_empty() || !active_tasks.is_empty() {
            // Launch new tasks if we have capacity and URLs
            while !queue.is_empty() && semaphore.available_permits() > 0 {
                if let Some((url, depth)) = queue.pop() {
                    // Check max pages limit
                    if self.config.max_pages > 0 && result.total_pages >= self.config.max_pages {
                        info!("Reached max pages limit: {}", self.config.max_pages);
                        break;
                    }

                    // Check depth limit
                    if depth > self.config.max_depth {
                        continue;
                    }

                    let semaphore = Arc::clone(&semaphore);
                    let http_client = self.http_client.clone();
                    let parser = self.parser.clone();
                    let js_parser = self.js_parser.clone();
                    let frame_parser = self.frame_parser.clone();
                    let graphql_parser = self.graphql_parser.clone();
                    let sourcemap_parser = self.sourcemap_parser.clone();
                    let url_normalizer = self.url_normalizer.clone();
                    let scope_validator = scope_validator.clone();
                    let max_depth = self.config.max_depth;
                    let aggressive = self.config.aggressive_discovery;
                    let secret_scanner = self.secret_scanner.clone();
                    let noise_filter = Arc::clone(&noise_filter);
                    let delay_config = delay_config.clone();
                    let graphql_introspect = self.config.graphql_introspect;
                    let parse_source_maps = self.config.parse_source_maps;
                    let js_confidence_threshold = self.config.js_confidence_threshold;
                    #[cfg(feature = "browser")]
                    let browser = self.browser.clone();

                    let task = tokio::spawn(async move {
                        let _permit = match semaphore.acquire().await {
                            Ok(permit) => permit,
                            Err(e) => {
                                error!("Failed to acquire semaphore: {}", e);
                                return Err(anyhow::anyhow!("Semaphore acquisition failed: {}", e));
                            }
                        };
                        let context = CrawlPageContext {
                            http_client,
                            parser,
                            js_parser,
                            frame_parser,
                            graphql_parser,
                            sourcemap_parser,
                            url_normalizer,
                            scope_validator,
                            max_depth,
                            aggressive,
                            secret_scanner,
                            noise_filter,
                            delay_config,
                            graphql_introspect,
                            parse_source_maps,
                            js_confidence_threshold,
                            #[cfg(feature = "browser")]
                            browser,
                        };
                        Self::crawl_page(url, depth, context).await
                    });

                    active_tasks.push(task);
                }
            }

            // Wait for at least one task to complete
            if !active_tasks.is_empty() {
                let (completed_result, _index, remaining) =
                    futures::future::select_all(active_tasks).await;
                active_tasks = remaining;

                match completed_result {
                    Ok(Ok((page, new_urls))) => {
                        result.total_pages += 1;

                        // Emit verified endpoints in real-time through the channel.
                        // A page qualifies only if it was not noise-filtered and it
                        // passes the full validity check (status, body, length).
                        if let Some(ref tx) = self.endpoint_tx {
                            if !page.was_noise_filtered && Self::is_valid_endpoint(&page) {
                                let canonical = page.url.as_str().to_string();
                                let should_send = {
                                    match self.emitted_urls.lock() {
                                        Ok(mut set) => set.insert(canonical),
                                        Err(e) => {
                                            debug!(
                                                "emitted_urls mutex poisoned, skipping dedup: {}",
                                                e
                                            );
                                            false
                                        }
                                    }
                                };
                                if should_send {
                                    let _ = tx.send(ValidEndpoint {
                                        url: page.url.clone(),
                                        status_code: page.status_code,
                                        content_type: page.content_type.clone().unwrap_or_default(),
                                    });
                                }
                            }
                        }

                        // Add new URLs to queue
                        for (new_url, new_depth) in new_urls {
                            if queue.push(new_url, new_depth) {
                                result.total_urls += 1;
                            }
                        }

                        result.pages.push(page);
                    }
                    Ok(Err(e)) => {
                        error!("Crawl error: {}", e);
                        result.errors.push(e.to_string());
                    }
                    Err(e) => {
                        error!("Task error: {}", e);
                        result.errors.push(format!("Task panic: {}", e));
                    }
                }
            }

            // Check if we should stop
            if self.config.max_pages > 0 && result.total_pages >= self.config.max_pages {
                break;
            }
        }

        info!(
            "Crawl completed. Pages: {}, URLs discovered: {}, Errors: {}",
            result.total_pages,
            result.total_urls,
            result.errors.len()
        );

        // Calculate secret findings statistics if secrets scanning was enabled
        if self.config.secrets_scanning {
            let mut stats = FindingStats::default();
            for page in &result.pages {
                for finding in &page.secrets {
                    stats.total += 1;
                    match finding.severity {
                        Severity::Critical => stats.critical += 1,
                        Severity::High => stats.high += 1,
                        Severity::Medium => stats.medium += 1,
                        Severity::Low => stats.low += 1,
                    }
                }
            }
            if stats.total > 0 {
                info!(
                    "Secret findings: {} total (Critical: {}, High: {}, Medium: {}, Low: {})",
                    stats.total, stats.critical, stats.high, stats.medium, stats.low
                );
                result.secret_findings = Some(stats);
            }
        }

        Ok(result)
    }

    /// Crawl a single page
    async fn crawl_page(
        url: Url,
        depth: usize,
        context: CrawlPageContext,
    ) -> anyhow::Result<(Page, Vec<(Url, usize)>)> {
        info!("Crawling: {} (depth: {})", url, depth);

        // Check if we should use browser mode for this URL
        #[cfg(feature = "browser")]
        let use_browser = context.browser.is_some() && Self::should_use_browser(&url);

        // Use browser or HTTP client based on configuration
        #[cfg(feature = "browser")]
        if use_browser {
            return Self::crawl_page_with_browser(url, depth, context).await;
        }

        // Default HTTP-based crawling
        Self::crawl_page_with_http(url, depth, context).await
    }

    /// Determine whether a crawled `Page` is a truly valid, reachable endpoint.
    ///
    /// A page is considered valid when **all** of the following hold:
    /// 1. The HTTP status code is in the 2xx range.
    /// 2. The response body is longer than `MIN_VALID_BODY_LEN` bytes (very short
    ///    responses are almost always empty error stubs).
    /// 3. The response body does not begin with soft-error keywords (custom
    ///    error pages that return HTTP 200).
    ///
    /// This function is the single gate used to decide whether a page's URL is
    /// emitted through the real-time valid-endpoint channel, ensuring users only
    /// see genuinely reachable endpoints.
    fn is_valid_endpoint(page: &Page) -> bool {
        // Only 2xx responses are considered valid endpoints.
        if !(200..300).contains(&page.status_code) {
            return false;
        }

        // Very short responses are almost always error stubs.
        const MIN_VALID_BODY_LEN: usize = 64;
        if page.body.len() < MIN_VALID_BODY_LEN {
            // Exception: API JSON endpoints can legitimately return small
            // bodies (e.g. `{}` or `{"ok":true}`).  Allow short bodies when
            // the content-type indicates JSON or a non-HTML data format.
            let is_data_type = page
                .content_type
                .as_deref()
                .map(|ct| {
                    ct.contains("application/json")
                        || ct.contains("text/plain")
                        || ct.contains("application/xml")
                        || ct.contains("text/xml")
                        || ct.contains("text/csv")
                })
                .unwrap_or(false);
            if !is_data_type {
                return false;
            }
        }

        // Reject soft-error bodies (custom error pages returning HTTP 200).
        if Self::is_soft_error_body(&page.body) {
            return false;
        }

        true
    }

    /// Detect whether a response body indicates a soft-error page — i.e. a
    /// custom 403/404/etc. page that was served with HTTP 200.
    ///
    /// Only the first 4 KB of the body is inspected so that the check is cheap
    /// even for large responses.  Real error pages put the message near the top,
    /// while legitimate pages that merely *mention* permission errors (e.g. help
    /// docs) tend to be much longer and contain the keyword deep in the body.
    fn is_soft_error_body(body: &str) -> bool {
        // Find a safe UTF-8 boundary at or before 4096 bytes so we never split
        // a multi-byte character.
        let limit = body
            .char_indices()
            .map(|(i, _)| i)
            .take_while(|&i| i < 4096)
            .last()
            .map(|i| {
                body[i..]
                    .chars()
                    .next()
                    .map(|c| i + c.len_utf8())
                    .unwrap_or(i)
            })
            .unwrap_or(body.len().min(4096));
        let lower = body[..limit].to_lowercase();

        // ── Soft-403 indicators ──────────────────────────────────────────────
        lower.contains("403 forbidden")
            || lower.contains("access denied")
            || lower.contains("access is denied")
            || lower.contains("you don't have permission")
            || lower.contains("you do not have permission")
            || lower.contains("permission denied")
            || lower.contains("not authorized")
            || lower.contains("unauthorized access")
            || lower.contains("you are not allowed")
            || lower.contains("forbidden")
            // ── Soft-404 indicators ──────────────────────────────────────────
            || lower.contains("404 not found")
            || lower.contains("page not found")
            || lower.contains("resource not found")
            || lower.contains("this page does not exist")
            || lower.contains("this page doesn't exist")
            || lower.contains("no such page")
            || lower.contains("the resource you requested")
            || lower.contains("nothing here")
            || lower.contains("error 404")
            // ── Generic error indicators ─────────────────────────────────────
            || lower.contains("invalid request")
            || lower.contains("bad request")
            || lower.contains("no results found")
            // ── Nginx / Apache default error pages ───────────────────────────
            || (lower.contains("<title>")
                && lower.contains("error")
                && (lower.contains("nginx") || lower.contains("apache")))
            // ── Cloudflare error indicators ──────────────────────────────────
            || lower.contains("cf-error-code")
            || lower.contains("cf_chl_opt")
            // ── Common CMS error indicators ──────────────────────────────────
            || (lower.contains("wordpress") && lower.contains("not found"))
    }

    /// Kept as a compatibility shim used by the secret-scanning path.
    ///
    /// Delegates to `is_soft_error_body` so both paths share the same
    /// expanded keyword list.
    #[inline]
    fn is_soft_forbidden_body(body: &str) -> bool {
        Self::is_soft_error_body(body)
    }

    /// Determine if we should use browser for this URL
    /// Browser is useful for HTML pages but not for API endpoints or static files
    #[cfg(feature = "browser")]
    fn should_use_browser(url: &Url) -> bool {
        let path = url.path();
        // Don't use browser for known static/API resources
        if path.ends_with(".js")
            || path.ends_with(".json")
            || path.ends_with(".css")
            || path.ends_with(".jpg")
            || path.ends_with(".png")
            || path.ends_with(".gif")
            || path.ends_with(".svg")
            || path.ends_with(".xml")
            || path.ends_with(".frame") // .frame files contain endpoint definitions (custom format)
            || path.contains("/api/")
        {
            return false;
        }
        // Use browser for likely HTML pages
        true
    }

    /// Crawl a page using headless browser
    ///
    /// # Known Limitations
    /// - Body content is not returned (empty string) to reduce memory usage
    /// - Secret scanning is not performed in browser mode
    /// - Future enhancement: Extract rendered HTML from browser
    #[cfg(feature = "browser")]
    async fn crawl_page_with_browser(
        url: Url,
        depth: usize,
        context: CrawlPageContext,
    ) -> anyhow::Result<(Page, Vec<(Url, usize)>)> {
        info!("Crawling with browser: {} (depth: {})", url, depth);

        let browser = context
            .browser
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Browser not initialized"))?;

        // Load page with browser
        let result = browser
            .load_page(&url)
            .await
            .map_err(|e| anyhow::anyhow!("Browser page load failed for {}: {}", url, e))?;

        // Convert browser links (strings) to Urls
        let mut links = Vec::new();
        for link_str in &result.links {
            match Url::parse(link_str) {
                Ok(link_url) => links.push(link_url),
                Err(_) => {
                    // Try resolving relative URL
                    if let Ok(resolved) = url.join(link_str) {
                        links.push(resolved);
                    }
                }
            }
        }

        // Add network requests as additional URLs to crawl (API endpoints discovered)
        for network_req in &result.network_requests {
            // Parse network request URL and add to links
            if let Ok(req_url) = Url::parse(&network_req.url) {
                // Only add if it's in scope and looks interesting
                if context.scope_validator.is_in_scope(&req_url) {
                    // Prioritize API endpoints
                    if network_req.url.contains("/api/")
                        || network_req.url.contains("/graphql")
                        || network_req.url.contains("/v1/")
                        || network_req.url.contains("/v2/")
                        || network_req.resource_type.contains("XHR")
                        || network_req.resource_type.contains("Fetch")
                    {
                        info!(
                            "Adding API endpoint from network request: {} {}",
                            network_req.method, network_req.url
                        );
                        links.push(req_url);
                    }
                }
            }
        }

        // If aggressive mode, generate URL variants
        if context.aggressive {
            let mut variants = Vec::new();
            for link in &links {
                variants.extend(context.url_normalizer.normalize(link));
                variants.extend(context.url_normalizer.generate_api_variations(link));
            }
            links.extend(variants);
        }

        // Deduplicate links
        let mut seen = std::collections::HashSet::new();
        links.retain(|link| {
            let canonical = context.url_normalizer.canonicalize(link);
            seen.insert(canonical)
        });

        // Create page object using fully rendered HTML body from the browser
        let body = result.rendered_html.unwrap_or_default();
        let mut page = Page::new(result.url.clone(), result.status_code, body.clone(), depth);
        page.links = links.clone();
        page.content_type = Some("text/html".to_string());

        // Run secret scanning on rendered HTML body if scanner is configured
        if let Some(ref scanner) = context.secret_scanner {
            if !body.is_empty() {
                let findings = scanner.scan(&body, result.url.as_str());
                if !findings.is_empty() {
                    info!(
                        "Found {} secret(s) in browser-rendered page at {}",
                        findings.len(),
                        result.url
                    );
                }
                page.secrets = findings
                    .into_iter()
                    .map(|f| crate::types::Finding {
                        secret_type: f.secret_type,
                        severity: match f.severity {
                            hazler_secrets::Severity::Critical => Severity::Critical,
                            hazler_secrets::Severity::High => Severity::High,
                            hazler_secrets::Severity::Medium => Severity::Medium,
                            hazler_secrets::Severity::Low => Severity::Low,
                        },
                        description: f.description,
                        line: f.line,
                        column: f.column,
                        context: f.context,
                        matched_text: f.matched_text,
                        location: f.location,
                    })
                    .collect();
            }
        }

        // Filter links by scope and depth
        let new_urls: Vec<(Url, usize)> = links
            .into_iter()
            .filter(|link| context.scope_validator.is_in_scope(link))
            .filter(|_| depth < context.max_depth)
            .map(|link| (link, depth + 1))
            .collect();

        Ok((page, new_urls))
    }

    /// Crawl a page using HTTP client
    async fn crawl_page_with_http(
        url: Url,
        depth: usize,
        context: CrawlPageContext,
    ) -> anyhow::Result<(Page, Vec<(Url, usize)>)> {
        // Apply request delay if configured (for WAF evasion)
        if let Some(ref delay_config) = context.delay_config {
            let delay = delay_config.get_delay();
            debug!("Applying request delay: {:?}", delay);
            tokio::time::sleep(delay).await;
        }

        // Fetch the page
        let response = context.http_client.fetch(&url).await?;

        // Check if this response pattern is noise (WAF blocks, etc.)
        let content_length = response.body.len();
        let should_filter = {
            let filter_result = context.noise_filter.lock();
            match filter_result {
                Ok(mut filter) => filter.should_filter(response.status_code, content_length),
                Err(e) => {
                    debug!("Noise filter mutex poisoned: {}, disabling filter", e);
                    false // Continue without filtering if mutex is poisoned
                }
            }
        };

        if should_filter {
            debug!(
                "Filtering response from {} (status: {}, length: {}) as noise",
                url, response.status_code, content_length
            );
            // Return early with an empty, noise-flagged page and no new URLs.
            let mut page = Page::new(url.clone(), response.status_code, String::new(), depth);
            page.was_noise_filtered = true;
            return Ok((page, Vec::new()));
        }

        let content_type = response.content_type.as_deref().unwrap_or("");
        let mut links = Vec::new();

        // Determine which parser to use based on content type and file extension
        if content_type.contains("text/html") {
            // HTML content - use HTML parser
            match context.parser.extract_links(&response.body, &url) {
                Ok(extracted_links) => {
                    links = extracted_links;
                }
                Err(e) => {
                    warn!("Failed to parse links from {}: {}", url, e);
                }
            }

            // Check for GraphQL endpoint references in HTML
            if let Some(graphql_endpoint) = context
                .graphql_parser
                .detect_graphql_endpoint(&url, &response.body)
            {
                info!(
                    "GraphQL endpoint detected at {} (confidence: {:.2})",
                    graphql_endpoint.url, graphql_endpoint.confidence
                );
                debug!("GraphQL indicators: {:?}", graphql_endpoint.indicators);
            }
        } else if content_type.contains("javascript")
            || content_type.contains("application/json")
            || url.path().ends_with(".js")
            || url.path().ends_with(".json")
        {
            // JavaScript or JSON content - use JS parser with confidence filtering.
            // Only endpoints that score at or above the configured threshold are
            // queued for crawling, which reduces noise from speculative patterns.
            let extracted_with_confidence = context
                .js_parser
                .extract_endpoints_with_confidence(&response.body, &url);
            let threshold = context.js_confidence_threshold;
            let before = extracted_with_confidence.len();
            let filtered: Vec<Url> = extracted_with_confidence
                .into_iter()
                .filter(|(_, confidence)| *confidence >= threshold)
                .map(|(url, _)| url)
                .collect();
            debug!(
                "Extracted {} endpoints from JavaScript at {} ({} below confidence threshold {:.2})",
                filtered.len(),
                url,
                before - filtered.len(),
                threshold
            );
            links = filtered;

            // Check for GraphQL endpoint
            if let Some(graphql_endpoint) = context
                .graphql_parser
                .detect_graphql_endpoint(&url, &response.body)
            {
                info!(
                    "GraphQL endpoint detected at {} (confidence: {:.2})",
                    graphql_endpoint.url, graphql_endpoint.confidence
                );
                debug!("GraphQL indicators: {:?}", graphql_endpoint.indicators);
            }

            // Check for source map references in JS files
            if context.parse_source_maps && url.path().ends_with(".js") {
                let source_map_refs = context
                    .sourcemap_parser
                    .detect_source_map_references(&response.body, &url);
                if !source_map_refs.is_empty() {
                    info!(
                        "Found {} source map reference(s) for {}",
                        source_map_refs.len(),
                        url
                    );
                    for sm_ref in source_map_refs {
                        if !sm_ref.inline {
                            debug!("Source map URL: {}", sm_ref.map_url);
                            // Add source map URL to links to be crawled
                            links.push(sm_ref.map_url);
                        }
                    }
                }
            }

            // Parse source map files (.map extension)
            if context.parse_source_maps && url.path().ends_with(".map") {
                match context.sourcemap_parser.parse_source_map(&response.body) {
                    Ok(source_map) => {
                        let analysis = context
                            .sourcemap_parser
                            .analyze_source_map(&source_map, url.as_str());
                        let report = context.sourcemap_parser.generate_report(&analysis);
                        info!("{}", report);
                    }
                    Err(e) => {
                        debug!("Failed to parse source map at {}: {}", url, e);
                    }
                }
            }
        } else if url.path().ends_with(".frame") {
            // Frame file - use frame parser
            let extracted = context.frame_parser.extract_endpoints(&response.body, &url);
            info!(
                "Extracted {} endpoints from .frame file at {}",
                extracted.len(),
                url
            );
            links = extracted;
        }

        // If aggressive mode is enabled, also try JS parser on HTML content
        if context.aggressive && content_type.contains("text/html") {
            let threshold = context.js_confidence_threshold;
            let js_endpoints: Vec<Url> = context
                .js_parser
                .extract_endpoints_with_confidence(&response.body, &url)
                .into_iter()
                .filter(|(_, c)| *c >= threshold)
                .map(|(u, _)| u)
                .collect();
            if !js_endpoints.is_empty() {
                debug!(
                    "Extracted {} additional endpoints from inline JS at {}",
                    js_endpoints.len(),
                    url
                );
                links.extend(js_endpoints);
            }
        }

        // If aggressive mode, generate URL variants
        if context.aggressive {
            let mut variants = Vec::new();
            for link in &links {
                variants.extend(context.url_normalizer.normalize(link));
                // Also try API variations for API-looking URLs
                variants.extend(context.url_normalizer.generate_api_variations(link));
            }
            links.extend(variants);
        }

        // Deduplicate links using canonicalization
        let mut seen = std::collections::HashSet::new();
        links.retain(|link| {
            let canonical = context.url_normalizer.canonicalize(link);
            seen.insert(canonical)
        });

        // Create page object
        let mut page = Page::new(
            url.clone(),
            response.status_code,
            response.body.clone(),
            depth,
        );
        page.headers = response.headers;
        page.content_type = response.content_type;
        page.links = links.clone();

        // Scan for secrets if enabled, but only for successful responses.
        // Skip scanning on 4xx/5xx error pages (404 Not Found, 403 Forbidden, etc.)
        // and responses that indicate soft-forbidden / soft-404 content, as these
        // pages rarely contain real secrets and generate excessive false positives.
        if let Some(ref scanner) = context.secret_scanner {
            let is_error_response = response.status_code >= 400;
            let is_soft_forbidden =
                response.status_code == 200 && Self::is_soft_forbidden_body(&response.body);

            if !is_error_response && !is_soft_forbidden {
                let findings = scanner.scan(&response.body, url.as_str());
                if !findings.is_empty() {
                    info!("Found {} secret(s) at {}", findings.len(), url);
                    // Convert hazler_secrets::Finding to our Finding type
                    page.secrets = findings
                        .into_iter()
                        .map(|f| Finding {
                            secret_type: f.secret_type,
                            severity: match f.severity {
                                hazler_secrets::Severity::Critical => Severity::Critical,
                                hazler_secrets::Severity::High => Severity::High,
                                hazler_secrets::Severity::Medium => Severity::Medium,
                                hazler_secrets::Severity::Low => Severity::Low,
                            },
                            description: f.description,
                            line: f.line,
                            column: f.column,
                            context: f.context,
                            matched_text: f.matched_text,
                            location: f.location,
                        })
                        .collect();
                }
            } else if is_error_response {
                debug!(
                    "Skipping secret scan for {} (HTTP {})",
                    url, response.status_code
                );
            } else {
                debug!(
                    "Skipping secret scan for {} (soft-forbidden body detected)",
                    url
                );
            }
        }

        // Filter links by scope and depth
        let new_urls: Vec<(Url, usize)> = links
            .into_iter()
            .filter(|link| context.scope_validator.is_in_scope(link))
            .filter(|_| depth < context.max_depth)
            .map(|link| (link, depth + 1))
            .collect();

        Ok((page, new_urls))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_crawler() {
        let config = Config::default();
        let crawler = Crawler::new(config);
        assert!(crawler.is_ok());
    }

    #[cfg(feature = "browser")]
    #[test]
    fn test_should_use_browser() {
        // HTML pages should use browser
        let html_url = Url::parse("https://example.com/page.html").unwrap();
        assert!(Crawler::should_use_browser(&html_url));

        let root_url = Url::parse("https://example.com/").unwrap();
        assert!(Crawler::should_use_browser(&root_url));

        // Static files should NOT use browser
        let js_url = Url::parse("https://example.com/app.js").unwrap();
        assert!(!Crawler::should_use_browser(&js_url));

        let json_url = Url::parse("https://example.com/data.json").unwrap();
        assert!(!Crawler::should_use_browser(&json_url));

        let css_url = Url::parse("https://example.com/style.css").unwrap();
        assert!(!Crawler::should_use_browser(&css_url));

        // API endpoints should NOT use browser
        let api_url = Url::parse("https://example.com/api/users").unwrap();
        assert!(!Crawler::should_use_browser(&api_url));

        // Images should NOT use browser
        let img_url = Url::parse("https://example.com/image.png").unwrap();
        assert!(!Crawler::should_use_browser(&img_url));

        let jpg_url = Url::parse("https://example.com/photo.jpg").unwrap();
        assert!(!Crawler::should_use_browser(&jpg_url));
    }
}
