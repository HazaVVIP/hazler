//! HTML Report Generator
//!
//! This module generates comprehensive HTML reports from crawl results.

use hazler_core::{CrawlResult, Severity};
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// Maximum number of endpoints to display in HTML report
const MAX_DISPLAYED_ENDPOINTS: usize = 100;

/// Generate an HTML report from crawl results
pub fn generate_html_report(result: &CrawlResult, output_path: &Path) -> anyhow::Result<()> {
    let html = build_html_report(result);

    let mut file = File::create(output_path)?;
    file.write_all(html.as_bytes())?;

    Ok(())
}

/// Build the HTML report content
fn build_html_report(result: &CrawlResult) -> String {
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");

    // Calculate statistics
    let total_secrets = result.pages.iter().map(|p| p.secrets.len()).sum::<usize>();

    let mut critical_secrets = 0;
    let mut high_secrets = 0;
    let mut medium_secrets = 0;
    let mut low_secrets = 0;

    for page in &result.pages {
        for secret in &page.secrets {
            match secret.severity {
                Severity::Critical => critical_secrets += 1,
                Severity::High => high_secrets += 1,
                Severity::Medium => medium_secrets += 1,
                Severity::Low => low_secrets += 1,
            }
        }
    }

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Hazler Crawl Report</title>
    <style>
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}
        
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
            line-height: 1.6;
            color: #333;
            background: #f5f5f5;
            padding: 20px;
        }}
        
        .container {{
            max-width: 1200px;
            margin: 0 auto;
            background: white;
            padding: 30px;
            border-radius: 8px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }}
        
        h1 {{
            color: #2c3e50;
            border-bottom: 3px solid #3498db;
            padding-bottom: 10px;
            margin-bottom: 30px;
        }}
        
        h2 {{
            color: #34495e;
            margin-top: 30px;
            margin-bottom: 15px;
            padding-bottom: 8px;
            border-bottom: 2px solid #ecf0f1;
        }}
        
        h3 {{
            color: #7f8c8d;
            margin-top: 20px;
            margin-bottom: 10px;
        }}
        
        .header-info {{
            background: #ecf0f1;
            padding: 15px;
            border-radius: 5px;
            margin-bottom: 20px;
        }}
        
        .stats-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 20px;
            margin: 20px 0;
        }}
        
        .stat-card {{
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 20px;
            border-radius: 8px;
            text-align: center;
        }}
        
        .stat-card.success {{
            background: linear-gradient(135deg, #11998e 0%, #38ef7d 100%);
        }}
        
        .stat-card.warning {{
            background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
        }}
        
        .stat-card.danger {{
            background: linear-gradient(135deg, #fa709a 0%, #fee140 100%);
        }}
        
        .stat-value {{
            font-size: 2.5em;
            font-weight: bold;
            margin: 10px 0;
        }}
        
        .stat-label {{
            font-size: 0.9em;
            opacity: 0.9;
        }}
        
        table {{
            width: 100%;
            border-collapse: collapse;
            margin: 20px 0;
        }}
        
        th {{
            background: #3498db;
            color: white;
            padding: 12px;
            text-align: left;
            font-weight: 600;
        }}
        
        td {{
            padding: 10px 12px;
            border-bottom: 1px solid #ecf0f1;
        }}
        
        tr:hover {{
            background: #f8f9fa;
        }}
        
        .url-cell {{
            word-break: break-all;
            font-family: 'Courier New', monospace;
            font-size: 0.9em;
        }}
        
        .status-200 {{ color: #27ae60; font-weight: bold; }}
        .status-300 {{ color: #f39c12; font-weight: bold; }}
        .status-400 {{ color: #e74c3c; font-weight: bold; }}
        .status-500 {{ color: #c0392b; font-weight: bold; }}
        
        .severity-critical {{
            background: #e74c3c;
            color: white;
            padding: 3px 8px;
            border-radius: 3px;
            font-size: 0.85em;
            font-weight: bold;
        }}
        
        .severity-high {{
            background: #e67e22;
            color: white;
            padding: 3px 8px;
            border-radius: 3px;
            font-size: 0.85em;
            font-weight: bold;
        }}
        
        .severity-medium {{
            background: #f39c12;
            color: white;
            padding: 3px 8px;
            border-radius: 3px;
            font-size: 0.85em;
            font-weight: bold;
        }}
        
        .severity-low {{
            background: #95a5a6;
            color: white;
            padding: 3px 8px;
            border-radius: 3px;
            font-size: 0.85em;
            font-weight: bold;
        }}
        
        .secret-card {{
            background: #fff3cd;
            border-left: 4px solid #f39c12;
            padding: 15px;
            margin: 10px 0;
            border-radius: 4px;
        }}
        
        .secret-card.critical {{
            background: #f8d7da;
            border-left-color: #e74c3c;
        }}
        
        .secret-card.high {{
            background: #ffe5d0;
            border-left-color: #e67e22;
        }}
        
        .code-block {{
            background: #2c3e50;
            color: #ecf0f1;
            padding: 15px;
            border-radius: 4px;
            overflow-x: auto;
            font-family: 'Courier New', monospace;
            font-size: 0.9em;
            margin: 10px 0;
        }}
        
        .endpoint-list {{
            list-style: none;
            padding: 0;
        }}
        
        .endpoint-item {{
            padding: 8px 12px;
            margin: 5px 0;
            background: #ecf0f1;
            border-radius: 4px;
            font-family: 'Courier New', monospace;
            font-size: 0.9em;
        }}
        
        .footer {{
            margin-top: 40px;
            padding-top: 20px;
            border-top: 1px solid #ecf0f1;
            text-align: center;
            color: #7f8c8d;
            font-size: 0.9em;
        }}
    </style>
</head>
<body>
    <div class="container">
        <h1>🌐 Hazler Crawl Report</h1>
        
        <div class="header-info">
            <p><strong>Generated:</strong> {}</p>
            <p><strong>Total Pages:</strong> {}</p>
            <p><strong>Total URLs Discovered:</strong> {}</p>
            <p><strong>Errors:</strong> {}</p>
        </div>
        
        <h2>📊 Overview</h2>
        <div class="stats-grid">
            <div class="stat-card success">
                <div class="stat-label">Pages Crawled</div>
                <div class="stat-value">{}</div>
            </div>
            <div class="stat-card">
                <div class="stat-label">URLs Discovered</div>
                <div class="stat-value">{}</div>
            </div>
            <div class="stat-card warning">
                <div class="stat-label">Total Secrets</div>
                <div class="stat-value">{}</div>
            </div>
            <div class="stat-card danger">
                <div class="stat-label">Critical Secrets</div>
                <div class="stat-value">{}</div>
            </div>
        </div>
        
        {}
        
        {}
        
        {}
        
        <div class="footer">
            <p>Generated by <strong>Hazler</strong> - Next-Generation Web Crawler</p>
            <p>Report generated at {}</p>
        </div>
    </div>
</body>
</html>"#,
        timestamp,
        result.total_pages,
        result.total_urls,
        result.errors.len(),
        result.total_pages,
        result.total_urls,
        total_secrets,
        critical_secrets,
        build_secrets_section(
            result,
            critical_secrets,
            high_secrets,
            medium_secrets,
            low_secrets
        ),
        build_pages_section(result),
        build_endpoints_section(result),
        timestamp
    )
}

/// Build the secrets findings section
fn build_secrets_section(
    result: &CrawlResult,
    critical: usize,
    high: usize,
    medium: usize,
    low: usize,
) -> String {
    if critical + high + medium + low == 0 {
        return String::from(
            r#"
        <h2>🔒 Secret Findings</h2>
        <p style="color: #27ae60; font-weight: bold;">✓ No secrets detected</p>
        "#,
        );
    }

    let mut html = format!(
        r#"
        <h2>🔒 Secret Findings</h2>
        <div class="stats-grid">
            <div class="stat-card danger">
                <div class="stat-label">Critical</div>
                <div class="stat-value">{}</div>
            </div>
            <div class="stat-card warning">
                <div class="stat-label">High</div>
                <div class="stat-value">{}</div>
            </div>
            <div class="stat-card">
                <div class="stat-label">Medium</div>
                <div class="stat-value">{}</div>
            </div>
            <div class="stat-card success">
                <div class="stat-label">Low</div>
                <div class="stat-value">{}</div>
            </div>
        </div>
        "#,
        critical, high, medium, low
    );

    // Add detailed findings
    html.push_str("<h3>Detailed Findings</h3>");

    for page in &result.pages {
        if !page.secrets.is_empty() {
            html.push_str(&format!(r#"<h4>📄 {}</h4>"#, page.url));

            for finding in &page.secrets {
                let severity_class = match finding.severity {
                    Severity::Critical => "critical",
                    Severity::High => "high",
                    Severity::Medium => "medium",
                    Severity::Low => "low",
                };

                let severity_label = match finding.severity {
                    Severity::Critical => "CRITICAL",
                    Severity::High => "HIGH",
                    Severity::Medium => "MEDIUM",
                    Severity::Low => "LOW",
                };

                html.push_str(&format!(
                    r#"
                    <div class="secret-card {}">
                        <p><strong>{}</strong> <span class="severity-{}">{}</span></p>
                        <p>{}</p>
                        <p><strong>Location:</strong> Line {}, Column {}</p>
                        <div class="code-block">{}</div>
                    </div>
                    "#,
                    severity_class,
                    finding.secret_type,
                    severity_class,
                    severity_label,
                    finding.description,
                    finding.line,
                    finding.column,
                    html_escape(&finding.context)
                ));
            }
        }
    }

    html
}

/// Build the crawled pages section
fn build_pages_section(result: &CrawlResult) -> String {
    let mut html = String::from(
        r#"
        <h2>📄 Crawled Pages</h2>
        <table>
            <thead>
                <tr>
                    <th>URL</th>
                    <th>Status</th>
                    <th>Depth</th>
                    <th>Links</th>
                    <th>Secrets</th>
                </tr>
            </thead>
            <tbody>
    "#,
    );

    for page in &result.pages {
        let status_class = match page.status_code {
            200..=299 => "status-200",
            300..=399 => "status-300",
            400..=499 => "status-400",
            _ => "status-500",
        };

        html.push_str(&format!(
            r#"
                <tr>
                    <td class="url-cell">{}</td>
                    <td class="{}">{}</td>
                    <td>{}</td>
                    <td>{}</td>
                    <td>{}</td>
                </tr>
            "#,
            html_escape(page.url.as_ref()),
            status_class,
            page.status_code,
            page.depth,
            page.links.len(),
            page.secrets.len()
        ));
    }

    html.push_str("</tbody></table>");
    html
}

/// Build the discovered endpoints section
fn build_endpoints_section(result: &CrawlResult) -> String {
    let mut all_links: Vec<String> = result
        .pages
        .iter()
        .flat_map(|p| p.links.iter())
        .map(|u| u.to_string())
        .collect();

    all_links.sort();
    all_links.dedup();

    let mut html = format!(
        r#"
        <h2>🔗 Discovered Endpoints</h2>
        <p>Total unique endpoints: <strong>{}</strong></p>
        <ul class="endpoint-list">
        "#,
        all_links.len()
    );

    for (i, link) in all_links.iter().enumerate() {
        if i < MAX_DISPLAYED_ENDPOINTS {
            html.push_str(&format!(
                r#"<li class="endpoint-item">{}</li>"#,
                html_escape(link)
            ));
        }
    }

    if all_links.len() > MAX_DISPLAYED_ENDPOINTS {
        html.push_str(&format!(
            r#"<li class="endpoint-item"><em>... and {} more endpoints</em></li>"#,
            all_links.len() - MAX_DISPLAYED_ENDPOINTS
        ));
    }

    html.push_str("</ul>");
    html
}

/// HTML escape utility
fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use hazler_core::{CrawlResult, Page};
    use url::Url;

    #[test]
    fn test_html_escape() {
        assert_eq!(html_escape("<script>"), "&lt;script&gt;");
        assert_eq!(html_escape("foo & bar"), "foo &amp; bar");
    }

    #[test]
    fn test_build_html_report() {
        let mut result = CrawlResult::new();
        let url = Url::parse("https://example.com").unwrap();
        let page = Page::new(url, 200, "test".to_string(), 0);
        result.pages.push(page);
        result.total_pages = 1;
        result.total_urls = 1;

        let html = build_html_report(&result);
        assert!(html.contains("Hazler Crawl Report"));
        assert!(html.contains("example.com"));
    }
}
