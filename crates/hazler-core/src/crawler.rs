use crate::config::Config;
use crate::normalizer::AdvancedUrlNormalizer;
use crate::queue::UrlQueue;
use crate::scope::ScopeValidator;
use crate::types::{CrawlResult, Page};
use hazler_http::HttpClient;
use hazler_js_parser::{FrameFileParser, JavaScriptParser};
use hazler_parser::HtmlParser;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tracing::{error, info, warn};
use url::Url;

/// Main crawler implementation
pub struct Crawler {
    config: Config,
    http_client: HttpClient,
    parser: HtmlParser,
    js_parser: JavaScriptParser,
    frame_parser: FrameFileParser,
    url_normalizer: AdvancedUrlNormalizer,
}

impl Crawler {
    /// Create a new crawler with the given configuration
    pub fn new(config: Config) -> anyhow::Result<Self> {
        let http_client =
            HttpClient::new(&config.user_agent, Duration::from_secs(config.timeout_secs))?;
        let parser = HtmlParser::new();
        let js_parser = JavaScriptParser::new()
            .map_err(|e| anyhow::anyhow!("Failed to create JS parser: {}", e))?;
        let frame_parser = FrameFileParser::new()
            .map_err(|e| anyhow::anyhow!("Failed to create frame parser: {}", e))?;
        let url_normalizer = AdvancedUrlNormalizer::new();

        Ok(Self {
            config,
            http_client,
            parser,
            js_parser,
            frame_parser,
            url_normalizer,
        })
    }

    /// Start crawling from a given URL
    pub async fn crawl(&self, start_url: Url) -> anyhow::Result<CrawlResult> {
        info!("Starting crawl from: {}", start_url);

        let mut queue = UrlQueue::new();
        let mut result = CrawlResult::new();
        let scope_validator = ScopeValidator::new(&start_url);

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

                    let task = tokio::spawn(async move {
                        let _permit = semaphore.acquire().await.unwrap();
                        Self::crawl_page(
                            url,
                            depth,
                            http_client,
                            parser,
                            js_parser,
                            frame_parser,
                            url_normalizer,
                            scope_validator,
                            max_depth,
                            aggressive,
                        )
                        .await
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

        Ok(result)
    }

    /// Crawl a single page
    async fn crawl_page(
        url: Url,
        depth: usize,
        http_client: HttpClient,
        parser: HtmlParser,
        js_parser: JavaScriptParser,
        frame_parser: FrameFileParser,
        url_normalizer: AdvancedUrlNormalizer,
        scope_validator: ScopeValidator,
        max_depth: usize,
        aggressive: bool,
    ) -> anyhow::Result<(Page, Vec<(Url, usize)>)> {
        info!("Crawling: {} (depth: {})", url, depth);

        // Fetch the page
        let response = http_client.fetch(&url).await?;

        let content_type = response.content_type.as_deref().unwrap_or("");
        let mut links = Vec::new();

        // Determine which parser to use based on content type and file extension
        if content_type.contains("text/html") {
            // HTML content - use HTML parser
            match parser.extract_links(&response.body, &url) {
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
            let extracted = js_parser.extract_endpoints(&response.body, &url);
            info!(
                "Extracted {} endpoints from JavaScript at {}",
                extracted.len(),
                url
            );
            links = extracted;
        } else if url.path().ends_with(".frame") {
            // Frame file - use frame parser
            let extracted = frame_parser.extract_endpoints(&response.body, &url);
            info!(
                "Extracted {} endpoints from .frame file at {}",
                extracted.len(),
                url
            );
            links = extracted;
        }

        // If aggressive mode is enabled, also try JS parser on HTML content
        if aggressive && content_type.contains("text/html") {
            let js_endpoints = js_parser.extract_endpoints(&response.body, &url);
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
        if aggressive {
            let mut all_links = links.clone();
            for link in &links {
                let variants = url_normalizer.normalize(link);
                all_links.extend(variants);

                // Also try API variations for API-looking URLs
                let api_variants = url_normalizer.generate_api_variations(link);
                all_links.extend(api_variants);
            }
            links = all_links;
        }

        // Deduplicate links using canonicalization
        let mut seen = std::collections::HashSet::new();
        links.retain(|link| {
            let canonical = url_normalizer.canonicalize(link);
            seen.insert(canonical)
        });

        // Create page object
        let mut page = Page::new(url.clone(), response.status_code, response.body, depth);
        page.headers = response.headers;
        page.content_type = response.content_type;
        page.links = links.clone();

        // Filter links by scope and depth
        let new_urls: Vec<(Url, usize)> = links
            .into_iter()
            .filter(|link| scope_validator.is_in_scope(link))
            .filter(|_| depth < max_depth)
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
