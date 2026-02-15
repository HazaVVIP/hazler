use clap::Parser;
use colored::Colorize;
use hazler_core::{Config, Crawler};
use std::process;
use tracing::{error, info, Level};
use url::Url;

mod output;
use output::{generate_report, generate_stats, OutputFormatter};

mod html_report;
use html_report::generate_html_report;

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

    /// Output format (json, jsonl, urls, csv, tree, nuclei, ffuf, or burp)
    #[arg(short = 'o', long, default_value = "tree")]
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

    /// Generate HTML report and save to file
    /// Creates a comprehensive HTML report with visualizations
    /// Example: --html-report report.html
    #[arg(long, value_name = "FILE")]
    html_report: Option<String>,

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

    /// Enable strict domain mode - only crawl the exact domain (no subdomains)
    /// When enabled, only the exact domain specified in the URL will be crawled
    /// Example: If URL is example.com, sub.example.com will be excluded
    #[arg(long)]
    strict_domain: bool,

    /// Allow subdomains - permits crawling of all subdomains of the target domain
    /// Example: If URL is example.com, also crawl sub.example.com, api.example.com, etc.
    /// Note: This is ignored if --strict-domain is enabled
    #[arg(long)]
    subs: bool,

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

    // Configure the crawler
    let mut config = Config::new()
        .max_depth(max_depth)
        .concurrency(args.concurrency)
        .max_pages(max_pages)
        .user_agent(args.user_agent)
        .timeout_secs(args.timeout)
        .aggressive(aggressive);

    // Apply stealth mode based on flag (defaults to enabled)
    config = config.stealth(enable_stealth);

    // Apply scope control options
    if args.strict_domain {
        config = config.strict_domain(true);
    } else if args.subs {
        config = config.allow_subdomains(true);
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
    config = config.graphql_introspect(args.graphql_introspect);
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
        let mut discovered_urls: Vec<Url> = result
            .pages
            .iter()
            .map(|page| page.url.clone())
            .collect();
        
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
            let mut differ = ResponseDiffer::with_baseline(diff_config.clone(), baseline_path.clone());
            
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
            let mut differ = ResponseDiffer::with_baseline(diff_config.clone(), compare_path.clone());
            
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
                if let Some(similarity) = differ.compare_with_baseline(page.url.as_str(), &page.body) {
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
            eprintln!("Similarity threshold: {:.0}%", diff_config.similarity_threshold * 100.0);
        }

        // Clustering mode (if enabled)
        if args.cluster_responses {
            use hazler_core::{SimHashCalculator, KMeansClusterer, DBSCANClusterer};
            
            eprintln!("\n{}", "Response Clustering".bright_cyan().bold());
            eprintln!("{}", "=".repeat(50));

            let calculator = SimHashCalculator::new();
            let responses: Vec<(String, hazler_core::SimHash)> = result.pages.iter()
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
                eprintln!("\n{} Cluster {} ({} URLs, cohesion: {:.1}%)", 
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

    // Generate HTML report if requested
    if let Some(html_report_path) = &args.html_report {
        match generate_html_report(&result, std::path::Path::new(html_report_path)) {
            Ok(_) => {
                eprintln!(
                    "{} HTML report generated: {}",
                    "✓".green().bold(),
                    html_report_path.bright_cyan()
                );
            }
            Err(e) => {
                error!("Failed to generate HTML report: {}", e);
            }
        }
    }

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
                "nuclei" => match formatter.format_nuclei(&result) {
                    Ok(lines) => {
                        for line in lines {
                            println!("{}", line);
                        }
                    }
                    Err(e) => {
                        error!("Failed to serialize Nuclei results: {}", e);
                        process::exit(1);
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
                _ => {
                    error!(
                        "Unknown output format: {}. Valid formats: json, jsonl, urls, csv, tree, nuclei, ffuf, burp",
                        args.output_format
                    );
                    process::exit(1);
                }
            }

            // Print summary to stderr (unless --stats or --report was used)
            if !args.stats && !args.report {
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
    }
