use colored::Colorize;
use hazler_fuzzer::{FuzzerConfig, UrlMutator, ParamDiscovery, FuzzStrategy};
use tracing::info;
use url::Url;

/// Apply fuzzing to discovered URLs
pub fn apply_fuzzing(
    urls: &[Url],
    fuzz: bool,
    fuzz_params: bool,
    fuzz_endpoints: bool,
    fuzz_level: &str,
) -> Vec<Url> {
    if !fuzz && !fuzz_params && !fuzz_endpoints {
        return Vec::new();
    }

    info!("Starting smart fuzzing...");

    let mut fuzzed_urls = Vec::new();

    // Configure fuzzer based on level
    let config = match fuzz_level {
        "minimal" => FuzzerConfig::minimal(),
        "aggressive" => FuzzerConfig::aggressive(),
        _ => FuzzerConfig::default(),
    };

    // Apply URL mutations
    if fuzz || fuzz_endpoints {
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
    if fuzz || fuzz_params {
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
