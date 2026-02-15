# Hazler Benchmarks

This directory contains performance benchmarks for Hazler using [Criterion.rs](https://github.com/bheisler/criterion.rs).

## Running Benchmarks

### Run All Benchmarks

```bash
cargo bench
```

### Run Specific Benchmark Suite

```bash
# Run HTML parsing benchmarks
cargo bench -p hazler-parser --bench parsing_bench

# Run URL normalization benchmarks
cargo bench -p hazler-core --bench url_bench
```

### Run Specific Benchmark

```bash
cargo bench -p hazler-parser -- parse_small_html
```

## Benchmark Suites

### HTML Parsing (`hazler-parser/benches/parsing_bench.rs`)

Tests HTML parsing and link extraction performance:

- **parse_small_html**: Parse a small HTML document (~100 bytes)
- **parse_medium_html**: Parse a medium HTML document (~500 bytes)
- **parse_repeated**: Parse 100 small HTML documents in sequence
- **extract_links_small**: Extract links from small HTML
- **extract_links_medium**: Extract links from medium HTML

**Current Performance (Release Build):**
- Small HTML: ~8.9 µs per parse
- Medium HTML: ~22.7 µs per parse
- Batch processing: 100 pages in ~884 µs (8.84 µs/page average)

### URL Normalization (`hazler-core/benches/url_bench.rs`)

Tests URL normalization and deduplication performance:

- **normalize_url_X**: Normalize different URL patterns
- **normalize_batch**: Normalize multiple URLs in sequence
- **deduplicate_100_urls**: Deduplicate 100 URLs using HashSet

## Viewing Results

Benchmark results are saved to `target/criterion/`:

```bash
# View HTML report
open target/criterion/report/index.html  # macOS
xdg-open target/criterion/report/index.html  # Linux
```

## Continuous Integration

Benchmarks can be run in CI to detect performance regressions:

```bash
# Run benchmarks without generating plots (faster in CI)
cargo bench -- --noplot

# Compare with baseline
cargo bench -- --baseline my-baseline
```

## Adding New Benchmarks

1. Create a new file in the appropriate crate's `benches/` directory
2. Add the benchmark to the crate's `Cargo.toml`:

```toml
[[bench]]
name = "my_bench"
harness = false
```

3. Use Criterion's API:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_my_function(c: &mut Criterion) {
    c.bench_function("my_function", |b| {
        b.iter(|| {
            // Code to benchmark
            black_box(my_function(black_box(input)))
        });
    });
}

criterion_group!(benches, bench_my_function);
criterion_main!(benches);
```

## Performance Goals

For v0.2.0 release:

- **Target Throughput**: 200+ pages/sec (10x improvement over v0.1.0)
- **Memory Usage**: <100MB for typical crawls
- **Latency**: <50ms average per request
- **Parsing Speed**: <10 µs for small pages, <30 µs for medium pages

## References

- [Criterion.rs User Guide](https://bheisler.github.io/criterion.rs/book/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
