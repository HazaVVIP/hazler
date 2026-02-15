//! Export Format Modules
//!
//! This module provides additional export formats: OpenAPI/Swagger and Postman.

use hazler_core::CrawlResult;
use serde_json::json;

/// Export crawl results as OpenAPI/Swagger specification
pub fn format_openapi(result: &CrawlResult) -> String {
    let mut paths = serde_json::Map::new();

    // Group pages by path
    for page in &result.pages {
        let path = page.url.path().to_string();
        let method = "get"; // Default to GET, could be enhanced to detect method

        let path_item = paths.entry(path.clone()).or_insert_with(|| json!({}));

        let operation = json!({
            "summary": format!("Discovered endpoint at {}", page.url),
            "responses": {
                page.status_code.to_string(): {
                    "description": format!("Response with status {}", page.status_code),
                    "content": {
                        page.content_type.as_deref().unwrap_or("text/html"): {
                            "schema": {
                                "type": "string"
                            }
                        }
                    }
                }
            },
            "tags": vec![page.url.host_str().unwrap_or("unknown")]
        });

        if let Some(obj) = path_item.as_object_mut() {
            obj.insert(method.to_string(), operation);
        }
    }

    // Get base URL from first page
    let base_url = result
        .pages
        .first()
        .map(|p| {
            format!(
                "{}://{}",
                p.url.scheme(),
                p.url.host_str().unwrap_or("localhost")
            )
        })
        .unwrap_or_else(|| "http://localhost".to_string());

    let spec = json!({
        "openapi": "3.0.0",
        "info": {
            "title": "Hazler Discovered API",
            "description": "API endpoints discovered by Hazler web crawler",
            "version": "1.0.0",
            "contact": {
                "name": "Hazler",
                "url": "https://github.com/HazaVVIP/hazler"
            }
        },
        "servers": [
            {
                "url": base_url,
                "description": "Discovered server"
            }
        ],
        "paths": paths,
        "components": {
            "schemas": {},
            "securitySchemes": {}
        }
    });

    serde_json::to_string_pretty(&spec).unwrap_or_default()
}

/// Export crawl results as Postman collection
pub fn format_postman(result: &CrawlResult) -> String {
    let mut items = Vec::new();

    for (i, page) in result.pages.iter().enumerate() {
        let item = json!({
            "name": format!("Request {}: {}", i + 1, page.url.path()),
            "request": {
                "method": "GET",
                "header": [],
                "url": {
                    "raw": page.url.as_str(),
                    "protocol": page.url.scheme(),
                    "host": page.url.host_str().unwrap_or("localhost").split('.').collect::<Vec<_>>(),
                    "port": page.url.port().map(|p| p.to_string()).unwrap_or_default(),
                    "path": page.url.path().split('/').filter(|s| !s.is_empty()).collect::<Vec<_>>(),
                    "query": page.url.query().map(|q| {
                        q.split('&').map(|pair| {
                            let parts: Vec<_> = pair.split('=').collect();
                            json!({
                                "key": parts.first().unwrap_or(&""),
                                "value": parts.get(1).unwrap_or(&"")
                            })
                        }).collect::<Vec<_>>()
                    }).unwrap_or_default()
                },
                "description": format!("Status: {}, Depth: {}, Links: {}",
                    page.status_code, page.depth, page.links.len())
            },
            "response": []
        });
        items.push(item);
    }

    let collection = json!({
        "info": {
            "name": "Hazler Crawl Collection",
            "description": "Collection of endpoints discovered by Hazler",
            "schema": "https://schema.getpostman.com/json/collection/v2.1.0/collection.json",
            "_postman_id": uuid::Uuid::new_v4().to_string()
        },
        "item": items,
        "variable": []
    });

    serde_json::to_string_pretty(&collection).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use hazler_core::{CrawlResult, Page};
    use url::Url;

    #[test]
    fn test_format_openapi() {
        let mut result = CrawlResult::new();
        let url = Url::parse("https://example.com/api/users").unwrap();
        let page = Page::new(url, 200, "test".to_string(), 0);
        result.pages.push(page);
        result.total_pages = 1;
        result.total_urls = 1;

        let openapi = format_openapi(&result);
        assert!(openapi.contains("openapi"));
        assert!(openapi.contains("3.0.0"));
        assert!(openapi.contains("/api/users"));
    }

    #[test]
    fn test_format_postman() {
        let mut result = CrawlResult::new();
        let url = Url::parse("https://example.com/api/users").unwrap();
        let page = Page::new(url, 200, "test".to_string(), 0);
        result.pages.push(page);
        result.total_pages = 1;
        result.total_urls = 1;

        let postman = format_postman(&result);
        assert!(postman.contains("Hazler Crawl Collection"));
        assert!(postman.contains("example.com"));
    }
}
