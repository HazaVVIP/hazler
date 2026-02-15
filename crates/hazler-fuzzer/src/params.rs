//! Parameter discovery and fuzzing

use crate::wordlists::Wordlists;
use std::collections::HashMap;
use url::Url;

/// Parameter fuzzing strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FuzzStrategy {
    /// Test one parameter at a time
    Individual,
    /// Test common parameter combinations
    Combinations,
    /// Test all parameters
    Exhaustive,
}

/// Parameter discovery engine
pub struct ParamDiscovery {
    strategy: FuzzStrategy,
}

impl ParamDiscovery {
    /// Create a new parameter discovery instance
    pub fn new(strategy: FuzzStrategy) -> Self {
        Self { strategy }
    }

    /// Generate parameter variations for testing
    pub fn generate_param_urls(&self, base_url: &Url) -> Vec<Url> {
        match self.strategy {
            FuzzStrategy::Individual => self.individual_params(base_url),
            FuzzStrategy::Combinations => self.combined_params(base_url),
            FuzzStrategy::Exhaustive => self.exhaustive_params(base_url),
        }
    }

    /// Test each parameter individually
    fn individual_params(&self, base_url: &Url) -> Vec<Url> {
        let mut urls = Vec::new();
        let params = Wordlists::params();

        for param in params {
            if let Ok(mut url) = base_url.clone().join("") {
                url.query_pairs_mut().append_pair(param, "1");
                urls.push(url);
            }
        }

        urls
    }

    /// Test common parameter combinations
    fn combined_params(&self, base_url: &Url) -> Vec<Url> {
        let mut urls = Vec::new();

        // Common combinations
        let combinations = vec![
            vec!["id", "action"],
            vec!["user_id", "token"],
            vec!["page", "limit"],
            vec!["search", "filter"],
            vec!["sort", "order"],
        ];

        for combo in combinations {
            if let Ok(mut url) = base_url.clone().join("") {
                for param in combo {
                    url.query_pairs_mut().append_pair(param, "1");
                }
                urls.push(url);
            }
        }

        urls
    }

    /// Test all parameters (exhaustive)
    fn exhaustive_params(&self, base_url: &Url) -> Vec<Url> {
        // For exhaustive, return individual params
        // In a real scenario, this could test all combinations
        self.individual_params(base_url)
    }
}

/// Parameter fuzzer for testing parameter values
pub struct ParamFuzzer {
    test_values: HashMap<String, Vec<String>>,
}

impl Default for ParamFuzzer {
    fn default() -> Self {
        Self::new()
    }
}

impl ParamFuzzer {
    /// Create a new parameter fuzzer
    pub fn new() -> Self {
        let mut test_values = HashMap::new();

        // ID-like parameters
        test_values.insert(
            "id".to_string(),
            vec![
                "1".to_string(),
                "2".to_string(),
                "123".to_string(),
                "0".to_string(),
                "-1".to_string(),
            ],
        );

        // Boolean parameters
        test_values.insert(
            "debug".to_string(),
            vec![
                "true".to_string(),
                "false".to_string(),
                "1".to_string(),
                "0".to_string(),
            ],
        );

        // Action parameters
        test_values.insert(
            "action".to_string(),
            vec![
                "view".to_string(),
                "edit".to_string(),
                "delete".to_string(),
                "create".to_string(),
            ],
        );

        Self { test_values }
    }

    /// Get test values for a parameter
    pub fn get_test_values(&self, param_name: &str) -> Vec<String> {
        // Check exact match
        if let Some(values) = self.test_values.get(param_name) {
            return values.clone();
        }

        // Check if parameter name contains known patterns
        if param_name.contains("id") {
            return self.test_values.get("id").unwrap().clone();
        }

        if param_name.contains("debug")
            || param_name.contains("test")
            || param_name.contains("preview")
        {
            return self.test_values.get("debug").unwrap().clone();
        }

        // Default test values
        vec!["1".to_string(), "test".to_string()]
    }

    /// Generate URLs with different parameter values
    pub fn fuzz_param(&self, base_url: &Url, param_name: &str) -> Vec<Url> {
        let mut urls = Vec::new();
        let test_values = self.get_test_values(param_name);

        for value in test_values {
            if let Ok(mut url) = base_url.clone().join("") {
                url.query_pairs_mut().append_pair(param_name, &value);
                urls.push(url);
            }
        }

        urls
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_individual_param_discovery() {
        let discovery = ParamDiscovery::new(FuzzStrategy::Individual);
        let base_url = Url::parse("https://api.example.com/users").unwrap();

        let urls = discovery.generate_param_urls(&base_url);

        assert!(!urls.is_empty(), "Should generate parameter URLs");
        assert!(
            urls.iter().any(|u| u.query().is_some()),
            "URLs should have query parameters"
        );
    }

    #[test]
    fn test_combined_param_discovery() {
        let discovery = ParamDiscovery::new(FuzzStrategy::Combinations);
        let base_url = Url::parse("https://api.example.com/users").unwrap();

        let urls = discovery.generate_param_urls(&base_url);

        assert!(!urls.is_empty(), "Should generate combined parameter URLs");
    }

    #[test]
    fn test_param_fuzzer_id() {
        let fuzzer = ParamFuzzer::new();
        let values = fuzzer.get_test_values("id");

        assert!(!values.is_empty(), "Should have test values for 'id'");
        assert!(values.contains(&"1".to_string()), "Should include '1'");
    }

    #[test]
    fn test_param_fuzzer_debug() {
        let fuzzer = ParamFuzzer::new();
        let values = fuzzer.get_test_values("debug");

        assert!(!values.is_empty(), "Should have test values for 'debug'");
        assert!(
            values.contains(&"true".to_string()),
            "Should include 'true'"
        );
        assert!(
            values.contains(&"false".to_string()),
            "Should include 'false'"
        );
    }

    #[test]
    fn test_fuzz_param_generates_urls() {
        let fuzzer = ParamFuzzer::new();
        let base_url = Url::parse("https://api.example.com/user").unwrap();

        let urls = fuzzer.fuzz_param(&base_url, "id");

        assert!(!urls.is_empty(), "Should generate URLs");
        assert!(
            urls.iter().all(|u| u.query().is_some()),
            "All URLs should have query parameters"
        );
    }
}
