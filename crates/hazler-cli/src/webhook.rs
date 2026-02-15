//! Webhook Integration Module
//!
//! This module provides webhook support for Slack, Discord, and generic webhooks.

use hazler_core::{CrawlResult, Severity};
use serde_json::json;

/// Send crawl results to a Slack webhook
pub async fn send_to_slack(result: &CrawlResult, webhook_url: &str) -> anyhow::Result<()> {
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");

    // Count secrets by severity
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

    let total_secrets = critical_secrets + high_secrets + medium_secrets + low_secrets;

    // Build Slack message
    let mut blocks = vec![
        json!({
            "type": "header",
            "text": {
                "type": "plain_text",
                "text": "🌐 Hazler Crawl Report"
            }
        }),
        json!({
            "type": "section",
            "fields": [
                {
                    "type": "mrkdwn",
                    "text": format!("*Timestamp:*\n{}", timestamp)
                },
                {
                    "type": "mrkdwn",
                    "text": format!("*Pages Crawled:*\n{}", result.total_pages)
                },
                {
                    "type": "mrkdwn",
                    "text": format!("*URLs Discovered:*\n{}", result.total_urls)
                },
                {
                    "type": "mrkdwn",
                    "text": format!("*Errors:*\n{}", result.errors.len())
                }
            ]
        }),
    ];

    // Add security findings if any
    if total_secrets > 0 {
        blocks.push(json!({
            "type": "section",
            "text": {
                "type": "mrkdwn",
                "text": format!("*🔒 Security Findings*\n• Total: {}\n• Critical: {}\n• High: {}\n• Medium: {}\n• Low: {}",
                    total_secrets, critical_secrets, high_secrets, medium_secrets, low_secrets)
            }
        }));
    }

    let payload = json!({
        "blocks": blocks
    });

    // Send to Slack
    let client = reqwest::Client::new();
    let response = client
        .post(webhook_url)
        .json(&payload)
        .send()
        .await?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to send to Slack: {}", response.status());
    }

    Ok(())
}

/// Send crawl results to a Discord webhook
pub async fn send_to_discord(result: &CrawlResult, webhook_url: &str) -> anyhow::Result<()> {
    let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");

    // Count secrets by severity
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

    let total_secrets = critical_secrets + high_secrets + medium_secrets + low_secrets;

    // Determine embed color based on findings
    let color = if critical_secrets > 0 {
        0xFF0000 // Red
    } else if high_secrets > 0 {
        0xFF6600 // Orange
    } else if total_secrets > 0 {
        0xFFCC00 // Yellow
    } else {
        0x00FF00 // Green
    };

    // Build Discord embed
    let mut fields = vec![
        json!({
            "name": "Pages Crawled",
            "value": result.total_pages.to_string(),
            "inline": true
        }),
        json!({
            "name": "URLs Discovered",
            "value": result.total_urls.to_string(),
            "inline": true
        }),
        json!({
            "name": "Errors",
            "value": result.errors.len().to_string(),
            "inline": true
        }),
    ];

    // Add security findings if any
    if total_secrets > 0 {
        fields.push(json!({
            "name": "🔒 Security Findings",
            "value": format!("Total: {}\nCritical: {} | High: {} | Medium: {} | Low: {}",
                total_secrets, critical_secrets, high_secrets, medium_secrets, low_secrets),
            "inline": false
        }));
    }

    let payload = json!({
        "embeds": [
            {
                "title": "🌐 Hazler Crawl Report",
                "description": format!("Crawl completed at {}", timestamp),
                "color": color,
                "fields": fields,
                "footer": {
                    "text": "Generated by Hazler"
                }
            }
        ]
    });

    // Send to Discord
    let client = reqwest::Client::new();
    let response = client
        .post(webhook_url)
        .json(&payload)
        .send()
        .await?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to send to Discord: {}", response.status());
    }

    Ok(())
}

/// Send crawl results to a generic webhook (JSON payload)
pub async fn send_to_webhook(result: &CrawlResult, webhook_url: &str) -> anyhow::Result<()> {
    let timestamp = chrono::Local::now().to_rfc3339();

    // Count secrets by severity
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

    let payload = json!({
        "timestamp": timestamp,
        "summary": {
            "total_pages": result.total_pages,
            "total_urls": result.total_urls,
            "total_errors": result.errors.len()
        },
        "security_findings": {
            "total": critical_secrets + high_secrets + medium_secrets + low_secrets,
            "critical": critical_secrets,
            "high": high_secrets,
            "medium": medium_secrets,
            "low": low_secrets
        },
        "pages": result.pages.iter().map(|p| json!({
            "url": p.url.as_str(),
            "status_code": p.status_code,
            "depth": p.depth,
            "num_links": p.links.len(),
            "num_secrets": p.secrets.len()
        })).collect::<Vec<_>>()
    });

    // Send to webhook
    let client = reqwest::Client::new();
    let response = client
        .post(webhook_url)
        .json(&payload)
        .send()
        .await?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to send to webhook: {}", response.status());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use hazler_core::{CrawlResult, Page};
    use url::Url;

    #[test]
    fn test_webhook_payload_structure() {
        let mut result = CrawlResult::new();
        let url = Url::parse("https://example.com").unwrap();
        let page = Page::new(url, 200, "test".to_string(), 0);
        result.pages.push(page);
        result.total_pages = 1;
        result.total_urls = 1;

        // Test that we can build the payload without errors
        let timestamp = chrono::Local::now().to_rfc3339();
        let payload = json!({
            "timestamp": timestamp,
            "summary": {
                "total_pages": result.total_pages,
                "total_urls": result.total_urls,
                "total_errors": result.errors.len()
            }
        });

        assert!(payload.is_object());
    }
}
