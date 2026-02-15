//! PDF Report Generator
//!
//! This module generates PDF reports from crawl results.

use hazler_core::{CrawlResult, Severity};
use printpdf::*;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

/// Generate a PDF report from crawl results
pub fn generate_pdf_report(result: &CrawlResult, output_path: &Path) -> anyhow::Result<()> {
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");

    // Create PDF document
    let (doc, page1, layer1) =
        PdfDocument::new("Hazler Crawl Report", Mm(210.0), Mm(297.0), "Layer 1");
    let current_layer = doc.get_page(page1).get_layer(layer1);

    // Load fonts
    let font = doc.add_builtin_font(BuiltinFont::Helvetica)?;
    let font_bold = doc.add_builtin_font(BuiltinFont::HelveticaBold)?;

    // Title
    current_layer.use_text("Hazler Crawl Report", 24.0, Mm(20.0), Mm(280.0), &font_bold);

    // Metadata
    let mut y_pos = 260.0;
    current_layer.use_text(
        &format!("Generated: {}", timestamp),
        12.0,
        Mm(20.0),
        Mm(y_pos),
        &font,
    );
    y_pos -= 10.0;

    // Summary statistics
    y_pos -= 10.0;
    current_layer.use_text("Summary Statistics", 16.0, Mm(20.0), Mm(y_pos), &font_bold);
    y_pos -= 10.0;

    current_layer.use_text(
        &format!("Total Pages Crawled: {}", result.total_pages),
        12.0,
        Mm(20.0),
        Mm(y_pos),
        &font,
    );
    y_pos -= 8.0;

    current_layer.use_text(
        &format!("Total URLs Discovered: {}", result.total_urls),
        12.0,
        Mm(20.0),
        Mm(y_pos),
        &font,
    );
    y_pos -= 8.0;

    current_layer.use_text(
        &format!("Errors Encountered: {}", result.errors.len()),
        12.0,
        Mm(20.0),
        Mm(y_pos),
        &font,
    );
    y_pos -= 15.0;

    // Security findings
    let total_secrets = result.pages.iter().map(|p| p.secrets.len()).sum::<usize>();
    if total_secrets > 0 {
        current_layer.use_text("Security Findings", 16.0, Mm(20.0), Mm(y_pos), &font_bold);
        y_pos -= 10.0;

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

        current_layer.use_text(
            &format!("Total Secrets: {}", total_secrets),
            12.0,
            Mm(20.0),
            Mm(y_pos),
            &font,
        );
        y_pos -= 8.0;

        if critical_secrets > 0 {
            current_layer.use_text(
                &format!("Critical: {}", critical_secrets),
                12.0,
                Mm(30.0),
                Mm(y_pos),
                &font,
            );
            y_pos -= 8.0;
        }

        if high_secrets > 0 {
            current_layer.use_text(
                &format!("High: {}", high_secrets),
                12.0,
                Mm(30.0),
                Mm(y_pos),
                &font,
            );
            y_pos -= 8.0;
        }

        if medium_secrets > 0 {
            current_layer.use_text(
                &format!("Medium: {}", medium_secrets),
                12.0,
                Mm(30.0),
                Mm(y_pos),
                &font,
            );
            y_pos -= 8.0;
        }

        if low_secrets > 0 {
            current_layer.use_text(
                &format!("Low: {}", low_secrets),
                12.0,
                Mm(30.0),
                Mm(y_pos),
                &font,
            );
            y_pos -= 8.0;
        }
    }

    // Status code distribution
    y_pos -= 10.0;
    current_layer.use_text(
        "Status Code Distribution",
        16.0,
        Mm(20.0),
        Mm(y_pos),
        &font_bold,
    );
    y_pos -= 10.0;

    let mut status_codes: std::collections::HashMap<u16, usize> = std::collections::HashMap::new();
    for page in &result.pages {
        *status_codes.entry(page.status_code).or_insert(0) += 1;
    }

    let mut codes: Vec<_> = status_codes.iter().collect();
    codes.sort_by_key(|(code, _)| *code);
    for (code, count) in codes.iter().take(10) {
        if y_pos < 30.0 {
            // Add new page if needed
            let (page_idx, layer_idx) = doc.add_page(Mm(210.0), Mm(297.0), "Layer");
            let _current_layer = doc.get_page(page_idx).get_layer(layer_idx);
            y_pos = 280.0;
        }

        current_layer.use_text(
            &format!("Status {}: {} pages", code, count),
            12.0,
            Mm(30.0),
            Mm(y_pos),
            &font,
        );
        y_pos -= 8.0;
    }

    // Top pages
    y_pos -= 10.0;
    if y_pos < 30.0 {
        let (page_idx, layer_idx) = doc.add_page(Mm(210.0), Mm(297.0), "Layer");
        let _current_layer = doc.get_page(page_idx).get_layer(layer_idx);
        y_pos = 280.0;
    }

    current_layer.use_text(
        "Crawled Pages (Top 20)",
        16.0,
        Mm(20.0),
        Mm(y_pos),
        &font_bold,
    );
    y_pos -= 10.0;

    for (i, page) in result.pages.iter().take(20).enumerate() {
        if y_pos < 30.0 {
            let (page_idx, layer_idx) = doc.add_page(Mm(210.0), Mm(297.0), "Layer");
            let _current_layer = doc.get_page(page_idx).get_layer(layer_idx);
            y_pos = 280.0;
        }

        // Truncate URL if too long
        let url_str = page.url.as_str();
        let display_url = if url_str.len() > 60 {
            format!("{}...", &url_str[..60])
        } else {
            url_str.to_string()
        };

        current_layer.use_text(
            &format!("{}. [{}] {}", i + 1, page.status_code, display_url),
            10.0,
            Mm(25.0),
            Mm(y_pos),
            &font,
        );
        y_pos -= 7.0;
    }

    // Save PDF
    let file = File::create(output_path)?;
    let mut writer = BufWriter::new(file);
    doc.save(&mut writer)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use hazler_core::{CrawlResult, Page};
    use url::Url;

    #[test]
    fn test_generate_pdf_report() {
        let mut result = CrawlResult::new();
        let url = Url::parse("https://example.com").unwrap();
        let page = Page::new(url, 200, "test".to_string(), 0);
        result.pages.push(page);
        result.total_pages = 1;
        result.total_urls = 1;

        let output_path = std::path::Path::new("/tmp/test_report.pdf");
        let res = generate_pdf_report(&result, output_path);
        assert!(res.is_ok());

        // Clean up
        let _ = std::fs::remove_file(output_path);
    }
}
