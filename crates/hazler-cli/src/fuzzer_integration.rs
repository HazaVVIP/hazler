use colored::Colorize;
use hazler_fuzzer::{FuzzStrategy, FuzzerConfig, ParamDiscovery, UrlMutator};
use std::io::Write;
use tracing::info;
use url::Url;

/// Apply fuzzing to discovered URLs based on fuzz flag and level
pub fn apply_fuzzing(urls: &[Url], fuzz: bool, fuzz_level: &str) -> Vec<Url> {
    // Check if fuzzing is disabled
    if !fuzz || fuzz_level == "off" {
        return Vec::new();
    }

    info!("Starting fuzzing with level: {}", fuzz_level);

    let mut fuzzed_urls = Vec::new();

    // Determine what to enable based on level
    let (enable_mutations, enable_params, enable_endpoints) = match fuzz_level {
        "minimal" => (true, false, false),   // Basic mutations only
        "default" => (true, false, false),   // Smart fuzzing (mutations)
        "aggressive" => (true, true, false), // Smart + params
        "full" => (true, true, true),        // All modes
        _ => (true, false, false),           // Default fallback
    };

    // Configure fuzzer based on level
    let config = match fuzz_level {
        "minimal" => FuzzerConfig::minimal(),
        "aggressive" | "full" => FuzzerConfig::aggressive(),
        _ => FuzzerConfig::default(),
    };

    // Apply URL mutations
    if enable_mutations || enable_endpoints {
        let mutator = UrlMutator::new(config.clone());

        for url in urls {
            let mutations = mutator.generate_mutations(url);

            eprintln!(
                "{} Generated {} mutations for {}",
                "→".bright_blue(),
                mutations.len().to_string().bright_green(),
                url.to_string().bright_cyan()
            );

            for mutation in mutations {
                fuzzed_urls.push(mutation.url);
            }
        }
    }

    // Apply parameter discovery
    if enable_params {
        let param_discovery = ParamDiscovery::new(FuzzStrategy::Individual);

        for url in urls {
            let param_urls = param_discovery.generate_param_urls(url);

            if !param_urls.is_empty() {
                eprintln!(
                    "{} Testing {} parameters on {}",
                    "→".bright_blue(),
                    param_urls.len().to_string().bright_green(),
                    url.to_string().bright_cyan()
                );

                fuzzed_urls.extend(param_urls);
            }
        }
    }

    eprintln!(
        "{} Total fuzzed URLs: {}",
        "✓".green().bold(),
        fuzzed_urls.len().to_string().bright_green().bold()
    );

    fuzzed_urls
}

/// Write a list of fuzzed URLs to a file (one URL per line)
pub fn write_fuzz_output(urls: &[Url], path: &str) -> anyhow::Result<()> {
    let file = std::fs::File::create(path)?;
    let mut writer = std::io::BufWriter::new(file);
    for url in urls {
        writeln!(writer, "{}", url)?;
    }
    info!("Wrote {} fuzzed URLs to {}", urls.len(), path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_fuzz_output_empty() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("fuzz.txt");
        let result = write_fuzz_output(&[], path.to_str().unwrap());
        assert!(result.is_ok());
        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(contents.is_empty());
    }

    #[test]
    fn test_write_fuzz_output_urls() {
        let dir = tempfile::TempDir::new().unwrap();
        let path = dir.path().join("fuzz.txt");
        let urls = vec![
            Url::parse("https://example.com/admin").unwrap(),
            Url::parse("https://example.com/api/v1").unwrap(),
        ];
        write_fuzz_output(&urls, path.to_str().unwrap()).unwrap();
        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(contents.contains("https://example.com/admin"));
        assert!(contents.contains("https://example.com/api/v1"));
        // Each URL is on its own line
        let lines: Vec<_> = contents.lines().collect();
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn test_apply_fuzzing_disabled() {
        let urls = vec![Url::parse("https://example.com/users").unwrap()];
        // With fuzz=false nothing is generated
        let result = apply_fuzzing(&urls, false, "default");
        assert!(result.is_empty());
    }

    #[test]
    fn test_apply_fuzzing_off_level() {
        let urls = vec![Url::parse("https://example.com/users").unwrap()];
        // level="off" also produces nothing
        let result = apply_fuzzing(&urls, true, "off");
        assert!(result.is_empty());
    }

    #[test]
    fn test_apply_fuzzing_minimal_generates_mutations() {
        let urls = vec![Url::parse("https://example.com/users").unwrap()];
        let result = apply_fuzzing(&urls, true, "minimal");
        // Minimal fuzzing should produce at least one mutation
        assert!(!result.is_empty());
    }
}
