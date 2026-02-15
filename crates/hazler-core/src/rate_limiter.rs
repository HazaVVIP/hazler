/// Per-domain rate limiting using token bucket algorithm
/// Implements adaptive rate limiting based on server responses

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{debug, warn};

/// Token bucket for rate limiting
#[derive(Debug, Clone)]
struct TokenBucket {
    /// Current number of tokens
    tokens: f64,
    /// Maximum number of tokens (capacity)
    capacity: f64,
    /// Refill rate (tokens per second)
    refill_rate: f64,
    /// Last refill time
    last_refill: Instant,
}

impl TokenBucket {
    /// Create a new token bucket
    fn new(capacity: f64, refill_rate: f64) -> Self {
        Self {
            tokens: capacity,
            capacity,
            refill_rate,
            last_refill: Instant::now(),
        }
    }

    /// Refill tokens based on elapsed time
    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        let tokens_to_add = elapsed * self.refill_rate;

        self.tokens = (self.tokens + tokens_to_add).min(self.capacity);
        self.last_refill = now;
    }

    /// Try to consume tokens, returns true if successful
    fn try_consume(&mut self, tokens: f64) -> bool {
        self.refill();

        if self.tokens >= tokens {
            self.tokens -= tokens;
            true
        } else {
            false
        }
    }

    /// Get time until tokens are available
    fn time_until_available(&self, tokens: f64) -> Duration {
        if self.tokens >= tokens {
            return Duration::ZERO;
        }

        let tokens_needed = tokens - self.tokens;
        let seconds = tokens_needed / self.refill_rate;
        Duration::from_secs_f64(seconds)
    }

    /// Adjust capacity and refill rate (for adaptive limiting)
    fn adjust(&mut self, new_capacity: f64, new_refill_rate: f64) {
        self.capacity = new_capacity;
        self.refill_rate = new_refill_rate;
        // Don't exceed new capacity
        self.tokens = self.tokens.min(new_capacity);
    }
}

/// Rate limiter configuration
#[derive(Clone, Debug)]
pub struct RateLimiterConfig {
    /// Initial requests per second per domain
    requests_per_second: f64,
    /// Burst capacity (max tokens)
    burst_capacity: f64,
    /// Enable adaptive rate limiting
    adaptive: bool,
    /// Minimum requests per second (for adaptive limiting)
    min_rps: f64,
    /// Maximum requests per second (for adaptive limiting)
    max_rps: f64,
}

impl RateLimiterConfig {
    /// Create a new rate limiter configuration
    pub fn new(requests_per_second: f64) -> Self {
        Self {
            requests_per_second,
            burst_capacity: requests_per_second * 2.0,
            adaptive: true,
            min_rps: 1.0,
            max_rps: 100.0,
        }
    }

    /// Conservative configuration (5 RPS)
    pub fn conservative() -> Self {
        Self::new(5.0)
    }

    /// Default configuration (10 RPS)
    pub fn default_config() -> Self {
        Self::new(10.0)
    }

    /// Aggressive configuration (20 RPS)
    pub fn aggressive() -> Self {
        Self::new(20.0)
    }

    /// Set burst capacity
    pub fn with_burst_capacity(mut self, capacity: f64) -> Self {
        self.burst_capacity = capacity;
        self
    }

    /// Enable or disable adaptive rate limiting
    pub fn with_adaptive(mut self, adaptive: bool) -> Self {
        self.adaptive = adaptive;
        self
    }

    /// Set min/max RPS for adaptive limiting
    pub fn with_adaptive_range(mut self, min_rps: f64, max_rps: f64) -> Self {
        self.min_rps = min_rps;
        self.max_rps = max_rps;
        self
    }
}

impl Default for RateLimiterConfig {
    fn default() -> Self {
        Self::default_config()
    }
}

/// Per-domain rate limiter state
#[derive(Debug)]
struct DomainLimiter {
    bucket: TokenBucket,
    consecutive_429s: u32,
    consecutive_success: u32,
    last_adjustment: Instant,
}

impl DomainLimiter {
    fn new(config: &RateLimiterConfig) -> Self {
        Self {
            bucket: TokenBucket::new(config.burst_capacity, config.requests_per_second),
            consecutive_429s: 0,
            consecutive_success: 0,
            last_adjustment: Instant::now(),
        }
    }
}

/// Per-domain rate limiter using token bucket algorithm
pub struct RateLimiter {
    config: RateLimiterConfig,
    limiters: Arc<Mutex<HashMap<String, DomainLimiter>>>,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(config: RateLimiterConfig) -> Self {
        Self {
            config,
            limiters: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Create a default rate limiter
    pub fn new_default() -> Self {
        Self::new(RateLimiterConfig::default())
    }

    /// Wait for rate limit permission for a domain
    pub async fn wait_for_permit(&self, domain: &str) {
        loop {
            let wait_duration = {
                let mut limiters = self.limiters.lock().unwrap();
                let limiter = limiters
                    .entry(domain.to_string())
                    .or_insert_with(|| DomainLimiter::new(&self.config));

                if limiter.bucket.try_consume(1.0) {
                    return;
                }

                limiter.bucket.time_until_available(1.0)
            };

            if wait_duration > Duration::ZERO {
                debug!(
                    "Rate limit reached for {}, waiting {:?}",
                    domain, wait_duration
                );
                tokio::time::sleep(wait_duration).await;
            }
        }
    }

    /// Record a 429 (Too Many Requests) response for adaptive limiting
    pub fn record_429(&self, domain: &str) {
        if !self.config.adaptive {
            return;
        }

        let mut limiters = self.limiters.lock().unwrap();
        if let Some(limiter) = limiters.get_mut(domain) {
            limiter.consecutive_429s += 1;
            limiter.consecutive_success = 0;

            // Adjust rate limit down after 3 consecutive 429s
            if limiter.consecutive_429s >= 3
                && limiter.last_adjustment.elapsed() > Duration::from_secs(10)
            {
                let current_rate = limiter.bucket.refill_rate;
                let new_rate = (current_rate * 0.5).max(self.config.min_rps);

                warn!(
                    "Adaptive rate limiting: Decreasing rate for {} from {:.1} to {:.1} RPS",
                    domain, current_rate, new_rate
                );

                limiter
                    .bucket
                    .adjust(new_rate * 2.0, new_rate);
                limiter.consecutive_429s = 0;
                limiter.last_adjustment = Instant::now();
            }
        }
    }

    /// Record a successful response for adaptive limiting
    pub fn record_success(&self, domain: &str) {
        if !self.config.adaptive {
            return;
        }

        let mut limiters = self.limiters.lock().unwrap();
        if let Some(limiter) = limiters.get_mut(domain) {
            limiter.consecutive_success += 1;
            limiter.consecutive_429s = 0;

            // Gradually increase rate after sustained success
            if limiter.consecutive_success >= 50
                && limiter.last_adjustment.elapsed() > Duration::from_secs(30)
            {
                let current_rate = limiter.bucket.refill_rate;
                let new_rate = (current_rate * 1.2).min(self.config.max_rps);

                if new_rate > current_rate {
                    debug!(
                        "Adaptive rate limiting: Increasing rate for {} from {:.1} to {:.1} RPS",
                        domain, current_rate, new_rate
                    );

                    limiter
                        .bucket
                        .adjust(new_rate * 2.0, new_rate);
                    limiter.consecutive_success = 0;
                    limiter.last_adjustment = Instant::now();
                }
            }
        }
    }

    /// Get current rate for a domain
    pub fn current_rate(&self, domain: &str) -> Option<f64> {
        let limiters = self.limiters.lock().unwrap();
        limiters.get(domain).map(|l| l.bucket.refill_rate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_bucket_creation() {
        let bucket = TokenBucket::new(10.0, 5.0);
        assert_eq!(bucket.capacity, 10.0);
        assert_eq!(bucket.refill_rate, 5.0);
        assert_eq!(bucket.tokens, 10.0);
    }

    #[test]
    fn test_token_bucket_consume() {
        let mut bucket = TokenBucket::new(10.0, 5.0);
        assert!(bucket.try_consume(5.0));
        assert!((bucket.tokens - 5.0).abs() < 0.01);

        assert!(bucket.try_consume(5.0));
        assert!(bucket.tokens < 0.01);

        assert!(!bucket.try_consume(1.0));
    }

    #[tokio::test]
    async fn test_token_bucket_refill() {
        let mut bucket = TokenBucket::new(10.0, 10.0); // 10 tokens per second
        
        // Consume all tokens
        assert!(bucket.try_consume(10.0));
        assert_eq!(bucket.tokens, 0.0);

        // Wait for refill
        tokio::time::sleep(Duration::from_millis(500)).await;
        
        // Should have refilled approximately 5 tokens
        bucket.refill();
        assert!(bucket.tokens >= 4.5 && bucket.tokens <= 5.5);
    }

    #[test]
    fn test_token_bucket_capacity_cap() {
        let mut bucket = TokenBucket::new(10.0, 5.0);
        bucket.tokens = 10.0;
        bucket.last_refill = Instant::now() - Duration::from_secs(10);

        bucket.refill();
        // Should not exceed capacity
        assert_eq!(bucket.tokens, 10.0);
    }

    #[test]
    fn test_rate_limiter_config() {
        let config = RateLimiterConfig::new(10.0);
        assert_eq!(config.requests_per_second, 10.0);
        assert_eq!(config.burst_capacity, 20.0);
        assert!(config.adaptive);
    }

    #[test]
    fn test_conservative_config() {
        let config = RateLimiterConfig::conservative();
        assert_eq!(config.requests_per_second, 5.0);
    }

    #[test]
    fn test_aggressive_config() {
        let config = RateLimiterConfig::aggressive();
        assert_eq!(config.requests_per_second, 20.0);
    }

    #[tokio::test]
    async fn test_rate_limiter_permits() {
        let limiter = RateLimiter::new(RateLimiterConfig::new(10.0));

        // Should get permits immediately (burst capacity)
        for _ in 0..10 {
            limiter.wait_for_permit("example.com").await;
        }
    }

    #[tokio::test]
    async fn test_rate_limiter_separate_domains() {
        let limiter = RateLimiter::new(RateLimiterConfig::new(10.0));

        // Initialize domains by requesting permits
        limiter.wait_for_permit("example.com").await;
        limiter.wait_for_permit("other.com").await;

        // Record events
        limiter.record_429("example.com");
        limiter.record_success("other.com");

        // Both domains should exist
        assert!(limiter.current_rate("example.com").is_some());
        assert!(limiter.current_rate("other.com").is_some());
    }

    #[test]
    fn test_adaptive_rate_limiting_decrease() {
        let config = RateLimiterConfig::new(10.0).with_adaptive(true);
        let limiter = RateLimiter::new(config);

        // Initialize domain with wait_for_permit to create limiter entry
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            limiter.wait_for_permit("example.com").await;
        });

        let initial_rate = limiter.current_rate("example.com").unwrap();

        // Set last_adjustment to past to allow immediate adjustment
        {
            let mut limiters = limiter.limiters.lock().unwrap();
            if let Some(domain_limiter) = limiters.get_mut("example.com") {
                domain_limiter.last_adjustment = std::time::Instant::now() - std::time::Duration::from_secs(20);
            }
        }

        // Record multiple 429s
        for _ in 0..3 {
            limiter.record_429("example.com");
        }

        let new_rate = limiter.current_rate("example.com").unwrap();
        assert!(new_rate < initial_rate, "Rate should decrease after 429s");
    }

    #[test]
    fn test_token_bucket_adjust() {
        let mut bucket = TokenBucket::new(10.0, 5.0);
        bucket.tokens = 10.0;

        bucket.adjust(20.0, 10.0);
        assert_eq!(bucket.capacity, 20.0);
        assert_eq!(bucket.refill_rate, 10.0);
        assert_eq!(bucket.tokens, 10.0); // Should not exceed current tokens
    }

    #[test]
    fn test_time_until_available() {
        let bucket = TokenBucket::new(10.0, 5.0);
        
        // With full tokens, should be immediate
        let duration = bucket.time_until_available(5.0);
        assert_eq!(duration, Duration::ZERO);

        // With empty bucket
        let mut bucket = TokenBucket::new(10.0, 5.0);
        bucket.tokens = 0.0;
        let duration = bucket.time_until_available(5.0);
        assert!(duration > Duration::ZERO);
    }
}
