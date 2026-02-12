use hazler_core::{CrawlResult, Page};
use serde_json::json;
use std::collections::HashMap;

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

        if !self.exclude_body {
            data["body"] = json!(page.body);
        }

        data["headers"] = json!(page.headers);
        data["content_type"] = json!(page.content_type);
        data["links"] = json!(page.links.iter().map(|u| u.as_str()).collect::<Vec<_>>());

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

        let output = json!({
            "pages": pages,
            "total_pages": result.total_pages,
            "total_urls": result.total_urls,
            "errors": result.errors,
        });

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

        // Group pages by depth
        let mut by_depth: HashMap<usize, Vec<&Page>> = HashMap::new();
        for page in &result.pages {
            by_depth.entry(page.depth).or_default().push(page);
        }

        // Get max depth
        let max_depth = by_depth.keys().max().copied().unwrap_or(0);

        // Print tree
        for depth in 0..=max_depth {
            if let Some(pages) = by_depth.get(&depth) {
                for page in pages {
                    let indent = "  ".repeat(depth);
                    let status_indicator = if page.status_code == 200 {
                        "✓"
                    } else {
                        "✗"
                    };

                    output.push_str(&format!(
                        "{}{} [{}] {} ({} links)\n",
                        indent,
                        status_indicator,
                        page.status_code,
                        page.url,
                        page.links.len()
                    ));
                }
            }
        }

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

    output.push_str("=== Crawl Statistics ===\n");
    output.push_str(&format!("Total pages crawled: {}\n", result.total_pages));
    output.push_str(&format!("Total URLs discovered: {}\n", result.total_urls));
    output.push_str(&format!("Errors: {}\n", result.errors.len()));

    // Status code distribution
    let mut status_codes: HashMap<u16, usize> = HashMap::new();
    for page in &result.pages {
        *status_codes.entry(page.status_code).or_insert(0) += 1;
    }

    output.push_str("\n=== Status Code Distribution ===\n");
    let mut codes: Vec<_> = status_codes.iter().collect();
    codes.sort_by_key(|(code, _)| *code);
    for (code, count) in codes {
        output.push_str(&format!("{}: {} pages\n", code, count));
    }

    // Depth distribution
    let mut depths: HashMap<usize, usize> = HashMap::new();
    for page in &result.pages {
        *depths.entry(page.depth).or_insert(0) += 1;
    }

    output.push_str("\n=== Depth Distribution ===\n");
    let mut depth_list: Vec<_> = depths.iter().collect();
    depth_list.sort_by_key(|(depth, _)| *depth);
    for (depth, count) in depth_list {
        output.push_str(&format!("Depth {}: {} pages\n", depth, count));
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

    output.push_str("\n=== Content Type Distribution ===\n");
    let mut ct_list: Vec<_> = content_types.iter().collect();
    ct_list.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
    for (ct, count) in ct_list.iter().take(10) {
        output.push_str(&format!("{}: {} pages\n", ct, count));
    }

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

    output
}
