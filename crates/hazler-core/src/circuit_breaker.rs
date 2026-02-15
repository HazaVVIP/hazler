/// Circuit breaker pattern for preventing cascading failures
/// Tracks failure rates and opens circuit when threshold is exceeded
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{debug, warn};

/// Circuit breaker states
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CircuitState {
    /// Circuit is closed, requests flow normally
    Closed,
    /// Circuit is open, requests are rejected immediately
    Open,
    /// Circuit is in half-open state, testing if service recovered
    HalfOpen,
}

/// Circuit breaker configuration
#[derive(Clone, Debug)]
pub struct CircuitBreakerConfig {
    /// Failure threshold to open circuit (number of failures)
    failure_threshold: u32,
    /// Success threshold to close circuit from half-open (number of successes)
    success_threshold: u32,
    /// Duration to wait before attempting half-open state
    timeout: Duration,
    /// Window size for tracking failures
    window_size: u32,
}

impl CircuitBreakerConfig {
    /// Create a new circuit breaker configuration
    pub fn new(failure_threshold: u32, timeout_secs: u64) -> Self {
        Self {
            failure_threshold,
            success_threshold: 2,
            timeout: Duration::from_secs(timeout_secs),
            window_size: 10,
        }
    }

    /// Conservative configuration (5 failures, 30s timeout)
    pub fn conservative() -> Self {
        Self::new(5, 30)
    }

    /// Aggressive configuration (10 failures, 10s timeout)
    pub fn aggressive() -> Self {
        Self::new(10, 10)
    }

    /// Strict configuration (3 failures, 60s timeout)
    pub fn strict() -> Self {
        Self::new(3, 60)
    }

    /// Set success threshold for half-open state
    pub fn with_success_threshold(mut self, threshold: u32) -> Self {
        self.success_threshold = threshold;
        self
    }

    /// Set window size for tracking failures
    pub fn with_window_size(mut self, size: u32) -> Self {
        self.window_size = size;
        self
    }
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self::conservative()
    }
}

/// Internal state of the circuit breaker
#[derive(Debug)]
struct CircuitBreakerState {
    state: CircuitState,
    failure_count: u32,
    success_count: u32,
    last_failure_time: Option<Instant>,
    opened_at: Option<Instant>,
}

impl CircuitBreakerState {
    fn new() -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            success_count: 0,
            last_failure_time: None,
            opened_at: None,
        }
    }
}

/// Circuit breaker for preventing cascading failures
#[derive(Clone)]
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: Arc<Mutex<CircuitBreakerState>>,
    name: String,
}

impl CircuitBreaker {
    /// Create a new circuit breaker
    pub fn new(name: String, config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: Arc::new(Mutex::new(CircuitBreakerState::new())),
            name,
        }
    }

    /// Create a default circuit breaker with given name
    pub fn new_default(name: String) -> Self {
        Self::new(name, CircuitBreakerConfig::default())
    }

    /// Check if the circuit breaker allows the request
    pub fn allow_request(&self) -> bool {
        let mut state = self.state.lock().unwrap();

        match state.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if timeout has elapsed
                if let Some(opened_at) = state.opened_at {
                    if opened_at.elapsed() >= self.config.timeout {
                        // Transition to half-open
                        debug!("Circuit breaker '{}' transitioning to half-open", self.name);
                        state.state = CircuitState::HalfOpen;
                        state.success_count = 0;
                        state.failure_count = 0;
                        return true;
                    }
                }
                false
            }
            CircuitState::HalfOpen => true,
        }
    }

    /// Record a successful operation
    pub fn record_success(&self) {
        let mut state = self.state.lock().unwrap();

        match state.state {
            CircuitState::Closed => {
                // Reset failure count on success
                state.failure_count = 0;
            }
            CircuitState::HalfOpen => {
                state.success_count += 1;
                if state.success_count >= self.config.success_threshold {
                    // Close the circuit
                    debug!(
                        "Circuit breaker '{}' closing after {} successes",
                        self.name, state.success_count
                    );
                    state.state = CircuitState::Closed;
                    state.failure_count = 0;
                    state.success_count = 0;
                    state.opened_at = None;
                }
            }
            CircuitState::Open => {
                // Shouldn't happen, but handle gracefully
            }
        }
    }

    /// Record a failed operation
    pub fn record_failure(&self) {
        let mut state = self.state.lock().unwrap();
        state.last_failure_time = Some(Instant::now());

        match state.state {
            CircuitState::Closed => {
                state.failure_count += 1;
                if state.failure_count >= self.config.failure_threshold {
                    // Open the circuit
                    warn!(
                        "Circuit breaker '{}' opening after {} failures",
                        self.name, state.failure_count
                    );
                    state.state = CircuitState::Open;
                    state.opened_at = Some(Instant::now());
                    state.success_count = 0;
                }
            }
            CircuitState::HalfOpen => {
                // Any failure in half-open immediately opens circuit
                warn!(
                    "Circuit breaker '{}' re-opening due to failure in half-open state",
                    self.name
                );
                state.state = CircuitState::Open;
                state.opened_at = Some(Instant::now());
                state.failure_count = 1;
                state.success_count = 0;
            }
            CircuitState::Open => {
                // Already open, just update counter
                state.failure_count += 1;
            }
        }
    }

    /// Get current state
    pub fn state(&self) -> CircuitState {
        self.state.lock().unwrap().state
    }

    /// Get failure count
    pub fn failure_count(&self) -> u32 {
        self.state.lock().unwrap().failure_count
    }

    /// Reset the circuit breaker to closed state
    pub fn reset(&self) {
        let mut state = self.state.lock().unwrap();
        debug!("Circuit breaker '{}' manually reset", self.name);
        state.state = CircuitState::Closed;
        state.failure_count = 0;
        state.success_count = 0;
        state.opened_at = None;
        state.last_failure_time = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::sleep;

    #[test]
    fn test_circuit_breaker_config() {
        let config = CircuitBreakerConfig::new(5, 30);
        assert_eq!(config.failure_threshold, 5);
        assert_eq!(config.timeout, Duration::from_secs(30));
    }

    #[test]
    fn test_conservative_config() {
        let config = CircuitBreakerConfig::conservative();
        assert_eq!(config.failure_threshold, 5);
        assert_eq!(config.timeout, Duration::from_secs(30));
    }

    #[test]
    fn test_aggressive_config() {
        let config = CircuitBreakerConfig::aggressive();
        assert_eq!(config.failure_threshold, 10);
        assert_eq!(config.timeout, Duration::from_secs(10));
    }

    #[test]
    fn test_strict_config() {
        let config = CircuitBreakerConfig::strict();
        assert_eq!(config.failure_threshold, 3);
        assert_eq!(config.timeout, Duration::from_secs(60));
    }

    #[test]
    fn test_initial_state() {
        let cb = CircuitBreaker::new_default("test".to_string());
        assert_eq!(cb.state(), CircuitState::Closed);
        assert_eq!(cb.failure_count(), 0);
        assert!(cb.allow_request());
    }

    #[test]
    fn test_failures_open_circuit() {
        let config = CircuitBreakerConfig::new(3, 5);
        let cb = CircuitBreaker::new("test".to_string(), config);

        // Record failures
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Closed);
        assert!(cb.allow_request());

        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Closed);
        assert!(cb.allow_request());

        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
        assert!(!cb.allow_request());
    }

    #[test]
    fn test_success_resets_failure_count() {
        let config = CircuitBreakerConfig::new(3, 5);
        let cb = CircuitBreaker::new("test".to_string(), config);

        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.failure_count(), 2);

        cb.record_success();
        assert_eq!(cb.failure_count(), 0);
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_transitions_to_half_open() {
        let config = CircuitBreakerConfig::new(2, 1); // 1 second timeout
        let cb = CircuitBreaker::new("test".to_string(), config);

        // Open the circuit
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
        assert!(!cb.allow_request());

        // Wait for timeout
        sleep(Duration::from_millis(1100)).await;

        // Should transition to half-open
        assert!(cb.allow_request());
        assert_eq!(cb.state(), CircuitState::HalfOpen);
    }

    #[test]
    fn test_half_open_closes_on_success() {
        let config = CircuitBreakerConfig::new(2, 1).with_success_threshold(2);
        let cb = CircuitBreaker::new("test".to_string(), config);

        // Open the circuit
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);

        // Manually transition to half-open
        {
            let mut state = cb.state.lock().unwrap();
            state.state = CircuitState::HalfOpen;
            state.opened_at = Some(Instant::now() - Duration::from_secs(2));
        }

        // Record successes
        cb.record_success();
        assert_eq!(cb.state(), CircuitState::HalfOpen);

        cb.record_success();
        assert_eq!(cb.state(), CircuitState::Closed);
    }

    #[test]
    fn test_half_open_reopens_on_failure() {
        let config = CircuitBreakerConfig::new(2, 1);
        let cb = CircuitBreaker::new("test".to_string(), config);

        // Open the circuit
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);

        // Manually transition to half-open
        {
            let mut state = cb.state.lock().unwrap();
            state.state = CircuitState::HalfOpen;
            state.opened_at = Some(Instant::now() - Duration::from_secs(2));
        }

        // Record failure
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
    }

    #[test]
    fn test_reset() {
        let config = CircuitBreakerConfig::new(2, 5);
        let cb = CircuitBreaker::new("test".to_string(), config);

        // Open the circuit
        cb.record_failure();
        cb.record_failure();
        assert_eq!(cb.state(), CircuitState::Open);
        assert!(!cb.allow_request());

        // Reset
        cb.reset();
        assert_eq!(cb.state(), CircuitState::Closed);
        assert_eq!(cb.failure_count(), 0);
        assert!(cb.allow_request());
    }
}
