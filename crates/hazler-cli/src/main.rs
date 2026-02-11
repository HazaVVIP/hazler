use clap::Parser;
use hazler_core::{Config, Crawler};
use serde_json;
use std::process;
use tracing::{error, Level};
use tracing_subscriber;
use url::Url;

#[derive(Parser, Debug)]
#[command(name = "hazler")]
#[command(author = "Hazler Team")]
#[command(version = "0.1.0")]
#[command(about = "Next-Generation Intelligent Web Crawler", long_about = None)]
struct Args {
    /// Target URL to crawl
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

    /// Output format (json or jsonl)
    #[arg(short = 'o', long, default_value = "jsonl")]
    output_format: String,

    /// Verbose output
    #[arg(short = 'v', long)]
    verbose: bool,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    // Setup logging
    let log_level = if args.verbose { Level::DEBUG } else { Level::INFO };
    tracing_subscriber::fmt()
        .with_max_level(log_level)
        .with_target(false)
        .init();

    // Parse the URL
    let start_url = match Url::parse(&args.url) {
        Ok(url) => url,
        Err(e) => {
            error!("Invalid URL '{}': {}", args.url, e);
            process::exit(1);
        }
    };

    // Configure the crawler
    let config = Config::new()
        .max_depth(args.max_depth)
        .concurrency(args.concurrency)
        .max_pages(args.max_pages)
        .user_agent(args.user_agent)
        .timeout_secs(args.timeout);

    // Create and run crawler
    let crawler = match Crawler::new(config) {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to create crawler: {}", e);
            process::exit(1);
        }
    };

    match crawler.crawl(start_url).await {
        Ok(result) => {
            // Output results
            match args.output_format.as_str() {
                "json" => {
                    // Output as single JSON object
                    match serde_json::to_string_pretty(&result) {
                        Ok(json) => println!("{}", json),
                        Err(e) => {
                            error!("Failed to serialize results: {}", e);
                            process::exit(1);
                        }
                    }
                }
                "jsonl" => {
                    // Output as JSON Lines (one page per line)
                    for page in &result.pages {
                        match serde_json::to_string(&page) {
                            Ok(json) => println!("{}", json),
                            Err(e) => {
                                error!("Failed to serialize page: {}", e);
                            }
                        }
                    }
                }
                _ => {
                    error!("Unknown output format: {}", args.output_format);
                    process::exit(1);
                }
            }

            // Print summary to stderr
            eprintln!("\n=== Crawl Summary ===");
            eprintln!("Total pages crawled: {}", result.total_pages);
            eprintln!("Total URLs discovered: {}", result.total_urls);
            eprintln!("Errors: {}", result.errors.len());
            
            if !result.errors.is_empty() && args.verbose {
                eprintln!("\n=== Errors ===");
                for error in &result.errors {
                    eprintln!("  - {}", error);
                }
            }
        }
        Err(e) => {
            error!("Crawl failed: {}", e);
            process::exit(1);
        }
    }
}
