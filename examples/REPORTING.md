# Reporting & Export Examples

This document provides examples of using Hazler's comprehensive reporting and export features.

## Table of Contents

1. [HTML Reports](#html-reports)
2. [PDF Reports](#pdf-reports)
3. [Database Export](#database-export)
4. [API Specifications](#api-specifications)
5. [Webhook Integrations](#webhook-integrations)
6. [Export Formats](#export-formats)

## HTML Reports

Generate an interactive HTML report with charts, graphs, and tabbed interface:

```bash
# Basic HTML report
hazler https://example.com --html-report report.html

# HTML report with deeper crawl
hazler https://example.com -d 5 --html-report deep-report.html

# HTML report with authentication
hazler https://example.com --auth-basic user:pass --html-report authenticated-report.html
```

The HTML report includes:
- Interactive status code and depth distribution charts
- Tabbed interface (Overview, Security Findings, Pages, Endpoints)
- Sortable tables with filtering capabilities
- Security findings highlighted by severity
- Responsive design for mobile viewing

## PDF Reports

Generate professional PDF reports for documentation:

```bash
# Basic PDF report
hazler https://example.com --pdf-report report.pdf

# PDF report with comprehensive scan
hazler https://example.com --all --pdf-report comprehensive-report.pdf

# Generate both HTML and PDF
hazler https://example.com \
    --html-report report.html \
    --pdf-report report.pdf
```

PDF reports include:
- Summary statistics
- Status code distribution
- Security findings by severity
- Top 20 crawled pages
- Professional formatting

## Database Export

Export crawl results to SQLite database for analysis:

```bash
# Basic SQLite export
hazler https://example.com --export-sqlite crawl.db

# Query the database
sqlite3 crawl.db "SELECT url, status_code FROM pages WHERE status_code = 200;"

# Analyze secrets found
sqlite3 crawl.db "SELECT secret_type, COUNT(*) FROM secrets GROUP BY secret_type;"

# View crawl metadata
sqlite3 crawl.db "SELECT * FROM crawl_metadata;"
```

Database schema includes:
- `crawl_metadata` - Crawl session information
- `pages` - All crawled pages with metadata
- `links` - Discovered links
- `secrets` - Security findings
- `errors` - Errors encountered

## API Specifications

### OpenAPI/Swagger Export

Generate OpenAPI 3.0 specification from discovered endpoints:

```bash
# Export as OpenAPI spec
hazler https://api.example.com --export-openapi swagger.json

# Or output directly
hazler https://api.example.com -o openapi > api-spec.json

# Use with Swagger UI
docker run -p 8080:8080 -e SWAGGER_JSON=/api-spec.json \
    -v $(pwd)/api-spec.json:/api-spec.json swaggerapi/swagger-ui
```

### Postman Collection Export

Generate Postman collection for API testing:

```bash
# Export as Postman collection
hazler https://api.example.com --export-postman collection.json

# Or output directly
hazler https://api.example.com -o postman > api-collection.json

# Import into Postman
# File → Import → collection.json
```

## Webhook Integrations

Send crawl results to team communication tools:

### Slack Integration

```bash
# Send results to Slack
hazler https://example.com \
    --webhook-slack https://hooks.slack.com/services/YOUR/WEBHOOK/URL

# With authentication
export SLACK_WEBHOOK="https://hooks.slack.com/services/YOUR/WEBHOOK/URL"
hazler https://example.com --auth-bearer "token" --webhook-slack "$SLACK_WEBHOOK"
```

Slack messages include:
- Crawl summary (pages, URLs, errors)
- Security findings breakdown
- Professional formatting with emojis

### Discord Integration

```bash
# Send results to Discord
hazler https://example.com \
    --webhook-discord https://discord.com/api/webhooks/YOUR/WEBHOOK/URL

# Multiple webhooks
hazler https://example.com \
    --webhook-slack "$SLACK_URL" \
    --webhook-discord "$DISCORD_URL"
```

Discord messages include:
- Embedded message with color coding
- Crawl statistics
- Security findings
- Timestamp and footer

### Generic Webhook

Send JSON payload to any webhook endpoint:

```bash
# Send to custom webhook
hazler https://example.com \
    --webhook-url https://your-server.com/webhook

# The payload includes:
# - timestamp
# - summary (pages, URLs, errors)
# - security_findings (by severity)
# - pages array with details
```

## Export Formats

### Nuclei Format

Export for use with Nuclei vulnerability scanner:

```bash
# Export as Nuclei JSON
hazler https://example.com -o nuclei > results.jsonl

# Use with Nuclei
nuclei -list results.jsonl -t ~/nuclei-templates/
```

### ffuf Format

Export for use with ffuf web fuzzer:

```bash
# Export as ffuf JSON
hazler https://example.com -o ffuf > ffuf-results.jsonl

# Analyze with jq
cat ffuf-results.jsonl | jq '.status'
```

### Burp Suite Format

Export for import into Burp Suite:

```bash
# Export as Burp XML
hazler https://example.com -o burp > sitemap.xml

# Import into Burp Suite:
# Target → Site map → Right-click → Import → sitemap.xml
```

## Combined Workflows

### Complete Documentation Workflow

```bash
# Generate all reports and exports
hazler https://example.com \
    --html-report report.html \
    --pdf-report report.pdf \
    --export-sqlite crawl.db \
    --export-openapi api-spec.json \
    --export-postman collection.json \
    --webhook-slack "$SLACK_WEBHOOK"
```

### Security Audit Workflow

```bash
# Deep security scan with multiple outputs
hazler https://example.com \
    --all \
    -d 5 \
    --html-report security-audit.html \
    -o nuclei > nuclei-targets.jsonl

# Follow up with Nuclei
nuclei -list nuclei-targets.jsonl -severity critical,high
```

### API Discovery Workflow

```bash
# Crawl API endpoints
hazler https://api.example.com \
    --auth-bearer "YOUR_API_KEY" \
    --export-openapi api-spec.json \
    --export-postman collection.json

# Test with Postman collection
newman run collection.json
```

### Continuous Monitoring

```bash
#!/bin/bash
# monitor.sh - Run periodic crawls and send to Discord

TARGET="https://example.com"
DISCORD_WEBHOOK="https://discord.com/api/webhooks/YOUR/WEBHOOK"

hazler "$TARGET" \
    --export-sqlite "crawl-$(date +%Y%m%d-%H%M%S).db" \
    --webhook-discord "$DISCORD_WEBHOOK"
```

## Advanced Tips

### Multiple Exports in Pipeline

```bash
# Crawl once, export multiple formats
hazler https://example.com -o json | tee results.json | \
    jq -r '.pages[].url' | \
    httpx -silent -status-code
```

### Database Analysis

```bash
# Create a database and analyze it
hazler https://example.com --export-sqlite crawl.db

# Find pages with secrets
sqlite3 crawl.db "
    SELECT p.url, s.secret_type, s.severity
    FROM pages p
    JOIN secrets s ON p.id = s.page_id
    WHERE s.severity IN ('Critical', 'High')
    ORDER BY s.severity;
"

# Status code distribution
sqlite3 crawl.db "
    SELECT status_code, COUNT(*) as count
    FROM pages
    GROUP BY status_code
    ORDER BY count DESC;
"
```

### Webhook Retry Logic

```bash
# Retry webhook if it fails
for i in {1..3}; do
    if hazler https://example.com --webhook-slack "$SLACK_URL"; then
        echo "Success!"
        break
    fi
    echo "Retry $i failed, waiting..."
    sleep 10
done
```

## Environment Variables

You can use environment variables for webhook URLs:

```bash
export HAZLER_SLACK_WEBHOOK="https://hooks.slack.com/services/YOUR/WEBHOOK"
export HAZLER_DISCORD_WEBHOOK="https://discord.com/api/webhooks/YOUR/WEBHOOK"

hazler https://example.com \
    --webhook-slack "$HAZLER_SLACK_WEBHOOK" \
    --webhook-discord "$HAZLER_DISCORD_WEBHOOK"
```

## Troubleshooting

### Large HTML Reports

If HTML reports are slow to render with many pages:

```bash
# Limit pages for faster reports
hazler https://example.com -p 100 --html-report report.html
```

### SQLite Database Locked

If you get "database is locked" errors:

```bash
# Use a different database file
hazler https://example.com --export-sqlite "crawl-$(date +%s).db"
```

### Webhook Failures

Enable verbose logging to debug webhook issues:

```bash
hazler https://example.com \
    --webhook-slack "$SLACK_URL" \
    -v 2>&1 | grep -i webhook
```

## Best Practices

1. **Use descriptive filenames** with timestamps:
   ```bash
   hazler https://example.com \
       --html-report "report-$(date +%Y%m%d).html"
   ```

2. **Combine exports for comprehensive documentation**:
   ```bash
   hazler https://example.com \
       --html-report report.html \
       --export-sqlite crawl.db
   ```

3. **Use webhooks for team notifications**:
   ```bash
   hazler https://production.example.com \
       --webhook-slack "$SLACK_WEBHOOK" \
       --webhook-discord "$DISCORD_WEBHOOK"
   ```

4. **Archive results for historical analysis**:
   ```bash
   mkdir -p reports/$(date +%Y%m)
   hazler https://example.com \
       --export-sqlite "reports/$(date +%Y%m)/crawl-$(date +%Y%m%d).db"
   ```

## See Also

- Main README: `../README.md`
- Authentication Examples: `./README.md` (Authentication section)
- Fuzzing Examples: `./README.md` (Fuzzing section)
