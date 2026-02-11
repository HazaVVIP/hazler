use crate::error::Result;
use scraper::{Html, Selector};
use url::Url;

/// HTML parser for extracting links and other data
#[derive(Clone)]
pub struct HtmlParser;

impl HtmlParser {
    /// Create a new HTML parser
    pub fn new() -> Self {
        Self
    }

    /// Extract all links from HTML content
    pub fn extract_links(&self, html: &str, base_url: &Url) -> Result<Vec<Url>> {
        let document = Html::parse_document(html);
        
        // Selectors for various link types
        let link_selectors = [
            "a[href]",
            "link[href]",
            "area[href]",
        ];

        let mut links = Vec::new();

        for selector_str in &link_selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                for element in document.select(&selector) {
                    if let Some(href) = element.value().attr("href") {
                        // Skip common non-HTTP(S) schemes and fragments
                        if href.starts_with('#') 
                            || href.starts_with("javascript:")
                            || href.starts_with("mailto:")
                            || href.starts_with("tel:")
                            || href.starts_with("data:")
                        {
                            continue;
                        }

                        // Try to resolve the URL relative to base
                        if let Ok(absolute_url) = base_url.join(href) {
                            // Only include HTTP(S) URLs
                            if absolute_url.scheme() == "http" || absolute_url.scheme() == "https" {
                                links.push(absolute_url);
                            }
                        }
                    }
                }
            }
        }

        // Deduplicate links
        links.sort();
        links.dedup();

        Ok(links)
    }

    /// Extract forms from HTML (for future API discovery)
    pub fn extract_forms(&self, html: &str) -> Result<Vec<FormData>> {
        let document = Html::parse_document(html);
        let mut forms = Vec::new();

        if let Ok(form_selector) = Selector::parse("form") {
            for form in document.select(&form_selector) {
                let action = form.value().attr("action").unwrap_or_default().to_string();
                let method = form.value().attr("method").unwrap_or("get").to_string();
                
                forms.push(FormData { action, method });
            }
        }

        Ok(forms)
    }
}

impl Default for HtmlParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Form data extracted from HTML
#[derive(Debug, Clone)]
pub struct FormData {
    pub action: String,
    pub method: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_links() {
        let parser = HtmlParser::new();
        let html = r###"
            <html>
                <body>
                    <a href="/page1">Link 1</a>
                    <a href="https://example.com/page2">Link 2</a>
                    <a href="#fragment">Fragment</a>
                    <a href="javascript:void(0)">JavaScript</a>
                </body>
            </html>
        "###;
        
        let base_url = Url::parse("https://example.com").unwrap();
        let links = parser.extract_links(html, &base_url).unwrap();
        
        assert_eq!(links.len(), 2);
        assert!(links.iter().any(|u| u.as_str() == "https://example.com/page1"));
        assert!(links.iter().any(|u| u.as_str() == "https://example.com/page2"));
    }

    #[test]
    fn test_extract_forms() {
        let parser = HtmlParser::new();
        let html = r###"
            <html>
                <body>
                    <form action="/submit" method="post">
                        <input name="test" />
                    </form>
                </body>
            </html>
        "###;
        
        let forms = parser.extract_forms(html).unwrap();
        assert_eq!(forms.len(), 1);
        assert_eq!(forms[0].action, "/submit");
        assert_eq!(forms[0].method, "post");
    }
}
