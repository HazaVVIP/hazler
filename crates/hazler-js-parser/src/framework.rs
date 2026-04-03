//! Framework detection and framework-specific endpoint extraction

use once_cell::sync::Lazy;
use regex::Regex;

/// Detected framework information
#[derive(Debug, Clone, PartialEq)]
pub enum Framework {
    React,
    Angular,
    Vue,
    NextJs,
    Nuxt,
    Svelte,
    Ember,
    Backbone,
    Unknown,
}

/// Helper function to compile regex patterns safely
/// Panics only during initialization if patterns are invalid (fail-fast on startup)
fn compile_regex(pattern: &str) -> Regex {
    Regex::new(pattern).unwrap_or_else(|e| {
        panic!("Invalid regex pattern '{}': {}", pattern, e);
    })
}

/// Framework detection patterns
pub static FRAMEWORK_PATTERNS: Lazy<Vec<(Framework, Vec<Regex>)>> = Lazy::new(|| {
    vec![
        // React patterns
        (
            Framework::React,
            vec![
                compile_regex(r"react\."),
                compile_regex(r"React\."),
                compile_regex(r"from\s+['\x22]react['\x22]"),
                compile_regex(r"ReactDOM"),
                compile_regex(r"useState|useEffect|useContext"),
                compile_regex(r"__webpack_require__.*react"),
            ],
        ),
        // Next.js patterns
        (
            Framework::NextJs,
            vec![
                compile_regex(r"next/"),
                compile_regex(r"_next/static/"),
                compile_regex(r"__NEXT_DATA__"),
                compile_regex(r"next\.config"),
            ],
        ),
        // Angular patterns
        (
            Framework::Angular,
            vec![
                compile_regex(r"@angular/"),
                compile_regex(r"angular\."),
                compile_regex(r"ng-"),
                compile_regex(r"platformBrowserDynamic"),
                compile_regex(r"NgModule"),
            ],
        ),
        // Vue.js patterns
        (
            Framework::Vue,
            vec![
                compile_regex(r"vue\."),
                compile_regex(r"Vue\."),
                compile_regex(r"from\s+['\x22]vue['\x22]"),
                compile_regex(r"createApp|Vue\.component"),
                compile_regex(r"v-if|v-for|v-model"),
            ],
        ),
        // Nuxt patterns
        (
            Framework::Nuxt,
            vec![compile_regex(r"nuxt"), compile_regex(r"__NUXT__")],
        ),
        // Svelte patterns
        (
            Framework::Svelte,
            vec![compile_regex(r"svelte"), compile_regex(r"SvelteComponent")],
        ),
        // Ember patterns
        (
            Framework::Ember,
            vec![compile_regex(r"Ember\."), compile_regex(r"ember-")],
        ),
        // Backbone patterns
        (
            Framework::Backbone,
            vec![compile_regex(r"Backbone\."), compile_regex(r"backbone")],
        ),
    ]
});

/// Framework-specific endpoint patterns
pub static FRAMEWORK_ENDPOINT_PATTERNS: Lazy<Vec<(Framework, Vec<&'static str>)>> =
    Lazy::new(|| {
        vec![
            // React Router patterns
            (
                Framework::React,
                vec![
                    r#"<Route\s+path=["']([^"']+)["']"#,
                    r#"path:\s*["']([^"']+)["']"#,
                    r#"useNavigate|useLocation|useParams"#,
                    r#"BrowserRouter|HashRouter|MemoryRouter"#,
                ],
            ),
            // Next.js patterns
            (
                Framework::NextJs,
                vec![
                    r#"pages/api/([^"'\s]+)"#,
                    r#"/api/([^"'\s]+)"#,
                    r#"getServerSideProps|getStaticProps"#,
                ],
            ),
            // Angular patterns
            (
                Framework::Angular,
                vec![
                    r#"RouterModule\.forRoot"#,
                    r#"path:\s*['"]([^'"]+)['"]"#,
                    r#"\.navigate\(\s*\[['"]([^'"]+)['"]"#,
                    r#"HttpClient\.(get|post|put|delete|patch)"#,
                ],
            ),
            // Vue Router patterns
            (
                Framework::Vue,
                vec![
                    r#"createRouter|new\s+VueRouter"#,
                    r#"path:\s*['"]([^'"]+)['"]"#,
                    r#"\$router\.push|this\.router\.push"#,
                ],
            ),
        ]
    });

/// Pre-compiled framework-specific endpoint regexes.
///
/// Compiled once at program startup so that `extract_framework_endpoints`
/// does not recompile the same patterns on every call.
pub static FRAMEWORK_ENDPOINT_REGEXES: Lazy<Vec<(Framework, Vec<Regex>)>> = Lazy::new(|| {
    FRAMEWORK_ENDPOINT_PATTERNS
        .iter()
        .map(|(fw, patterns)| {
            let compiled = patterns.iter().filter_map(|p| Regex::new(p).ok()).collect();
            (fw.clone(), compiled)
        })
        .collect()
});

/// Detect framework from JavaScript content
pub fn detect_framework(js_content: &str) -> Vec<Framework> {
    let mut detected = Vec::new();

    for (framework, patterns) in FRAMEWORK_PATTERNS.iter() {
        for pattern in patterns {
            if pattern.is_match(js_content) {
                detected.push(framework.clone());
                break;
            }
        }
    }

    if detected.is_empty() {
        detected.push(Framework::Unknown);
    }

    detected
}

/// Get framework-specific endpoint patterns as pre-compiled [`Regex`] references.
///
/// Using pre-compiled regexes avoids re-compiling the same patterns on every
/// call to `extract_framework_endpoints`, which is a significant performance
/// improvement when processing large JavaScript files.
pub fn get_compiled_framework_patterns(framework: &Framework) -> Vec<&'static Regex> {
    FRAMEWORK_ENDPOINT_REGEXES
        .iter()
        .filter(|(fw, _)| fw == framework)
        .flat_map(|(_, patterns)| patterns.iter())
        .collect()
}

/// Get framework-specific endpoint patterns
pub fn get_framework_patterns(framework: &Framework) -> Vec<String> {
    FRAMEWORK_ENDPOINT_PATTERNS
        .iter()
        .filter(|(fw, _)| fw == framework)
        .flat_map(|(_, patterns)| patterns.iter().map(|p| p.to_string()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_react() {
        let code = r#"
            import React from 'react';
            import { useState, useEffect } from 'react';
        "#;
        let detected = detect_framework(code);
        assert!(detected.contains(&Framework::React));
    }

    #[test]
    fn test_detect_nextjs() {
        let code = r#"
            const data = __NEXT_DATA__;
            import { GetServerSideProps } from 'next';
        "#;
        let detected = detect_framework(code);
        assert!(detected.contains(&Framework::NextJs));
    }

    #[test]
    fn test_detect_angular() {
        let code = r#"
            import { NgModule } from '@angular/core';
            platformBrowserDynamic().bootstrapModule(AppModule);
        "#;
        let detected = detect_framework(code);
        assert!(detected.contains(&Framework::Angular));
    }

    #[test]
    fn test_detect_vue() {
        let code = r#"
            import Vue from 'vue';
            const app = createApp({});
        "#;
        let detected = detect_framework(code);
        assert!(detected.contains(&Framework::Vue));
    }

    #[test]
    fn test_unknown_framework() {
        let code = "console.log('hello world');";
        let detected = detect_framework(code);
        assert!(detected.contains(&Framework::Unknown));
    }
}
