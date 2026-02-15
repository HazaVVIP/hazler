//! # Hazler Fuzzer
//!
//! Smart fuzzing module for Hazler web crawler.
//! 
//! This crate provides intelligent fuzzing capabilities including:
//! - URL mutation engine (pluralization, extensions, versioning)
//! - Parameter discovery with common parameter wordlists
//! - Built-in wordlists for endpoints, parameters, and files
//! - BOLA/IDOR detection through response comparison
//!
//! ## Features
//!
//! - **URL Mutations**: Automatically generate URL variations
//!   - Pluralization: `/api/user` -> `/api/users`
//!   - Extensions: `/api/user` -> `/api/user.json`, `/api/user.xml`
//!   - Versioning: `/api/user` -> `/api/v1/user`, `/api/v2/user`
//!
//! - **Parameter Discovery**: Test common parameter names
//!   - Built-in wordlists for common API parameters
//!   - Automatic parameter fuzzing
//!
//! - **BOLA/IDOR Detection**: Compare responses to identify access control issues
//!   - Response similarity analysis
//!   - Pattern-based detection
//!
//! ## Example
//!
//! ```rust
//! use hazler_fuzzer::{UrlMutator, FuzzerConfig};
//! use url::Url;
//!
//! let config = FuzzerConfig::default();
//! let mutator = UrlMutator::new(config);
//! 
//! let url = Url::parse("https://api.example.com/user").unwrap();
//! let mutations = mutator.generate_mutations(&url);
//! 
//! for mutation in mutations {
//!     println!("Testing: {:?}", mutation.url);
//! }
//! ```

pub mod config;
pub mod detector;
pub mod mutator;
pub mod params;
pub mod wordlists;

pub use config::FuzzerConfig;
pub use detector::{BolaDetector, ResponseComparison};
pub use mutator::{Mutation, MutationType, UrlMutator};
pub use params::{FuzzStrategy, ParamDiscovery, ParamFuzzer};
pub use wordlists::{Wordlists, COMMON_ENDPOINTS, COMMON_PARAMS, FILE_EXTENSIONS};
