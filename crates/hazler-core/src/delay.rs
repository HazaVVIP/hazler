/// Request timing randomization for WAF evasion
/// Adds random delays between requests to appear more human-like

use rand::Rng;
use std::time::Duration;

/// Request delay configuration
#[derive(Clone)]
pub struct DelayConfig {
    /// Minimum delay in milliseconds
    min_delay_ms: u64,
    /// Maximum delay in milliseconds
    max_delay_ms: u64,
    /// Enable jitter (additional random delay)
    jitter: bool,
}

impl DelayConfig {
    /// Create a new delay configuration
    pub fn new(min_ms: u64, max_ms: u64) -> Self {
        Self {
            min_delay_ms: min_ms,
            max_delay_ms: max_ms,
            jitter: true,
        }
    }
    
    /// Create a stealth delay configuration (100-500ms with jitter)
    pub fn stealth() -> Self {
        Self::new(100, 500)
    }
    
    /// Create an aggressive delay configuration (50-200ms)
    pub fn aggressive() -> Self {
        Self::new(50, 200)
    }
    
    /// Create a cautious delay configuration (500-2000ms)
    pub fn cautious() -> Self {
        Self::new(500, 2000)
    }
    
    /// Disable jitter
    pub fn without_jitter(mut self) -> Self {
        self.jitter = false;
        self
    }
    
    /// Get a random delay duration
    pub fn get_delay(&self) -> Duration {
        let mut rng = rand::thread_rng();
        let base_delay = rng.gen_range(self.min_delay_ms..=self.max_delay_ms);
        
        let final_delay = if self.jitter {
            // Add up to 20% jitter
            let jitter_amount = (base_delay as f64 * 0.2) as u64;
            let jitter = rng.gen_range(0..=jitter_amount);
            base_delay + jitter
        } else {
            base_delay
        };
        
        Duration::from_millis(final_delay)
    }
}

impl Default for DelayConfig {
    fn default() -> Self {
        Self::stealth()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delay_config() {
        let config = DelayConfig::new(100, 500);
        let delay = config.get_delay();
        assert!(delay.as_millis() >= 100);
        assert!(delay.as_millis() <= 650); // 500 + 20% jitter = 600, with some margin
    }
    
    #[test]
    fn test_stealth_delay() {
        let config = DelayConfig::stealth();
        let delay = config.get_delay();
        assert!(delay.as_millis() >= 100);
        assert!(delay.as_millis() <= 650);
    }
    
    #[test]
    fn test_aggressive_delay() {
        let config = DelayConfig::aggressive();
        let delay = config.get_delay();
        assert!(delay.as_millis() >= 50);
        assert!(delay.as_millis() <= 260);
    }
    
    #[test]
    fn test_cautious_delay() {
        let config = DelayConfig::cautious();
        let delay = config.get_delay();
        assert!(delay.as_millis() >= 500);
        assert!(delay.as_millis() <= 2500);
    }
    
    #[test]
    fn test_without_jitter() {
        let config = DelayConfig::new(100, 500).without_jitter();
        let delay = config.get_delay();
        assert!(delay.as_millis() >= 100);
        assert!(delay.as_millis() <= 500); // No jitter
    }
}
