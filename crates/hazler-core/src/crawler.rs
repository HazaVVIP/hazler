use crate::config::Config;
use crate::noise_filter::NoiseFilter;
use crate::normalizer::AdvancedUrlNormalizer;
use crate::queue::UrlQueue;
use crate::scope::ScopeValidator;
use crate::types::{CrawlResult, Finding, FindingStats, Page, Severity};
use hazler_http::HttpClient;
use hazler_js_parser::{FrameFileParser, JavaScriptParser};
use hazler_parser::HtmlParser;
use hazler_secrets::SecretScanner;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::Semaphore;
use tracing::{error, info, warn};
use url::Url;

/// Context for crawling a page, containing all necessary dependencies
struct CrawlPageContext {
    http_client: HttpClient,
    parser: HtmlParser,
    js_parser: JavaScriptParser,
    frame_parser: FrameFileParser,
    url_normalizer: AdvancedUrlNormalizer,
    scope_validator: ScopeValidator,
    max_depth: usize,
    aggressive: bool,
    secret_scanner: Option<SecretScanner>,
    noise_filter: Arc<Mutex<NoiseFilter>>,
}

/// Main crawler implementation
pub struct Crawler {
    config: Config,
    http_client: HttpClient,
    parser: HtmlParser,
    js_parser: JavaScriptParser,
    frame_parser: FrameFileParser,
    url_normalizer: AdvancedUrlNormalizer,
    secret_scanner: Option<SecretScanner>,
}

impl Crawler {
    /// Create a new crawler with the given configuration
    pub fn new(config: Config) -> anyhow::Result<Self> {
        // Create HTTP client with or without stealth mode
        let http_client = if config.stealth_mode {
            HttpClient::new_with_stealth(&config.user_agent, Duration::from_secs(config.timeout_secs))?
        } else {
            HttpClient::new(&config.user_agent, Duration::from_secs(config.timeout_secs))?
        };
        
        let parser = HtmlParser::new();
        let js_parser = JavaScriptParser::new()
            .map_err(|e| anyhow::anyhow!("Failed to create JS parser: {}", e))?;
        let frame_parser = FrameFileParser::new()
            .map_err(|e| anyhow::anyhow!("Failed to create frame parser: {}", e))?;
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
            url_normalizer,
            secret_scanner,
        })
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
                    let url_normalizer = self.url_normalizer.clone();
                    let scope_validator = scope_validator.clone();
                    let max_depth = self.config.max_depth;
                    let aggressive = self.config.aggressive_discovery;
                    let secret_scanner = self.secret_scanner.clone();
                    let noise_filter = Arc::clone(&noise_filter);

                    let task = tokio::spawn(async move {
                        let _permit = semaphore.acquire().await.unwrap();
                        let context = CrawlPageContext {
                            http_client,
                            parser,
                            js_parser,
                            frame_parser,
                            url_normalizer,
                            scope_validator,
                            max_depth,
                            aggressive,
                            secret_scanner,
                            noise_filter,
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

        // Fetch the page
        let response = context.http_client.fetch(&url).await?;

        // Check if this response pattern is noise (WAF blocks, etc.)
        let content_length = response.body.len();
        let should_filter = {
            let mut filter = context.noise_filter.lock().unwrap();
            filter.should_filter(response.status_code, content_length)
        };

        if should_filter {
            warn!(
                "Filtering response from {} (status: {}, length: {}) as noise",
                url, response.status_code, content_length
            );
            // Return early with empty page and no new URLs
            let page = Page::new(url.clone(), response.status_code, String::new(), depth);
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
        } else if content_type.contains("javascript")
            || content_type.contains("application/json")
            || url.path().ends_with(".js")
            || url.path().ends_with(".json")
        {
            // JavaScript or JSON content - use JS parser
            let extracted = context.js_parser.extract_endpoints(&response.body, &url);
            info!(
                "Extracted {} endpoints from JavaScript at {}",
                extracted.len(),
                url
            );
            links = extracted;
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
            let js_endpoints = context.js_parser.extract_endpoints(&response.body, &url);
            if !js_endpoints.is_empty() {
                info!(
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

        // Scan for secrets if enabled
        if let Some(ref scanner) = context.secret_scanner {
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
}
