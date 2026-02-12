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
    /// Warning: This may generate more requests
    #[arg(long)]
    aggressive: bool,
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

    // Configure the crawler
    let config = Config::new()
        .max_depth(args.max_depth)
        .concurrency(args.concurrency)
        .max_pages(args.max_pages)
        .user_agent(args.user_agent)
        .timeout_secs(args.timeout)
        .aggressive(args.aggressive);

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
