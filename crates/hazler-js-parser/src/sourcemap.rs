use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use url::Url;

/// Source map parser for extracting original source paths
#[derive(Clone)]
pub struct SourceMapParser {
    /// Maximum source map size to process (in bytes)
    max_size: usize,
}

/// Parsed source map information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceMap {
    pub version: i32,
    pub file: Option<String>,
    pub sources: Vec<String>,
    pub sources_content: Option<Vec<Option<String>>>,
    pub names: Vec<String>,
    pub mappings: String,
    pub source_root: Option<String>,
}

/// Source map detection result
#[derive(Debug, Clone)]
pub struct SourceMapReference {
    pub js_url: Url,
    pub map_url: Url,
    pub inline: bool,
}

/// Analyzed source map with extracted paths
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceMapAnalysis {
    pub map_url: String,
    pub total_sources: usize,
    pub interesting_paths: Vec<InterestingPath>,
    pub frameworks_detected: Vec<String>,
    pub project_structure: Vec<String>,
}

/// Interesting path found in source map
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterestingPath {
    pub path: String,
    pub category: PathCategory,
    pub priority: Priority,
    pub reason: String,
}

/// Category of an interesting path
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PathCategory {
    Admin,
    Api,
    Auth,
    Config,
    Database,
    Internal,
    Secret,
    Test,
    Other,
}

/// Priority level for interesting paths
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Critical,
    High,
    Medium,
    Low,
}

impl SourceMapParser {
    /// Create a new source map parser
    pub fn new() -> Self {
        Self {
            max_size: 50 * 1024 * 1024, // 50 MB default
        }
    }

    /// Create a new source map parser with custom max size
    pub fn with_max_size(max_size: usize) -> Self {
        Self { max_size }
    }

    /// Detect source map references in JavaScript content
    pub fn detect_source_map_references(
        &self,
        js_content: &str,
        js_url: &Url,
    ) -> Vec<SourceMapReference> {
        let mut references = Vec::new();

        // Check for sourceMappingURL comment at the end of the file
        let lines: Vec<&str> = js_content.lines().collect();
        if let Some(last_line) = lines.last() {
            if let Some(map_url_str) = self.extract_mapping_url(last_line) {
                if let Ok(map_url) = js_url.join(map_url_str) {
                    references.push(SourceMapReference {
                        js_url: js_url.clone(),
                        map_url,
                        inline: map_url_str.starts_with("data:"),
                    });
                }
            }
        }

        // Also check second to last line (some bundlers put it there)
        if lines.len() > 1 {
            if let Some(second_last) = lines.get(lines.len() - 2) {
                if let Some(map_url_str) = self.extract_mapping_url(second_last) {
                    if let Ok(map_url) = js_url.join(map_url_str) {
                        if !references.iter().any(|r| r.map_url == map_url) {
                            references.push(SourceMapReference {
                                js_url: js_url.clone(),
                                map_url,
                                inline: map_url_str.starts_with("data:"),
                            });
                        }
                    }
                }
            }
        }

        // Try adding .map extension as fallback
        if let Ok(map_url) = Url::parse(&format!("{}.map", js_url.as_str())) {
            if !references.iter().any(|r| r.map_url == map_url) {
                references.push(SourceMapReference {
                    js_url: js_url.clone(),
                    map_url,
                    inline: false,
                });
            }
        }

        references
    }

    /// Extract sourceMappingURL from a line
    fn extract_mapping_url<'a>(&self, line: &'a str) -> Option<&'a str> {
        let trimmed = line.trim();

        // Handle //#sourceMappingURL=
        if let Some(idx) = trimmed.find("sourceMappingURL=") {
            let url_start = idx + "sourceMappingURL=".len();
            return Some(trimmed[url_start..].trim());
        }

        // Handle //@ sourceMappingURL= (deprecated but still used)
        if let Some(idx) = trimmed.find("@ sourceMappingURL=") {
            let url_start = idx + "@ sourceMappingURL=".len();
            return Some(trimmed[url_start..].trim());
        }

        None
    }

    /// Parse source map from JSON content
    pub fn parse_source_map(&self, content: &str) -> Result<SourceMap> {
        // Check size limit
        if content.len() > self.max_size {
            return Err(crate::error::Error::SourceMapTooLarge(content.len()));
        }

        let source_map: SourceMap = serde_json::from_str(content)?;
        Ok(source_map)
    }

    /// Analyze source map and extract interesting information
    pub fn analyze_source_map(&self, source_map: &SourceMap, map_url: &str) -> SourceMapAnalysis {
        let mut interesting_paths = Vec::new();
        let mut frameworks = HashSet::new();
        let mut project_dirs = HashSet::new();

        for source in &source_map.sources {
            // Detect frameworks
            if source.contains("node_modules") {
                self.detect_framework_from_path(source, &mut frameworks);
            }

            // Extract project structure (non node_modules paths)
            if !source.contains("node_modules") && !source.contains("webpack") {
                if let Some(dir) = self.extract_directory(source) {
                    project_dirs.insert(dir);
                }
            }

            // Check for interesting paths
            if let Some(interesting) = self.classify_path(source) {
                interesting_paths.push(interesting);
            }
        }

        // Sort by priority
        interesting_paths.sort_by(|a, b| a.priority.cmp(&b.priority));

        SourceMapAnalysis {
            map_url: map_url.to_string(),
            total_sources: source_map.sources.len(),
            interesting_paths,
            frameworks_detected: frameworks.into_iter().collect(),
            project_structure: project_dirs.into_iter().collect(),
        }
    }

    /// Detect framework from node_modules path
    fn detect_framework_from_path(&self, path: &str, frameworks: &mut HashSet<String>) {
        let common_frameworks = [
            "react", "vue", "angular", "@angular", "svelte", "next", "nuxt", "gatsby", "express",
            "fastify", "nest", "redux", "mobx", "axios", "apollo", "graphql",
        ];

        for framework in &common_frameworks {
            if path.contains(&format!("node_modules/{}", framework)) {
                frameworks.insert(framework.to_string());
            }
        }
    }

    /// Extract directory from file path
    fn extract_directory(&self, path: &str) -> Option<String> {
        let normalized = path.replace('\\', "/");
        let parts: Vec<&str> = normalized.split('/').filter(|p| !p.is_empty()).collect();

        if parts.is_empty() {
            return None;
        }

        // Skip webpack:// and other protocols
        let start_idx = if parts[0].ends_with(':') { 1 } else { 0 };

        if start_idx < parts.len() {
            // Return the first non-empty, meaningful directory
            Some(parts[start_idx].to_string())
        } else {
            None
        }
    }

    /// Classify path by category and priority
    fn classify_path(&self, path: &str) -> Option<InterestingPath> {
        let lower_path = path.to_lowercase();

        // Skip node_modules and webpack internals
        if lower_path.contains("node_modules") || lower_path.contains("webpack/runtime") {
            return None;
        }

        // Critical paths
        if lower_path.contains("admin") {
            return Some(InterestingPath {
                path: path.to_string(),
                category: PathCategory::Admin,
                priority: Priority::Critical,
                reason: "Admin panel component detected".to_string(),
            });
        }

        if lower_path.contains("secret")
            || lower_path.contains("credential")
            || lower_path.contains("password")
            || lower_path.contains(".env")
        {
            return Some(InterestingPath {
                path: path.to_string(),
                category: PathCategory::Secret,
                priority: Priority::Critical,
                reason: "Potential secret or credential reference".to_string(),
            });
        }

        // High priority paths
        if lower_path.contains("/api/")
            || lower_path.contains("_api")
            || lower_path.contains("api.")
        {
            return Some(InterestingPath {
                path: path.to_string(),
                category: PathCategory::Api,
                priority: Priority::High,
                reason: "API implementation or routes".to_string(),
            });
        }

        if lower_path.contains("auth")
            || lower_path.contains("login")
            || lower_path.contains("session")
        {
            return Some(InterestingPath {
                path: path.to_string(),
                category: PathCategory::Auth,
                priority: Priority::High,
                reason: "Authentication logic detected".to_string(),
            });
        }

        if lower_path.contains("config") || lower_path.contains("settings") {
            return Some(InterestingPath {
                path: path.to_string(),
                category: PathCategory::Config,
                priority: Priority::High,
                reason: "Configuration file detected".to_string(),
            });
        }

        // Medium priority paths
        if lower_path.contains("internal") || lower_path.contains("private") {
            return Some(InterestingPath {
                path: path.to_string(),
                category: PathCategory::Internal,
                priority: Priority::Medium,
                reason: "Internal/private component".to_string(),
            });
        }

        if lower_path.contains("database")
            || lower_path.contains("db")
            || lower_path.contains("model")
        {
            return Some(InterestingPath {
                path: path.to_string(),
                category: PathCategory::Database,
                priority: Priority::Medium,
                reason: "Database or model definition".to_string(),
            });
        }

        // Low priority but still interesting
        if lower_path.contains("test")
            || lower_path.contains("spec")
            || lower_path.contains("__test__")
        {
            return Some(InterestingPath {
                path: path.to_string(),
                category: PathCategory::Test,
                priority: Priority::Low,
                reason: "Test file (may reveal endpoints)".to_string(),
            });
        }

        None
    }

    /// Generate a summary report from analysis
    pub fn generate_report(&self, analysis: &SourceMapAnalysis) -> String {
        let mut report = String::new();

        report.push_str(&format!(
            "\n[INFO] Source Map Analysis: {}\n",
            analysis.map_url
        ));
        report.push_str(&format!(
            "[INFO] Total sources: {}\n",
            analysis.total_sources
        ));

        if !analysis.frameworks_detected.is_empty() {
            report.push_str(&format!(
                "[INFO] Frameworks detected: {}\n",
                analysis.frameworks_detected.join(", ")
            ));
        }

        if !analysis.project_structure.is_empty() {
            report.push_str(&format!(
                "[INFO] Project directories: {}\n",
                analysis.project_structure.join(", ")
            ));
        }

        if !analysis.interesting_paths.is_empty() {
            report.push_str(&format!(
                "\n[HIGH] Found {} interesting paths:\n",
                analysis.interesting_paths.len()
            ));

            for path_info in &analysis.interesting_paths {
                let priority_label = match path_info.priority {
                    Priority::Critical => "CRITICAL",
                    Priority::High => "HIGH",
                    Priority::Medium => "MEDIUM",
                    Priority::Low => "LOW",
                };

                report.push_str(&format!(
                    "  [{}] {} - {}\n",
                    priority_label, path_info.path, path_info.reason
                ));
            }
        }

        report
    }
}

impl Default for SourceMapParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_source_map_references() {
        let parser = SourceMapParser::new();
        let js_content = r#"
        console.log("Hello");
        //# sourceMappingURL=app.js.map
        "#;
        let js_url = Url::parse("https://example.com/static/app.js").unwrap();

        let refs = parser.detect_source_map_references(js_content, &js_url);
        assert!(!refs.is_empty());
        assert_eq!(
            refs[0].map_url.as_str(),
            "https://example.com/static/app.js.map"
        );
    }

    #[test]
    fn test_detect_source_map_with_deprecated_syntax() {
        let parser = SourceMapParser::new();
        let js_content = r#"
        console.log("Hello");
        //@ sourceMappingURL=app.js.map
        "#;
        let js_url = Url::parse("https://example.com/static/app.js").unwrap();

        let refs = parser.detect_source_map_references(js_content, &js_url);
        assert!(!refs.is_empty());
    }

    #[test]
    fn test_parse_source_map() {
        let parser = SourceMapParser::new();
        let content = r#"{
            "version": 3,
            "file": "bundle.js",
            "sources": ["src/index.js", "src/admin/Dashboard.tsx"],
            "names": ["console", "log"],
            "mappings": "AAAA"
        }"#;

        let result = parser.parse_source_map(content);
        assert!(result.is_ok());

        let map = result.unwrap();
        assert_eq!(map.version, 3);
        assert_eq!(map.sources.len(), 2);
    }

    #[test]
    fn test_classify_admin_path() {
        let parser = SourceMapParser::new();
        let result = parser.classify_path("src/admin/Dashboard.tsx");

        assert!(result.is_some());
        let classified = result.unwrap();
        assert_eq!(classified.category, PathCategory::Admin);
        assert_eq!(classified.priority, Priority::Critical);
    }

    #[test]
    fn test_classify_api_path() {
        let parser = SourceMapParser::new();
        let result = parser.classify_path("src/api/users.ts");

        assert!(result.is_some());
        let classified = result.unwrap();
        assert_eq!(classified.category, PathCategory::Api);
        assert_eq!(classified.priority, Priority::High);
    }

    #[test]
    fn test_classify_secret_path() {
        let parser = SourceMapParser::new();
        let result = parser.classify_path("src/config/secrets.ts");

        assert!(result.is_some());
        let classified = result.unwrap();
        assert_eq!(classified.category, PathCategory::Secret);
        assert_eq!(classified.priority, Priority::Critical);
    }

    #[test]
    fn test_skip_node_modules() {
        let parser = SourceMapParser::new();
        let result = parser.classify_path("node_modules/react/index.js");

        assert!(result.is_none());
    }

    #[test]
    fn test_analyze_source_map() {
        let parser = SourceMapParser::new();
        let source_map = SourceMap {
            version: 3,
            file: Some("bundle.js".to_string()),
            sources: vec![
                "src/admin/Dashboard.tsx".to_string(),
                "src/api/users.ts".to_string(),
                "src/components/Button.tsx".to_string(),
                "node_modules/react/index.js".to_string(),
            ],
            sources_content: None,
            names: vec![],
            mappings: "AAAA".to_string(),
            source_root: None,
        };

        let analysis = parser.analyze_source_map(&source_map, "https://example.com/bundle.js.map");

        assert_eq!(analysis.total_sources, 4);
        assert!(!analysis.interesting_paths.is_empty());

        // Should find admin and api paths
        assert!(analysis
            .interesting_paths
            .iter()
            .any(|p| p.category == PathCategory::Admin));
        assert!(analysis
            .interesting_paths
            .iter()
            .any(|p| p.category == PathCategory::Api));

        // Should detect React
        assert!(analysis.frameworks_detected.contains(&"react".to_string()));
    }

    #[test]
    fn test_generate_report() {
        let parser = SourceMapParser::new();
        let analysis = SourceMapAnalysis {
            map_url: "https://example.com/app.js.map".to_string(),
            total_sources: 10,
            interesting_paths: vec![InterestingPath {
                path: "src/admin/panel.tsx".to_string(),
                category: PathCategory::Admin,
                priority: Priority::Critical,
                reason: "Admin panel component detected".to_string(),
            }],
            frameworks_detected: vec!["react".to_string()],
            project_structure: vec!["src".to_string()],
        };

        let report = parser.generate_report(&analysis);

        assert!(report.contains("Source Map Analysis"));
        assert!(report.contains("Total sources: 10"));
        assert!(report.contains("react"));
        assert!(report.contains("CRITICAL"));
        assert!(report.contains("admin/panel.tsx"));
    }

    #[test]
    fn test_extract_directory() {
        let parser = SourceMapParser::new();

        assert_eq!(
            parser.extract_directory("webpack://src/components/Button.tsx"),
            Some("src".to_string())
        );
        assert_eq!(
            parser.extract_directory("src/admin/Dashboard.tsx"),
            Some("src".to_string())
        );
    }

    #[test]
    fn test_detect_multiple_source_map_refs() {
        let parser = SourceMapParser::new();
        // A JS file that embeds its own content and references an external map
        let js_content = r#"
            (function() { return 42; })();
            //# sourceMappingURL=chunk1.js.map
        "#;
        let js_url = Url::parse("https://example.com/js/chunk1.js").unwrap();
        let refs = parser.detect_source_map_references(js_content, &js_url);
        assert!(!refs.is_empty());
        assert!(refs[0].map_url.as_str().contains("chunk1.js.map"));
        assert!(!refs[0].inline);
    }

    #[test]
    fn test_analyze_source_map_no_interesting_paths() {
        let parser = SourceMapParser::new();
        let source_map = SourceMap {
            version: 3,
            file: None,
            sources: vec![
                "src/components/Button.tsx".to_string(),
                "src/components/Modal.tsx".to_string(),
            ],
            sources_content: None,
            names: vec![],
            mappings: "AAAA".to_string(),
            source_root: None,
        };

        let analysis = parser.analyze_source_map(&source_map, "https://example.com/bundle.js.map");
        assert_eq!(analysis.total_sources, 2);
        // None of these paths are admin/api/secret etc., so no interesting paths expected
        assert!(
            analysis.interesting_paths.is_empty(),
            "Plain component paths should not be classified as interesting"
        );
    }

    #[test]
    fn test_parse_source_map_with_source_root() {
        let parser = SourceMapParser::new();
        let content = r#"{
            "version": 3,
            "sourceRoot": "/project/src",
            "sources": ["index.js"],
            "names": [],
            "mappings": "AAAA"
        }"#;

        let map = parser.parse_source_map(content).unwrap();
        assert_eq!(map.source_root, Some("/project/src".to_string()));
    }

    #[test]
    fn test_classify_auth_path() {
        let parser = SourceMapParser::new();
        let result = parser.classify_path("src/auth/login.ts");
        assert!(result.is_some());
        let classified = result.unwrap();
        assert_eq!(classified.category, PathCategory::Auth);
    }

    #[test]
    fn test_classify_test_path() {
        let parser = SourceMapParser::new();
        let result = parser.classify_path("src/__tests__/App.test.tsx");
        // Test paths should be classified (category Test)
        if let Some(classified) = result {
            assert_eq!(classified.category, PathCategory::Test);
        }
    }

    #[test]
    fn test_framework_detection_vue() {
        let parser = SourceMapParser::new();
        let source_map = SourceMap {
            version: 3,
            file: None,
            sources: vec![
                "node_modules/vue/dist/vue.esm.js".to_string(),
                "src/App.vue".to_string(),
            ],
            sources_content: None,
            names: vec![],
            mappings: "AAAA".to_string(),
            source_root: None,
        };

        let analysis = parser.analyze_source_map(&source_map, "https://example.com/bundle.js.map");
        assert!(
            analysis.frameworks_detected.contains(&"vue".to_string()),
            "Should detect Vue from node_modules/vue"
        );
    }
}
