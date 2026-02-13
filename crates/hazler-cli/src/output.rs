use hazler_core::{CrawlResult, Page, Severity};
use serde_json::json;
use std::collections::HashMap;
use colored::*;

/// Format crawl results for output
pub struct OutputFormatter {
    exclude_body: bool,
    fields: Option<Vec<String>>,
}

impl OutputFormatter {
    pub fn new(exclude_body: bool, fields: Option<String>) -> Self {
        let fields = fields.map(|f| {
            f.split(',')
                .map(|s| s.trim().to_string())
                .collect::<Vec<_>>()
        });

        Self {
            exclude_body,
            fields,
        }
    }

    /// Filter page data based on options
    fn filter_page(&self, page: &Page) -> serde_json::Value {
        let mut data = json!({
            "url": page.url.as_str(),
            "status_code": page.status_code,
            "depth": page.depth,
        });

        // Threshold for warning about large body content (100KB)
        // TODO: Make this configurable via CLI or env variable
        const LARGE_BODY_THRESHOLD: usize = 100_000;

        if !self.exclude_body {
            // Warn if body is very large
            if page.body.len() > LARGE_BODY_THRESHOLD {
                eprintln!(
                    "Warning: Large body content for {} ({} bytes)",
                    page.url,
                    page.body.len()
                );
            }
            data["body"] = json!(page.body);
        } else {
            // Include body size instead of full body
            data["body_size"] = json!(page.body.len());
        }

        data["headers"] = json!(page.headers);
        data["content_type"] = json!(page.content_type);
        data["links"] = json!(page.links.iter().map(|u| u.as_str()).collect::<Vec<_>>());

        // Include secret findings if any
        if !page.secrets.is_empty() {
            data["secrets"] = json!(page.secrets);
        }

        // If specific fields are requested, filter to only those
        if let Some(ref fields) = self.fields {
            let mut filtered = serde_json::Map::new();
            let obj = data
                .as_object()
                .expect("filter_page: data should always be a JSON object");

            for field in fields {
                if let Some(value) = obj.get(field) {
                    filtered.insert(field.clone(), value.clone());
                }
            }

            return serde_json::Value::Object(filtered);
        }

        data
    }

    /// Format as JSON
    pub fn format_json(&self, result: &CrawlResult) -> Result<String, serde_json::Error> {
        let pages: Vec<_> = result.pages.iter().map(|p| self.filter_page(p)).collect();

        let mut output = json!({
            "pages": pages,
            "total_pages": result.total_pages,
            "total_urls": result.total_urls,
            "errors": result.errors,
        });

        // Include secret findings if any
        if let Some(ref stats) = result.secret_findings {
            output["secret_findings"] = json!(stats);
        }

        serde_json::to_string_pretty(&output)
    }

    /// Format as JSON Lines
    pub fn format_jsonl(&self, result: &CrawlResult) -> Result<Vec<String>, serde_json::Error> {
        result
            .pages
            .iter()
            .map(|page| serde_json::to_string(&self.filter_page(page)))
            .collect()
    }

    /// Format as URL list (one URL per line)
    pub fn format_urls(&self, result: &CrawlResult) -> String {
        result
            .pages
            .iter()
            .map(|p| p.url.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Format as CSV
    pub fn format_csv(&self, result: &CrawlResult) -> String {
        let mut output = String::new();

        // Header
        output.push_str("url,status_code,depth,content_type,num_links\n");

        // Rows
        for page in &result.pages {
            let content_type = page.content_type.as_deref().unwrap_or("");
            let num_links = page.links.len();

            output.push_str(&format!(
                "\"{}\",{},{},\"{}\",{}\n",
                Self::escape_csv(page.url.as_str()),
                page.status_code,
                page.depth,
                Self::escape_csv(content_type),
                num_links
            ));
        }

        output
    }

    /// Format as tree structure
    pub fn format_tree(&self, result: &CrawlResult) -> String {
        let mut output = String::new();

        // Add a nice header
        output.push_str(&format!("\n{}\n", "═".repeat(80).bright_blue()));
        output.push_str(&format!("{}\n", "🌐 HAZLER CRAWL RESULTS".bright_cyan().bold()));
        output.push_str(&format!("{}\n\n", "═".repeat(80).bright_blue()));

        // Group pages by depth
        let mut by_depth: HashMap<usize, Vec<&Page>> = HashMap::new();
        for page in &result.pages {
            by_depth.entry(page.depth).or_default().push(page);
        }

        // Get max depth
        let max_depth = by_depth.keys().max().copied().unwrap_or(0);

        // Print tree with colors
        for depth in 0..=max_depth {
            if let Some(pages) = by_depth.get(&depth) {
                for page in pages {
                    let indent = "  ".repeat(depth);
                    let (status_indicator, status_color) = match page.status_code {
                        200..=299 => ("✓", "green"),
                        300..=399 => ("↻", "yellow"),
                        400..=499 => ("✗", "red"),
                        500..=599 => ("⚠", "bright_red"),
                        _ => ("?", "white"),
                    };

                    let status_str = format!("[{}]", page.status_code);
                    let colored_status = match status_color {
                        "green" => status_str.green(),
                        "yellow" => status_str.yellow(),
                        "red" => status_str.red(),
                        "bright_red" => status_str.bright_red(),
                        _ => status_str.white(),
                    };

                    let indicator_colored = match status_color {
                        "green" => status_indicator.green(),
                        "yellow" => status_indicator.yellow(),
                        "red" => status_indicator.red(),
                        "bright_red" => status_indicator.bright_red(),
                        _ => status_indicator.white(),
                    };

                    // Show secrets indicator if any found
                    let secrets_indicator = if !page.secrets.is_empty() {
                        format!(" 🔒 {} secrets", page.secrets.len()).bright_red().to_string()
                    } else {
                        String::new()
                    };

                    output.push_str(&format!(
                        "{}{} {} {} ({} links){}",
                        indent,
                        indicator_colored,
                        colored_status,
                        page.url.to_string().bright_white(),
                        page.links.len().to_string().cyan(),
                        secrets_indicator
                    ));

                    // Show content type if interesting
                    if let Some(ref ct) = page.content_type {
                        if !ct.starts_with("text/html") {
                            output.push_str(&format!(" [{}]", ct.dimmed()));
                        }
                    }

                    output.push('\n');
                }
            }
        }

        output.push_str(&format!("\n{}\n", "═".repeat(80).bright_blue()));

        output
    }

    /// Escape CSV field
    fn escape_csv(s: &str) -> String {
        s.replace("\"", "\"\"")
    }
}

/// Generate statistics report
pub fn generate_stats(result: &CrawlResult) -> String {
    let mut output = String::new();

    output.push_str(&format!("\n{}\n", "═".repeat(80).bright_blue()));
    output.push_str(&format!("{}\n", "📊 CRAWL STATISTICS".bright_cyan().bold()));
    output.push_str(&format!("{}\n\n", "═".repeat(80).bright_blue()));

    output.push_str(&format!("{} {}\n", "Total pages crawled:".bright_white(), result.total_pages.to_string().green().bold()));
    output.push_str(&format!("{} {}\n", "Total URLs discovered:".bright_white(), result.total_urls.to_string().cyan().bold()));
    output.push_str(&format!("{} {}\n", "Errors encountered:".bright_white(), 
        if result.errors.len() > 0 {
            result.errors.len().to_string().red().bold()
        } else {
            result.errors.len().to_string().green().bold()
        }
    ));

    // Status code distribution
    let mut status_codes: HashMap<u16, usize> = HashMap::new();
    for page in &result.pages {
        *status_codes.entry(page.status_code).or_insert(0) += 1;
    }

    output.push_str(&format!("\n{}\n", "Status Code Distribution:".yellow().bold()));
    let mut codes: Vec<_> = status_codes.iter().collect();
    codes.sort_by_key(|(code, _)| *code);
    for (code, count) in codes {
        let code_str = format!("  {}: {} pages", code, count);
        let colored_str = match *code {
            200..=299 => code_str.green(),
            300..=399 => code_str.yellow(),
            400..=499 => code_str.red(),
            500..=599 => code_str.bright_red(),
            _ => code_str.white(),
        };
        output.push_str(&format!("{}\n", colored_str));
    }

    // Depth distribution
    let mut depths: HashMap<usize, usize> = HashMap::new();
    for page in &result.pages {
        *depths.entry(page.depth).or_insert(0) += 1;
    }

    output.push_str(&format!("\n{}\n", "Depth Distribution:".yellow().bold()));
    let mut depth_list: Vec<_> = depths.iter().collect();
    depth_list.sort_by_key(|(depth, _)| *depth);
    for (depth, count) in depth_list {
        output.push_str(&format!("  {}: {} pages\n", format!("Depth {}", depth).cyan(), count));
    }

    // Content type distribution
    let mut content_types: HashMap<String, usize> = HashMap::new();
    for page in &result.pages {
        let ct = page
            .content_type
            .clone()
            .unwrap_or_else(|| "unknown".to_string());
        *content_types.entry(ct).or_insert(0) += 1;
    }

    output.push_str(&format!("\n{}\n", "Content Type Distribution:".yellow().bold()));
    let mut ct_list: Vec<_> = content_types.iter().collect();
    ct_list.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
    for (ct, count) in ct_list.iter().take(10) {
        output.push_str(&format!("  {}: {} pages\n", ct.cyan(), count));
    }

    output.push_str(&format!("\n{}\n", "═".repeat(80).bright_blue()));

    output
}

/// Generate summary report with issues
pub fn generate_report(result: &CrawlResult) -> String {
    let mut output = String::new();

    output.push_str("=== HAZLER CRAWL REPORT ===\n\n");

    // Summary
    output.push_str(&generate_stats(result));

    // Issues detection
    output.push_str("\n=== Issues Detected ===\n");

    let mut issues_found = false;

    // Check for errors
    if !result.errors.is_empty() {
        issues_found = true;
        output.push_str(&format!(
            "\n⚠️  {} errors encountered:\n",
            result.errors.len()
        ));
        for (i, error) in result.errors.iter().take(10).enumerate() {
            output.push_str(&format!("  {}. {}\n", i + 1, error));
        }
        if result.errors.len() > 10 {
            output.push_str(&format!("  ... and {} more\n", result.errors.len() - 10));
        }
    }

    // Check for 404s
    let not_found: Vec<_> = result
        .pages
        .iter()
        .filter(|p| p.status_code == 404)
        .collect();

    if !not_found.is_empty() {
        issues_found = true;
        output.push_str(&format!(
            "\n⚠️  {} pages returned 404 Not Found:\n",
            not_found.len()
        ));
        for (i, page) in not_found.iter().take(10).enumerate() {
            output.push_str(&format!("  {}. {}\n", i + 1, page.url));
        }
        if not_found.len() > 10 {
            output.push_str(&format!("  ... and {} more\n", not_found.len() - 10));
        }
    }

    // Check for 500s
    let server_errors: Vec<_> = result
        .pages
        .iter()
        .filter(|p| p.status_code >= 500)
        .collect();

    if !server_errors.is_empty() {
        issues_found = true;
        output.push_str(&format!(
            "\n⚠️  {} pages returned server errors (5xx):\n",
            server_errors.len()
        ));
        for (i, page) in server_errors.iter().take(10).enumerate() {
            output.push_str(&format!(
                "  {}. [{}] {}\n",
                i + 1,
                page.status_code,
                page.url
            ));
        }
        if server_errors.len() > 10 {
            output.push_str(&format!("  ... and {} more\n", server_errors.len() - 10));
        }
    }

    // Check for redirects
    let redirects: Vec<_> = result
        .pages
        .iter()
        .filter(|p| p.status_code >= 300 && p.status_code < 400)
        .collect();

    if !redirects.is_empty() {
        output.push_str(&format!(
            "\nℹ️  {} pages returned redirects (3xx):\n",
            redirects.len()
        ));
        for (i, page) in redirects.iter().take(10).enumerate() {
            output.push_str(&format!(
                "  {}. [{}] {}\n",
                i + 1,
                page.status_code,
                page.url
            ));
        }
        if redirects.len() > 10 {
            output.push_str(&format!("  ... and {} more\n", redirects.len() - 10));
        }
    }

    if !issues_found && redirects.is_empty() {
        output.push_str("\n✅ No issues detected!\n");
    }

    // Security Findings Section
    if let Some(ref stats) = result.secret_findings {
        if stats.total > 0 {
            output.push_str("\n=== 🔒 SECURITY FINDINGS ===\n");
            output.push_str(&format!("\nTotal secrets found: {}\n", stats.total));
            
            if stats.critical > 0 {
                output.push_str(&format!("  🔴 Critical: {}\n", stats.critical));
            }
            if stats.high > 0 {
                output.push_str(&format!("  🟠 High: {}\n", stats.high));
            }
            if stats.medium > 0 {
                output.push_str(&format!("  🟡 Medium: {}\n", stats.medium));
            }
            if stats.low > 0 {
                output.push_str(&format!("  🟢 Low: {}\n", stats.low));
            }

            // Show detailed findings by severity
            let mut critical_findings = Vec::new();
            let mut high_findings = Vec::new();

            for page in &result.pages {
                for finding in &page.secrets {
                    match finding.severity {
                        Severity::Critical => critical_findings.push((page, finding)),
                        Severity::High => high_findings.push((page, finding)),
                        _ => {}
                    }
                }
            }

            // Show critical findings
            if !critical_findings.is_empty() {
                output.push_str("\n🔴 CRITICAL Findings:\n");
                for (i, (page, finding)) in critical_findings.iter().take(10).enumerate() {
                    output.push_str(&format!(
                        "  {}. {} at {}\n     Location: line {}, column {}\n     Context: {}\n",
                        i + 1,
                        finding.secret_type,
                        page.url,
                        finding.line,
                        finding.column,
                        truncate_string(&finding.context, 100)
                    ));
                }
                if critical_findings.len() > 10 {
                    output.push_str(&format!("  ... and {} more critical findings\n", critical_findings.len() - 10));
                }
            }

            // Show high severity findings
            if !high_findings.is_empty() {
                output.push_str("\n🟠 HIGH Severity Findings:\n");
                for (i, (page, finding)) in high_findings.iter().take(5).enumerate() {
                    output.push_str(&format!(
                        "  {}. {} at {}\n     Location: line {}, column {}\n",
                        i + 1,
                        finding.secret_type,
                        page.url,
                        finding.line,
                        finding.column
                    ));
                }
                if high_findings.len() > 5 {
                    output.push_str(&format!("  ... and {} more high severity findings\n", high_findings.len() - 5));
                }
            }

            output.push_str("\n⚠️  IMPORTANT: Review and remediate all findings immediately!\n");
        }
    }

    output
}

/// Truncate a string to a maximum length with ellipsis
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}
