# Hazler Fuzzer

Smart fuzzing module for Hazler web crawler. Provides intelligent fuzzing capabilities for proactive endpoint discovery.

## Features

### 🎯 URL Mutation Engine
Automatically generates URL variations to discover hidden endpoints:
- **Pluralization**: `/api/user` → `/api/users`
- **File Extensions**: `/api/user` → `/api/user.json`, `/api/user.xml`, `/api/user.php`
- **API Versioning**: `/api/user` → `/api/v1/user`, `/api/v2/user`, `/api/v3/user`

### 🔍 Parameter Discovery
Tests common parameter names on discovered endpoints:
- Built-in wordlist of 70+ common parameters
- Individual, combination, and exhaustive fuzzing strategies
- Smart parameter value testing based on parameter type

### 📚 Built-in Wordlists
- **60+ Common Endpoints**: admin, api, users, login, etc.
- **70+ Common Parameters**: id, user_id, token, page, limit, etc.
- **30+ File Extensions**: json, xml, php, html, etc.

### 🔐 BOLA/IDOR Detection
Response comparison for identifying access control issues:
- Response similarity analysis
- Status code comparison
- Content-based detection
- Automatic suspicious pattern identification

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
hazler-fuzzer = { path = "../hazler-fuzzer" }
```

## Usage

### Basic URL Mutation

```rust
use hazler_fuzzer::{UrlMutator, FuzzerConfig};
use url::Url;

let config = FuzzerConfig::default();
let mutator = UrlMutator::new(config);

let url = Url::parse("https://api.example.com/user").unwrap();
let mutations = mutator.generate_mutations(&url);

for mutation in mutations {
    println!("Testing: {} ({})", mutation.url, mutation.description);
}
```

### Parameter Discovery

```rust
use hazler_fuzzer::{ParamDiscovery, FuzzStrategy};
use url::Url;

let discovery = ParamDiscovery::new(FuzzStrategy::Individual);
let base_url = Url::parse("https://api.example.com/endpoint").unwrap();

let param_urls = discovery.generate_param_urls(&base_url);
for url in param_urls {
    println!("Testing: {}", url);
}
```

### BOLA/IDOR Detection

```rust
use hazler_fuzzer::{BolaDetector, Response};

let detector = BolaDetector::default();

let response1 = Response::new(
    "https://api.example.com/user/1".to_string(),
    200,
    "User 1 data".to_string(),
);

let response2 = Response::new(
    "https://api.example.com/user/2".to_string(),
    200,
    "User 2 data".to_string(),
);

let comparison = detector.compare_responses(&response1, &response2);

if comparison.is_suspicious {
    println!("⚠️  Potential BOLA/IDOR: {}", comparison.reason.unwrap());
}
```

### Configuration Levels

```rust
use hazler_fuzzer::FuzzerConfig;

// Minimal fuzzing (fastest)
let minimal = FuzzerConfig::minimal();

// Default fuzzing (balanced)
let default = FuzzerConfig::default();

// Aggressive fuzzing (most comprehensive)
let aggressive = FuzzerConfig::aggressive();
```

## CLI Integration

When using with Hazler CLI:

```bash
# Enable all fuzzing features
hazler https://api.example.com --fuzz

# Parameter discovery only
hazler https://api.example.com --fuzz-params

# Endpoint fuzzing only
hazler https://api.example.com --fuzz-endpoints

# Aggressive fuzzing mode
hazler https://api.example.com --fuzz --fuzz-level aggressive
```

## Architecture

The fuzzer is organized into several modules:

- **config**: Configuration and fuzzing strategies
- **mutator**: URL mutation engine (pluralization, extensions, versioning)
- **params**: Parameter discovery and fuzzing
- **wordlists**: Built-in wordlists for common endpoints, parameters, and extensions
- **detector**: BOLA/IDOR detection through response comparison

## Testing

The crate includes 27+ comprehensive tests:

```bash
cargo test -p hazler-fuzzer
```

Test coverage:
- URL mutation tests (6 tests)
- Parameter discovery tests (5 tests)
- Wordlist tests (5 tests)
- BOLA/IDOR detection tests (7 tests)
- Configuration tests (3 tests)
- Doc tests (1 test)

## Performance

The fuzzer is designed to be efficient:
- Maximum mutations limit to prevent explosion
- Deduplication to avoid redundant tests
- Lazy-loaded wordlists for minimal memory usage
- Configurable aggressiveness levels

## Examples

### Example 1: Basic Mutation

Input: `https://api.example.com/user`

Generated mutations:
- `https://api.example.com/users` (pluralization)
- `https://api.example.com/user.json` (extension)
- `https://api.example.com/user.xml` (extension)
- `https://api.example.com/v1/user` (versioning)
- `https://api.example.com/v2/user` (versioning)

### Example 2: Parameter Fuzzing

Input: `https://api.example.com/search`

Generated parameter tests:
- `https://api.example.com/search?id=1`
- `https://api.example.com/search?user_id=1`
- `https://api.example.com/search?page=1`
- `https://api.example.com/search?limit=10`
- ... (70+ variations)

## Contributing

Contributions are welcome! Please ensure:
1. All tests pass: `cargo test -p hazler-fuzzer`
2. Code is formatted: `cargo fmt`
3. No warnings: `cargo clippy`

## License

MIT License - see LICENSE file for details
