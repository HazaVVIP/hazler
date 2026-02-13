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

/// Framework detection patterns
pub static FRAMEWORK_PATTERNS: Lazy<Vec<(Framework, Vec<Regex>)>> = Lazy::new(|| {
    vec![
        // React patterns
        (
            Framework::React,
            vec![
                Regex::new(r"react\.").unwrap(),
                Regex::new(r"React\.").unwrap(),
                Regex::new(r"from\s+['\x22]react['\x22]").unwrap(),
                Regex::new(r"ReactDOM").unwrap(),
                Regex::new(r"useState|useEffect|useContext").unwrap(),
                Regex::new(r"__webpack_require__.*react").unwrap(),
            ],
        ),
        // Next.js patterns
        (
            Framework::NextJs,
            vec![
                Regex::new(r"next/").unwrap(),
                Regex::new(r"_next/static/").unwrap(),
                Regex::new(r"__NEXT_DATA__").unwrap(),
                Regex::new(r"next\.config").unwrap(),
            ],
        ),
        // Angular patterns
        (
            Framework::Angular,
            vec![
                Regex::new(r"@angular/").unwrap(),
                Regex::new(r"angular\.").unwrap(),
                Regex::new(r"ng-").unwrap(),
                Regex::new(r"platformBrowserDynamic").unwrap(),
                Regex::new(r"NgModule").unwrap(),
            ],
        ),
        // Vue.js patterns
        (
            Framework::Vue,
            vec![
                Regex::new(r"vue\.").unwrap(),
                Regex::new(r"Vue\.").unwrap(),
                Regex::new(r"from\s+['\x22]vue['\x22]").unwrap(),
                Regex::new(r"createApp|Vue\.component").unwrap(),
                Regex::new(r"v-if|v-for|v-model").unwrap(),
            ],
        ),
        // Nuxt patterns
        (
            Framework::Nuxt,
            vec![
                Regex::new(r"nuxt").unwrap(),
                Regex::new(r"__NUXT__").unwrap(),
            ],
        ),
        // Svelte patterns
        (
            Framework::Svelte,
            vec![
                Regex::new(r"svelte").unwrap(),
                Regex::new(r"SvelteComponent").unwrap(),
            ],
        ),
        // Ember patterns
        (
            Framework::Ember,
            vec![
                Regex::new(r"Ember\.").unwrap(),
                Regex::new(r"ember-").unwrap(),
            ],
        ),
        // Backbone patterns
        (
            Framework::Backbone,
            vec![
                Regex::new(r"Backbone\.").unwrap(),
                Regex::new(r"backbone").unwrap(),
            ],
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
