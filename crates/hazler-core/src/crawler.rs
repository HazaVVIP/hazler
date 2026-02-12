use crate::config::Config;
use crate::queue::UrlQueue;
use crate::scope::ScopeValidator;
use crate::types::{CrawlResult, Page};
use hazler_http::HttpClient;
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
}

impl Crawler {
    /// Create a new crawler with the given configuration
    pub fn new(config: Config) -> anyhow::Result<Self> {
        let http_client =
            HttpClient::new(&config.user_agent, Duration::from_secs(config.timeout_secs))?;
        let parser = HtmlParser::new();

        Ok(Self {
            config,
            http_client,
            parser,
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
                    let scope_validator = scope_validator.clone();
                    let max_depth = self.config.max_depth;

                    let task = tokio::spawn(async move {
                        let _permit = semaphore.acquire().await.unwrap();
                        Self::crawl_page(
                            url,
                            depth,
                            http_client,
                            parser,
                            scope_validator,
                            max_depth,
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
        scope_validator: ScopeValidator,
        max_depth: usize,
    ) -> anyhow::Result<(Page, Vec<(Url, usize)>)> {
        info!("Crawling: {} (depth: {})", url, depth);

        // Fetch the page
        let response = http_client.fetch(&url).await?;

        // Only parse HTML content
        let should_parse = response
            .content_type
            .as_ref()
            .map(|ct| ct.contains("text/html"))
            .unwrap_or(false);

        let mut links = Vec::new();
        if should_parse {
            // Extract links
            match parser.extract_links(&response.body, &url) {
                Ok(extracted_links) => {
                    links = extracted_links;
                }
                Err(e) => {
                    warn!("Failed to parse links from {}: {}", url, e);
                }
            }
        }

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
