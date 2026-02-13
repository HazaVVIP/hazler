use clap::Parser;
use hazler_core::{Config, Crawler};
use std::process;
use tracing::{error, Level};
use url::Url;

mod output;
use output::{generate_report, generate_stats, OutputFormatter};

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

    /// Output format (json, jsonl, urls, csv, or tree)
    #[arg(short = 'o', long, default_value = "jsonl")]
    output_format: String,

    /// Include response body in output (excluded by default for clean output)
    #[arg(long)]
    include_body: bool,

    /// Select specific fields to output (comma-separated: url,status_code,depth,links)
    #[arg(long)]
    fields: Option<String>,

    /// Show crawl statistics
    #[arg(long)]
    stats: bool,

    /// Generate summary report
    #[arg(long)]
    report: bool,

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
    /// - Comprehensive reporting
    #[arg(long)]
    all: bool,

    /// Enable stealth mode to evade WAF detection
    /// - Randomizes request patterns
    /// - Implements adaptive rate limiting
    /// - Maintains session state
    /// - Uses realistic browser headers
    #[arg(long)]
    stealth: bool,

    /// Proxy URL (e.g., socks5://localhost:1080, http://proxy:8080)
    #[arg(long)]
    proxy: Option<String>,
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

    // Parse the URL
    let start_url = match Url::parse(&args.url) {
        Ok(url) => url,
        Err(e) => {
            error!("Invalid URL '{}': {}", args.url, e);
            process::exit(1);
        }
    };

    // Apply --all mode defaults if enabled
    let (max_depth, max_pages, aggressive, enable_secrets, enable_stealth) = if args.all {
        // --all mode: comprehensive scanning
        let depth = if args.max_depth == 3 { 5 } else { args.max_depth }; // Increase depth if using default
        let pages = if args.max_pages == 0 { 0 } else { args.max_pages }; // Keep unlimited or user value
        (depth, pages, true, true, args.stealth) // Force aggressive mode, enable secrets
    } else {
        (args.max_depth, args.max_pages, args.aggressive, false, args.stealth)
    };

    // Configure the crawler
    let mut config = Config::new()
        .max_depth(max_depth)
        .concurrency(args.concurrency)
        .max_pages(max_pages)
        .user_agent(args.user_agent)
        .timeout_secs(args.timeout)
        .aggressive(aggressive);

    // Apply stealth mode if enabled
    if enable_stealth {
        config = config.stealth(true);
    }

    // Apply proxy if provided
    if let Some(proxy_url) = args.proxy {
        config = config.proxy(proxy_url);
    }

    // Apply secrets scanning if enabled
    if enable_secrets {
        config = config.secrets_scanning(true);
    }

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
            // Generate report if requested (to stderr, doesn't interfere with output)
            if args.report {
                eprintln!("{}", generate_report(&result));
            } else if args.stats {
                eprintln!("{}", generate_stats(&result));
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
                    Err(e) => {
                        error!("Failed to serialize results: {}", e);
                        process::exit(1);
                    }
                },
                "urls" => {
                    println!("{}", formatter.format_urls(&result));
                }
                "csv" => {
                    println!("{}", formatter.format_csv(&result));
                }
                "tree" => {
                    println!("{}", formatter.format_tree(&result));
                }
                _ => {
                    error!(
                        "Unknown output format: {}. Valid formats: json, jsonl, urls, csv, tree",
                        args.output_format
                    );
                    process::exit(1);
                }
            }

            // Print summary to stderr (unless --stats or --report was used)
            if !args.stats && !args.report {
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
        }
        Err(e) => {
            error!("Crawl failed: {}", e);
            process::exit(1);
        }
    }
}
