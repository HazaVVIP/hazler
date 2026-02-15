# Retry & Persistence Features

This document describes the retry, persistence, and resilience features implemented in Hazler.

## Overview

Hazler now includes a comprehensive suite of features for reliable, resilient web crawling:

- **Smart Retry Logic**: Exponential backoff with jitter for failed requests
- **Circuit Breaker**: Prevents cascading failures for problematic domains
- **Rate Limiting**: Per-domain token bucket rate limiting with adaptive adjustments
- **State Persistence**: Save and resume crawl sessions
- **Graceful Shutdown**: Proper cleanup and state saving on interruption
- **Progress Tracking**: Real-time progress reporting during crawls

## Features

### 1. Smart Retry Logic

Implements intelligent retry mechanisms with exponential backoff:

```bash
# Set maximum retry attempts (default: 3)
hazler https://example.com --max-retries 5
```

**Features:**
- Exponential backoff (delays increase: 1s, 2s, 4s, 8s, etc.)
- Jitter (random variance to prevent thundering herd)
- Max delay cap (prevents excessive wait times)
- Automatic retry for retryable status codes (408, 429, 500, 502, 503, 504)

### 2. Circuit Breaker

Prevents wasting resources on consistently failing domains:

```bash
# Enable circuit breaker
hazler https://example.com --circuit-breaker
```

**How It Works:**
- Tracks failure rates per domain
- Opens circuit after threshold failures (default: 5)
- Blocks requests to open circuits
- Automatically attempts recovery (half-open state)
- Closes circuit after successful requests

### 3. Per-Domain Rate Limiting

Token bucket algorithm for fair rate limiting:

```bash
# Set requests per second per domain (default: 10)
hazler https://example.com --rate-limit 5
```

**Features:**
- Separate rate limits for each domain
- Token bucket algorithm (burst capacity + refill rate)
- Adaptive rate limiting (adjusts based on 429 responses)

### 4. State Persistence

Save and resume crawl sessions:

```bash
# Auto-save state every 60 seconds (default)
hazler https://example.com --auto-save 30

# Resume from saved state
hazler https://example.com --resume hazler-state.json
```

### 5. Graceful Shutdown

Handles Ctrl+C and signals properly.

### 6. Progress Tracking

Real-time progress reporting with crawl statistics.

## Testing

```bash
cargo test --package hazler-core --lib
```

**Test Statistics:** 140+ unit tests

## License

MIT License
