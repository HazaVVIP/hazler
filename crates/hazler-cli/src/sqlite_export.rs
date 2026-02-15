//! SQLite Export Module
//!
//! This module exports crawl results to SQLite database.

use hazler_core::CrawlResult;
use rusqlite::{params, Connection};
use std::path::Path;

/// Export crawl results to SQLite database
pub fn export_to_sqlite(result: &CrawlResult, db_path: &Path) -> anyhow::Result<()> {
    let conn = Connection::open(db_path)?;

    // Create tables
    conn.execute(
        "CREATE TABLE IF NOT EXISTS crawl_metadata (
            id INTEGER PRIMARY KEY,
            timestamp TEXT NOT NULL,
            total_pages INTEGER NOT NULL,
            total_urls INTEGER NOT NULL,
            total_errors INTEGER NOT NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS pages (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            crawl_id INTEGER NOT NULL,
            url TEXT NOT NULL,
            status_code INTEGER NOT NULL,
            depth INTEGER NOT NULL,
            content_type TEXT,
            body_size INTEGER NOT NULL,
            num_links INTEGER NOT NULL,
            num_secrets INTEGER NOT NULL,
            FOREIGN KEY(crawl_id) REFERENCES crawl_metadata(id)
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS links (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            page_id INTEGER NOT NULL,
            link_url TEXT NOT NULL,
            FOREIGN KEY(page_id) REFERENCES pages(id)
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS secrets (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            page_id INTEGER NOT NULL,
            secret_type TEXT NOT NULL,
            severity TEXT NOT NULL,
            description TEXT NOT NULL,
            line INTEGER NOT NULL,
            column INTEGER NOT NULL,
            context TEXT NOT NULL,
            FOREIGN KEY(page_id) REFERENCES pages(id)
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS errors (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            crawl_id INTEGER NOT NULL,
            error_message TEXT NOT NULL,
            FOREIGN KEY(crawl_id) REFERENCES crawl_metadata(id)
        )",
        [],
    )?;

    // Insert crawl metadata
    let timestamp = chrono::Local::now().to_rfc3339();
    conn.execute(
        "INSERT INTO crawl_metadata (timestamp, total_pages, total_urls, total_errors) VALUES (?1, ?2, ?3, ?4)",
        params![timestamp, result.total_pages, result.total_urls, result.errors.len()],
    )?;

    let crawl_id = conn.last_insert_rowid();

    // Insert pages
    for page in &result.pages {
        conn.execute(
            "INSERT INTO pages (crawl_id, url, status_code, depth, content_type, body_size, num_links, num_secrets)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                crawl_id,
                page.url.as_str(),
                page.status_code,
                page.depth,
                page.content_type.as_deref().unwrap_or("unknown"),
                page.body.len(),
                page.links.len(),
                page.secrets.len()
            ],
        )?;

        let page_id = conn.last_insert_rowid();

        // Insert links
        for link in &page.links {
            conn.execute(
                "INSERT INTO links (page_id, link_url) VALUES (?1, ?2)",
                params![page_id, link.as_str()],
            )?;
        }

        // Insert secrets
        for secret in &page.secrets {
            conn.execute(
                "INSERT INTO secrets (page_id, secret_type, severity, description, line, column, context)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    page_id,
                    secret.secret_type,
                    format!("{:?}", secret.severity),
                    secret.description,
                    secret.line,
                    secret.column,
                    secret.context
                ],
            )?;
        }
    }

    // Insert errors
    for error in &result.errors {
        conn.execute(
            "INSERT INTO errors (crawl_id, error_message) VALUES (?1, ?2)",
            params![crawl_id, error],
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use hazler_core::{CrawlResult, Page};
    use url::Url;

    #[test]
    fn test_export_to_sqlite() {
        let mut result = CrawlResult::new();
        let url = Url::parse("https://example.com").unwrap();
        let page = Page::new(url, 200, "test".to_string(), 0);
        result.pages.push(page);
        result.total_pages = 1;
        result.total_urls = 1;

        let db_path = std::path::Path::new("/tmp/test_crawl.db");
        let res = export_to_sqlite(&result, db_path);
        assert!(res.is_ok());

        // Verify data was inserted
        let conn = Connection::open(db_path).unwrap();
        let count: i32 = conn
            .query_row("SELECT COUNT(*) FROM pages", [], |row| row.get(0))
            .unwrap();
        assert_eq!(count, 1);

        // Clean up
        let _ = std::fs::remove_file(db_path);
    }
}
