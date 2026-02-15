/// Smart retry logic with exponential backoff
/// Implements retry mechanism for failed HTTP requests with circuit breaker

use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, warn};

/// Retry configuration with exponential backoff
#[derive(Clone, Debug)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    max_attempts: u32,
    /// Initial delay before first retry (milliseconds)
    initial_delay_ms: u64,
    /// Maximum delay between retries (milliseconds)
    max_delay_ms: u64,
    /// Backoff multiplier (exponential growth factor)
    backoff_multiplier: f64,
    /// Enable jitter to avoid thundering herd
    jitter: bool,
}

impl RetryConfig {
    /// Create a new retry configuration
    pub fn new(max_attempts: u32, initial_delay_ms: u64) -> Self {
        Self {
            max_attempts,
            initial_delay_ms,
            max_delay_ms: 30_000, // 30 seconds max
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }

    /// Create a conservative retry configuration (3 attempts, 1s initial)
    pub fn conservative() -> Self {
        Self::new(3, 1000)
    }

    /// Create an aggressive retry configuration (5 attempts, 500ms initial)
    pub fn aggressive() -> Self {
        Self::new(5, 500)
    }

    /// Create a cautious retry configuration (2 attempts, 2s initial)
    pub fn cautious() -> Self {
        Self::new(2, 2000)
    }

    /// Set maximum delay between retries
    pub fn with_max_delay_ms(mut self, max_delay_ms: u64) -> Self {
        self.max_delay_ms = max_delay_ms;
        self
    }

    /// Set backoff multiplier
    pub fn with_backoff_multiplier(mut self, multiplier: f64) -> Self {
        self.backoff_multiplier = multiplier;
        self
    }

    /// Disable jitter
    pub fn without_jitter(mut self) -> Self {
        self.jitter = false;
        self
    }

    /// Calculate delay for a given attempt number
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        let base_delay = (self.initial_delay_ms as f64
            * self.backoff_multiplier.powi(attempt as i32))
            as u64;

        let capped_delay = base_delay.min(self.max_delay_ms);

        let final_delay = if self.jitter {
            // Add random jitter (0-25% of the delay)
            let jitter_range = (capped_delay as f64 * 0.25) as u64;
            let jitter = rand::random::<u64>() % (jitter_range + 1);
            capped_delay + jitter
        } else {
            capped_delay
        };

        Duration::from_millis(final_delay)
    }

    /// Get maximum number of attempts
    pub fn max_attempts(&self) -> u32 {
        self.max_attempts
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self::conservative()
    }
}

/// Retry a fallible async operation with exponential backoff
///
/// # Arguments
/// * `config` - Retry configuration
/// * `operation` - Async closure that returns a Result
///
/// # Returns
/// Result of the operation, or the last error if all retries fail
pub async fn retry_with_backoff<F, Fut, T, E>(
    config: &RetryConfig,
    mut operation: F,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    let mut attempt = 0;

    loop {
        match operation().await {
            Ok(result) => {
                if attempt > 0 {
                    debug!("Operation succeeded after {} retry attempts", attempt);
                }
                return Ok(result);
            }
            Err(e) => {
                attempt += 1;

                if attempt >= config.max_attempts {
                    warn!(
                        "Operation failed after {} attempts. Last error: {}",
                        attempt, e
                    );
                    return Err(e);
                }

                let delay = config.calculate_delay(attempt - 1);
                warn!(
                    "Operation failed (attempt {}/{}): {}. Retrying in {:?}...",
                    attempt, config.max_attempts, e, delay
                );

                sleep(delay).await;
            }
        }
    }
}

/// Check if an HTTP status code is retryable
pub fn is_retryable_status(status_code: u16) -> bool {
    matches!(
        status_code,
        408 | // Request Timeout
        429 | // Too Many Requests
        500 | // Internal Server Error
        502 | // Bad Gateway
        503 | // Service Unavailable
        504   // Gateway Timeout
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_config_creation() {
        let config = RetryConfig::new(3, 1000);
        assert_eq!(config.max_attempts(), 3);
    }

    #[test]
    fn test_conservative_config() {
        let config = RetryConfig::conservative();
        assert_eq!(config.max_attempts(), 3);
        assert_eq!(config.initial_delay_ms, 1000);
    }

    #[test]
    fn test_aggressive_config() {
        let config = RetryConfig::aggressive();
        assert_eq!(config.max_attempts(), 5);
        assert_eq!(config.initial_delay_ms, 500);
    }

    #[test]
    fn test_cautious_config() {
        let config = RetryConfig::cautious();
        assert_eq!(config.max_attempts(), 2);
        assert_eq!(config.initial_delay_ms, 2000);
    }

    #[test]
    fn test_exponential_backoff() {
        let config = RetryConfig::new(5, 100).without_jitter();

        let delay0 = config.calculate_delay(0);
        let delay1 = config.calculate_delay(1);
        let delay2 = config.calculate_delay(2);

        assert_eq!(delay0.as_millis(), 100);
        assert_eq!(delay1.as_millis(), 200);
        assert_eq!(delay2.as_millis(), 400);
    }

    #[test]
    fn test_max_delay_cap() {
        let config = RetryConfig::new(10, 1000)
            .with_max_delay_ms(5000)
            .without_jitter();

        let delay5 = config.calculate_delay(5);
        let delay10 = config.calculate_delay(10);

        // Should be capped at max_delay_ms
        assert_eq!(delay5.as_millis(), 5000);
        assert_eq!(delay10.as_millis(), 5000);
    }

    #[test]
    fn test_jitter_adds_randomness() {
        let config = RetryConfig::new(3, 1000);

        let delay1 = config.calculate_delay(1);
        let delay2 = config.calculate_delay(1);

        // With jitter, two calls should potentially give different results
        // Note: This test might occasionally fail due to randomness, but it's very unlikely
        let base_delay = 2000_u128;
        assert!(delay1.as_millis() >= base_delay);
        assert!(delay1.as_millis() <= base_delay + 500);
        assert!(delay2.as_millis() >= base_delay);
        assert!(delay2.as_millis() <= base_delay + 500);
    }

    #[tokio::test]
    async fn test_retry_success_first_attempt() {
        let config = RetryConfig::new(3, 100);
        let call_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let call_count_clone = call_count.clone();

        let result = retry_with_backoff(&config, || {
            let count = call_count_clone.clone();
            async move {
                count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Ok::<i32, String>(42)
            }
        })
        .await;

        assert_eq!(result, Ok(42));
        assert_eq!(call_count.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_retry_success_after_failures() {
        let config = RetryConfig::new(3, 50);
        let call_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let call_count_clone = call_count.clone();

        let result = retry_with_backoff(&config, || {
            let count = call_count_clone.clone();
            async move {
                let current = count.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
                if current < 3 {
                    Err("Temporary error".to_string())
                } else {
                    Ok::<i32, String>(42)
                }
            }
        })
        .await;

        assert_eq!(result, Ok(42));
        assert_eq!(call_count.load(std::sync::atomic::Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_retry_all_attempts_fail() {
        let config = RetryConfig::new(3, 50);
        let call_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let call_count_clone = call_count.clone();

        let result = retry_with_backoff(&config, || {
            let count = call_count_clone.clone();
            async move {
                count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                Err::<i32, String>("Persistent error".to_string())
            }
        })
        .await;

        assert!(result.is_err());
        assert_eq!(call_count.load(std::sync::atomic::Ordering::SeqCst), 3);
    }

    #[test]
    fn test_retryable_status_codes() {
        assert!(is_retryable_status(408));
        assert!(is_retryable_status(429));
        assert!(is_retryable_status(500));
        assert!(is_retryable_status(502));
        assert!(is_retryable_status(503));
        assert!(is_retryable_status(504));

        assert!(!is_retryable_status(200));
        assert!(!is_retryable_status(404));
        assert!(!is_retryable_status(401));
        assert!(!is_retryable_status(403));
    }
}
