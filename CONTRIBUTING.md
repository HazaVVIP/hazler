# Contributing to Hazler

Thank you for your interest in contributing to Hazler! This document provides guidelines and instructions for contributing.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Project Structure](#project-structure)
- [Making Changes](#making-changes)
- [Testing](#testing)
- [Code Style](#code-style)
- [Commit Messages](#commit-messages)
- [Pull Request Process](#pull-request-process)
- [Releasing](#releasing)
- [Reporting Bugs](#reporting-bugs)
- [Suggesting Features](#suggesting-features)

## Code of Conduct

- Be respectful and inclusive
- Focus on constructive feedback
- Help others learn and grow
- Maintain a positive and collaborative environment

## Getting Started

1. Fork the repository on GitHub
2. Clone your fork locally
3. Set up the development environment
4. Create a new branch for your changes
5. Make your changes
6. Test your changes
7. Submit a pull request

## Development Setup

### Prerequisites

- Rust 1.70 or later
- System dependencies (see [README.md](README.md#prerequisites))

### Setup Steps

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/hazler.git
cd hazler

# Add upstream remote
git remote add upstream https://github.com/HazaVVIP/hazler.git

# Install dependencies and build
cargo build

# Run tests to ensure everything works
cargo test
```

### Development Tools

We recommend installing these tools for better development experience:

```bash
# Rust formatter
rustup component add rustfmt

# Rust linter
rustup component add clippy

# Documentation tool
cargo install cargo-doc
```

## Project Structure

```
hazler/
├── crates/
│   ├── hazler-core/       # Core crawling logic
│   │   ├── src/
│   │   │   ├── config.rs        # Configuration
│   │   │   ├── crawler.rs       # Main crawler
│   │   │   ├── queue.rs         # URL queue
│   │   │   ├── scope.rs         # Scope validation
│   │   │   ├── types.rs         # Data types
│   │   │   ├── noise_filter.rs  # WAF/404 noise suppression
│   │   │   ├── normalizer.rs    # URL normalisation
│   │   │   ├── persistence.rs   # JSON/SQLite state storage
│   │   │   ├── retry.rs         # Retry with exponential backoff
│   │   │   ├── circuit_breaker.rs # Per-domain circuit breaker
│   │   │   ├── rate_limiter.rs  # Token-bucket rate limiter
│   │   │   ├── differ/          # SimHash, clustering, baseline
│   │   │   └── ...
│   │   └── tests/         # Integration tests
│   ├── hazler-http/       # HTTP client wrapper
│   │   ├── src/
│   │   │   ├── client.rs  # HTTP client
│   │   │   └── error.rs   # Error types
│   │   └── tests/
│   ├── hazler-parser/     # HTML parsing
│   │   ├── src/
│   │   │   ├── parser.rs  # HTML parser
│   │   │   └── error.rs   # Error types
│   │   └── tests/
│   ├── hazler-js-parser/  # JavaScript endpoint extraction
│   ├── hazler-secrets/    # Secret & credential detection
│   ├── hazler-browser/    # Headless browser integration (CDP/chromiumoxide)
│   ├── hazler-fuzzer/     # URL mutation and parameter fuzzing
│   └── hazler-cli/        # CLI interface
│       └── src/
│           └── main.rs    # CLI entry point
├── docs/
│   ├── CLI.md             # Full CLI flag reference
│   └── ARCHITECTURE.md    # Crate dependency graph and data-flow
├── Cargo.toml             # Workspace manifest
├── CHANGELOG.md           # Version history
├── ROADMAP.md             # Feature roadmap
├── SECURITY.md            # Security policy
├── README.md
└── CONTRIBUTING.md        # This file
```

## Making Changes

### Branching Strategy

- `main` - Stable, production-ready code
- `feature/*` - New features
- `fix/*` - Bug fixes
- `docs/*` - Documentation updates
- `refactor/*` - Code refactoring

Example:
```bash
git checkout -b feature/add-csv-output
```

### Writing Code

1. **Keep changes focused**: One logical change per PR
2. **Write tests**: Add tests for new functionality
3. **Update documentation**: Update README, docs, and comments as needed
4. **Follow existing patterns**: Match the style and structure of existing code
5. **Handle errors properly**: Use proper error types and error handling

### Adding a New Feature

Example: Adding CSV output format

1. Update types in `hazler-core/src/types.rs` if needed
2. Add output logic in `hazler-cli/src/main.rs`
3. Add tests
4. Update README with new feature
5. Update help text in CLI

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run tests for specific crate
cargo test -p hazler-core

# Run specific test
cargo test test_name

# Run tests with output
cargo test -- --nocapture
```

### Writing Tests

Tests should be:
- **Clear**: Easy to understand what's being tested
- **Isolated**: Don't depend on other tests
- **Deterministic**: Same results every time
- **Fast**: Run quickly to encourage frequent testing

Example test:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_scope_validation() {
        let base = Url::parse("https://example.com").unwrap();
        let validator = ScopeValidator::new(base);
        
        let same_domain = Url::parse("https://example.com/page").unwrap();
        assert!(validator.is_in_scope(&same_domain));
        
        let different_domain = Url::parse("https://other.com").unwrap();
        assert!(!validator.is_in_scope(&different_domain));
    }
}
```

### Integration Tests

For end-to-end tests, create files in `crates/*/tests/`:

```rust
// crates/hazler-core/tests/integration_test.rs
use hazler_core::{Config, Crawler};

#[tokio::test]
async fn test_basic_crawl() {
    // Test implementation
}
```

## Code Style

### Rust Style Guidelines

We follow the official [Rust Style Guide](https://doc.rust-lang.org/stable/style-guide/). Key points:

- Use `rustfmt` for automatic formatting
- Use `clippy` for linting
- Maximum line length: 100 characters
- Use 4 spaces for indentation (not tabs)

### Formatting

Before committing, run:

```bash
# Format all code
cargo fmt

# Check formatting without modifying
cargo fmt -- --check
```

### Linting

Run clippy to catch common issues:

```bash
# Run clippy
cargo clippy

# Run clippy with strict mode
cargo clippy -- -D warnings
```

### Naming Conventions

- **Types**: `PascalCase` (e.g., `CrawlResult`, `UrlQueue`)
- **Functions**: `snake_case` (e.g., `parse_html`, `extract_links`)
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `MAX_DEPTH`, `DEFAULT_TIMEOUT`)
- **Modules**: `snake_case` (e.g., `http_client`, `url_parser`)

### Documentation

All public APIs must have documentation:

```rust
/// Crawls a website starting from the given URL.
///
/// # Arguments
///
/// * `start_url` - The URL to begin crawling from
///
/// # Returns
///
/// Returns a `CrawlResult` containing all crawled pages and statistics.
///
/// # Errors
///
/// Returns an error if the initial URL cannot be fetched.
///
/// # Examples
///
/// ```
/// use hazler_core::{Config, Crawler};
/// use url::Url;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let config = Config::new();
/// let crawler = Crawler::new(config)?;
/// let url = Url::parse("https://example.com")?;
/// let result = crawler.crawl(url).await?;
/// # Ok(())
/// # }
/// ```
pub async fn crawl(&self, start_url: Url) -> Result<CrawlResult> {
    // Implementation
}
```

## Commit Messages

### Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

### Examples

```
feat(cli): add CSV output format

Add support for CSV output format via --output-format csv flag.
Includes proper escaping and header row.

Closes #42
```

```
fix(core): prevent duplicate URL crawling

Fixed issue where URLs with trailing slashes were treated as
different from the same URLs without trailing slashes.

Fixes #38
```

```
docs(readme): update installation instructions

Added prerequisites section with platform-specific dependencies.
Clarified build instructions for Windows users.
```

## Pull Request Process

### Before Submitting

1. **Update from upstream**: Rebase your branch on latest `main`
   ```bash
   git fetch upstream
   git rebase upstream/main
   ```

2. **Run all tests**: Ensure all tests pass
   ```bash
   cargo test
   ```

3. **Format code**: Run rustfmt
   ```bash
   cargo fmt
   ```

4. **Lint code**: Run clippy
   ```bash
   cargo clippy
   ```

5. **Update documentation**: Update README, docs, CHANGELOG as needed

### Submitting

1. Push your branch to your fork
2. Open a Pull Request on GitHub
3. Fill out the PR template completely
4. Link any related issues

### PR Title Format

Use the same format as commit messages:
```
feat(cli): add CSV output format
fix(core): prevent duplicate URL crawling
docs: update installation instructions
```

### PR Description

Include:
- **What**: What changes are included
- **Why**: Why these changes are needed
- **How**: How the changes work
- **Testing**: How you tested the changes
- **Screenshots**: If applicable (UI changes)
- **Breaking Changes**: If any
- **Related Issues**: Link to issues

### Review Process

1. Maintainers will review your PR
2. Address any feedback or requested changes
3. Once approved, a maintainer will merge your PR

### After Merge

1. Delete your feature branch
2. Update your local `main`:
   ```bash
   git checkout main
   git pull upstream main
   ```

## Releasing

This section is for maintainers who are preparing a new release.

### Prerequisites

- Write access to the repository
- A `CARGO_REGISTRY_TOKEN` secret configured in the repository settings (for crates.io publishing)

### Steps

1. **Bump the version** using the provided script. This updates `Cargo.toml`, `Dockerfile`,
   `install.sh`, and `CHANGELOG.md` in one shot:

   ```bash
   ./scripts/bump-version.sh <new-version>
   # e.g. ./scripts/bump-version.sh 0.3.0
   # e.g. ./scripts/bump-version.sh 0.3.0-alpha.1
   ```

2. **Fill in the changelog** — edit the new section in `CHANGELOG.md` with release notes.

3. **Commit and tag**:

   ```bash
   git commit -am "chore: bump version to <new-version>"
   git tag v<new-version>
   git push origin main --tags
   ```

4. **The release workflow runs automatically** on tag push and will:
   - Run the full CI gate (tests, clippy, fmt, security audit)
   - Build binaries for all platforms (Linux x86_64/aarch64, macOS x86_64/aarch64, Windows x86_64)
   - Create a GitHub Release with SHA256-verified archives
   - Build and push a multi-arch Docker image to GHCR
   - Publish all crates to crates.io (stable releases only)

### Manual release (workflow_dispatch)

You can also trigger a release manually from the **Actions** tab using the
`workflow_dispatch` event. Provide the version string (without the `v` prefix).
**The tag must already exist in the repository** — the workflow will fail if the
tag is missing to prevent accidental orphan tags.

### Pre-release versions

Any version containing `-alpha`, `-beta`, or `-rc` (e.g. `0.3.0-alpha.1`) is
automatically treated as a pre-release: the GitHub Release is marked as such and
the `latest` Docker tag is **not** updated.

## Reporting Bugs

### Before Reporting

1. Check if the bug has already been reported
2. Try to reproduce the bug with latest version
3. Gather relevant information

### Bug Report Template

```markdown
**Describe the bug**
A clear description of what the bug is.

**To Reproduce**
Steps to reproduce:
1. Run command: `hazler https://example.com`
2. See error

**Expected behavior**
What you expected to happen.

**Actual behavior**
What actually happened.

**Environment**
- OS: [e.g., Ubuntu 22.04]
- Rust version: [e.g., 1.75.0]
- Hazler version: [e.g., 0.1.0]

**Additional context**
Any other relevant information.
```

## Suggesting Features

### Feature Request Template

```markdown
**Feature Description**
Clear description of the feature.

**Use Case**
Why is this feature needed? What problem does it solve?

**Proposed Solution**
How you think this should work.

**Alternatives Considered**
Other solutions you've considered.

**Additional Context**
Any other relevant information.
```

## Questions?

If you have questions about contributing:

1. Check existing documentation
2. Search [GitHub Issues](https://github.com/HazaVVIP/hazler/issues)
3. Open a new issue with the `question` label
4. Contact the maintainers

## License

By contributing to Hazler, you agree that your contributions will be licensed under the MIT License.

---

Thank you for contributing to Hazler! 🎉
