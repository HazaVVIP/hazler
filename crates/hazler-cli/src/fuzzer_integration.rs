use colored::Colorize;
use hazler_fuzzer::{FuzzStrategy, FuzzerConfig, ParamDiscovery, UrlMutator};
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
