//! # Hazler Core
//!
//! Hazler is a next-generation intelligent web crawler built in Rust.
//!
//! This crate provides the core crawling functionality, including:
//! - Concurrent crawling with configurable concurrency
//! - Scope validation to stay within domain boundaries
//! - URL queue management with deduplication
//! - HTML parsing and link extraction
//! - Depth control and page limits
//!
//! ## Quick Start
//!
//! ```no_run
//! use hazler_core::{Config, Crawler};
//! use url::Url;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Configure the crawler
//!     let config = Config::new()
//!         .max_depth(3)
//!         .concurrency(10)
//!         .max_pages(100);
//!     
//!     // Create crawler instance
//!     let crawler = Crawler::new(config)?;
//!     
//!     // Start crawling
//!     let url = Url::parse("https://example.com")?;
//!     let result = crawler.crawl(url).await?;
//!     
//!     println!("Crawled {} pages", result.total_pages);
//!     Ok(())
//! }
//! ```
//!
//! ## Architecture
//!
//! The crawler uses a breadth-first search approach:
//! 1. Start with a seed URL at depth 0
//! 2. Fetch and parse the page
//! 3. Extract links and add them to the queue at depth+1
//! 4. Process queue entries up to max_depth
//! 5. Respect concurrency limits using semaphores
//!
//! ## Configuration
//!
//! Use [`Config`] to customize crawler behavior:
//! - `max_depth`: Maximum crawl depth (default: 3)
//! - `concurrency`: Number of concurrent requests (default: 10)
//! - `max_pages`: Maximum pages to crawl (default: 0 = unlimited)
//! - `timeout_secs`: Request timeout in seconds (default: 10)
//! - `user_agent`: Custom User-Agent string (default: "Hazler/0.1.0")
//!
//! ## Output
//!
//! The crawler returns a [`CrawlResult`] containing:
//! - `pages`: Vector of successfully crawled [`Page`] objects
//! - `total_pages`: Count of pages crawled
//! - `total_urls`: Count of unique URLs discovered
//! - `errors`: List of error messages encountered
//!
//! ## Example: Custom Configuration
//!
//! ```no_run
//! use hazler_core::{Config, Crawler};
//! use url::Url;
//!
//! # #[tokio::main]
//! # async fn main() -> anyhow::Result<()> {
//! let config = Config::new()
//!     .max_depth(5)
//!     .concurrency(20)
//!     .max_pages(1000)
//!     .user_agent("MyBot/1.0".to_string())
//!     .timeout_secs(30);
//!     
//! let crawler = Crawler::new(config)?;
//! let url = Url::parse("https://example.com")?;
//! let result = crawler.crawl(url).await?;
//! # Ok(())
//! # }
//! ```

pub mod config;
pub mod crawler;
pub mod normalizer;
pub mod queue;
pub mod scope;
pub mod types;
pub mod noise_filter;

pub use config::Config;
pub use crawler::Crawler;
pub use normalizer::AdvancedUrlNormalizer;
pub use queue::UrlQueue;
pub use scope::ScopeValidator;
pub use types::{CrawlResult, Finding, FindingStats, Page, Severity};
pub use noise_filter::{NoiseFilter, NoiseFilterStats, ResponsePattern};
