//! URL mutation engine for generating URL variations

use crate::config::FuzzerConfig;
use regex::Regex;
use std::collections::HashSet;
use url::Url;

/// Type of mutation applied to a URL
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MutationType {
    /// Pluralized path segment
    Pluralization,
    /// Added file extension
    Extension,
    /// Added API version
    Versioning,
    /// Original URL (no mutation)
    Original,
}

/// A mutated URL with metadata
#[derive(Debug, Clone)]
pub struct Mutation {
    /// The mutated URL
    pub url: Url,
    /// Type of mutation applied
    pub mutation_type: MutationType,
    /// Description of the mutation
    pub description: String,
}

/// URL mutation engine
pub struct UrlMutator {
    config: FuzzerConfig,
}

impl UrlMutator {
    /// Create a new URL mutator with the given configuration
    pub fn new(config: FuzzerConfig) -> Self {
        Self { config }
    }

    /// Generate all mutations for a given URL
    pub fn generate_mutations(&self, url: &Url) -> Vec<Mutation> {
        let mut mutations = Vec::new();
        let mut seen = HashSet::new();

        // Add original URL
        if seen.insert(url.to_string()) {
            mutations.push(Mutation {
                url: url.clone(),
                mutation_type: MutationType::Original,
                description: "Original URL".to_string(),
            });
        }

        // Apply pluralization
        if self.config.enable_pluralization {
            mutations.extend(self.pluralize_url(url, &mut seen));
        }

        // Apply extensions
        if self.config.enable_extensions {
            mutations.extend(self.add_extensions(url, &mut seen));
        }

        // Apply versioning
        if self.config.enable_versioning {
            mutations.extend(self.add_versions(url, &mut seen));
        }

        // Limit mutations
        mutations.truncate(self.config.max_mutations);
        mutations
    }

    /// Generate pluralized versions of URL path segments
    fn pluralize_url(&self, url: &Url, seen: &mut HashSet<String>) -> Vec<Mutation> {
        let mut mutations = Vec::new();
        let path = url.path();
        
        // Split path into segments
        let segments: Vec<&str> = path.split('/').collect();
        
        for (i, segment) in segments.iter().enumerate() {
            if segment.is_empty() {
                continue;
            }

            let plural = self.pluralize(segment);
            if plural != *segment {
                let mut new_segments = segments.clone();
                new_segments[i] = &plural;
                let new_path = new_segments.join("/");
                
                if let Ok(mut new_url) = url.clone().join(&new_path) {
                    // Preserve query and fragment
                    new_url.set_query(url.query());
                    new_url.set_fragment(url.fragment());
                    
                    if seen.insert(new_url.to_string()) {
                        mutations.push(Mutation {
                            url: new_url,
                            mutation_type: MutationType::Pluralization,
                            description: format!("Pluralized '{}' to '{}'", segment, plural),
                        });
                    }
                }
            }
        }
        
        mutations
    }

    /// Simple pluralization logic
    fn pluralize(&self, word: &str) -> String {
        if word.is_empty() {
            return word.to_string();
        }

        // Common irregular plurals
        let irregulars = [
            ("person", "people"),
            ("child", "children"),
            ("man", "men"),
            ("woman", "women"),
            ("tooth", "teeth"),
            ("foot", "feet"),
            ("mouse", "mice"),
        ];

        for (singular, plural) in &irregulars {
            if word.eq_ignore_ascii_case(singular) {
                return plural.to_string();
            }
        }

        // Already plural
        if word.ends_with('s') || word.ends_with("es") {
            return word.to_string();
        }

        // Words ending in 'y'
        if word.ends_with('y') && word.len() > 1 {
            let before_y = word.chars().nth(word.len() - 2).unwrap();
            if !"aeiou".contains(before_y) {
                return format!("{}ies", &word[..word.len() - 1]);
            }
        }

        // Words ending in 'o', 'ch', 'sh', 's', 'x', 'z'
        if word.ends_with('o')
            || word.ends_with("ch")
            || word.ends_with("sh")
            || word.ends_with('x')
            || word.ends_with('z')
        {
            return format!("{}es", word);
        }

        // Default: add 's'
        format!("{}s", word)
    }

    /// Add file extensions to the URL
    fn add_extensions(&self, url: &Url, seen: &mut HashSet<String>) -> Vec<Mutation> {
        let mut mutations = Vec::new();
        let path = url.path();

        // Don't add extensions if path already has one
        if path.contains('.') && !path.ends_with('/') {
            return mutations;
        }

        for ext in &self.config.file_extensions {
            let new_path = if path.ends_with('/') {
                format!("{}index.{}", path, ext)
            } else {
                format!("{}.{}", path, ext)
            };

            if let Ok(mut new_url) = Url::parse(&format!("{}{}", url.origin().ascii_serialization(), new_path)) {
                new_url.set_query(url.query());
                new_url.set_fragment(url.fragment());
                
                if seen.insert(new_url.to_string()) {
                    mutations.push(Mutation {
                        url: new_url,
                        mutation_type: MutationType::Extension,
                        description: format!("Added .{} extension", ext),
                    });
                }
            }
        }

        mutations
    }

    /// Add API version prefixes to the URL path
    fn add_versions(&self, url: &Url, seen: &mut HashSet<String>) -> Vec<Mutation> {
        let mut mutations = Vec::new();
        let path = url.path();

        // Check if version already exists
        let version_regex = Regex::new(r"/v\d+/").unwrap();
        if version_regex.is_match(path) {
            return mutations;
        }

        for version in &self.config.api_versions {
            // Add version at different positions
            let positions = vec![
                format!("/{}{}", version, path),                    // /v1/api/users
                format!("/api/{}{}", version, path.trim_start_matches("/api")), // /api/v1/users
            ];

            for new_path in positions {
                if let Ok(mut new_url) = Url::parse(&format!("{}{}", url.origin().ascii_serialization(), new_path)) {
                    new_url.set_query(url.query());
                    new_url.set_fragment(url.fragment());
                    
                    if seen.insert(new_url.to_string()) {
                        mutations.push(Mutation {
                            url: new_url,
                            mutation_type: MutationType::Versioning,
                            description: format!("Added {} version", version),
                        });
                    }
                }
            }
        }

        mutations
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pluralize() {
        let config = FuzzerConfig::default();
        let mutator = UrlMutator::new(config);

        assert_eq!(mutator.pluralize("user"), "users");
        assert_eq!(mutator.pluralize("box"), "boxes");
        assert_eq!(mutator.pluralize("city"), "cities");
        assert_eq!(mutator.pluralize("person"), "people");
        assert_eq!(mutator.pluralize("child"), "children");
    }

    #[test]
    fn test_pluralize_url() {
        let config = FuzzerConfig::default();
        let mutator = UrlMutator::new(config);
        let url = Url::parse("https://api.example.com/user/123").unwrap();
        
        let mutations = mutator.generate_mutations(&url);
        
        // Should contain pluralized version
        let has_users = mutations.iter().any(|m| m.url.path().contains("users"));
        assert!(has_users, "Should generate /users mutation");
    }

    #[test]
    fn test_add_extensions() {
        let config = FuzzerConfig::default();
        let mutator = UrlMutator::new(config);
        let url = Url::parse("https://api.example.com/user").unwrap();
        
        let mutations = mutator.generate_mutations(&url);
        
        // Should contain extension mutations
        let has_json = mutations.iter().any(|m| m.url.path().ends_with(".json"));
        let has_xml = mutations.iter().any(|m| m.url.path().ends_with(".xml"));
        
        assert!(has_json, "Should generate .json mutation");
        assert!(has_xml, "Should generate .xml mutation");
    }

    #[test]
    fn test_add_versions() {
        let config = FuzzerConfig::default();
        let mutator = UrlMutator::new(config);
        let url = Url::parse("https://api.example.com/user").unwrap();
        
        let mutations = mutator.generate_mutations(&url);
        
        // Should contain version mutations
        let has_v1 = mutations.iter().any(|m| m.url.path().contains("/v1/"));
        let has_v2 = mutations.iter().any(|m| m.url.path().contains("/v2/"));
        
        assert!(has_v1, "Should generate /v1/ mutation");
        assert!(has_v2, "Should generate /v2/ mutation");
    }

    #[test]
    fn test_no_duplicate_mutations() {
        let config = FuzzerConfig::default();
        let mutator = UrlMutator::new(config);
        let url = Url::parse("https://api.example.com/user").unwrap();
        
        let mutations = mutator.generate_mutations(&url);
        let unique_urls: HashSet<String> = mutations.iter().map(|m| m.url.to_string()).collect();
        
        assert_eq!(mutations.len(), unique_urls.len(), "Should not generate duplicate mutations");
    }

    #[test]
    fn test_max_mutations_limit() {
        let mut config = FuzzerConfig::default();
        config.max_mutations = 5;
        let mutator = UrlMutator::new(config);
        let url = Url::parse("https://api.example.com/user").unwrap();
        
        let mutations = mutator.generate_mutations(&url);
        
        assert!(mutations.len() <= 5, "Should respect max_mutations limit");
    }
}
