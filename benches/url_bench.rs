use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hazler_core::normalizer::AdvancedUrlNormalizer;
use url::Url;

fn bench_url_normalization(c: &mut Criterion) {
    let normalizer = AdvancedUrlNormalizer::new();

    let test_urls = vec![
        "https://example.com/path/to/page?id=123&sort=asc",
        "https://example.com/PATH/TO/PAGE?sort=asc&id=123",
        "https://example.com/path/to/page/?id=123",
        "https://example.com:443/path/to/page?id=123",
        "https://example.com/path/../page?id=123",
    ];

    let mut group = c.benchmark_group("url_normalization");

    for (i, url_str) in test_urls.iter().enumerate() {
        let url = Url::parse(url_str).unwrap();
        group.bench_function(format!("normalize_url_{}", i), |b| {
            b.iter(|| {
                normalizer.normalize(black_box(&url))
            });
        });
    }

    group.bench_function("normalize_batch", |b| {
        let urls: Vec<Url> = test_urls
            .iter()
            .map(|s| Url::parse(s).unwrap())
            .collect();
        
        b.iter(|| {
            for url in &urls {
                normalizer.normalize(black_box(url));
            }
        });
    });

    group.finish();
}

fn bench_url_deduplication(c: &mut Criterion) {
    let normalizer = AdvancedUrlNormalizer::new();

    c.bench_function("deduplicate_100_urls", |b| {
        let urls: Vec<Url> = (0..100)
            .map(|i| Url::parse(&format!("https://example.com/page{}", i)).unwrap())
            .collect();

        b.iter(|| {
            let mut seen = std::collections::HashSet::new();
            for url in &urls {
                let normalized = normalizer.normalize(black_box(url));
                seen.insert(normalized);
            }
            black_box(seen.len())
        });
    });
}

criterion_group!(benches, bench_url_normalization, bench_url_deduplication);
criterion_main!(benches);
