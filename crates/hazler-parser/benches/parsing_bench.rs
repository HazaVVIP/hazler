use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hazler_parser::HtmlParser;
use url::Url;

const SMALL_HTML: &str = r#"<!DOCTYPE html>
<html>
<head><title>Test Page</title></head>
<body>
<a href="/page1">Link 1</a>
<a href="/page2">Link 2</a>
<a href="/page3">Link 3</a>
</body>
</html>"#;

const MEDIUM_HTML: &str = r#"<!DOCTYPE html>
<html>
<head><title>Test Page</title></head>
<body>
<nav>
    <a href="/home">Home</a>
    <a href="/about">About</a>
    <a href="/contact">Contact</a>
</nav>
<main>
    <article>
        <h1>Article Title</h1>
        <p>Article content</p>
        <a href="/article/1">Read more</a>
    </article>
    <article>
        <h1>Another Article</h1>
        <p>More content</p>
        <a href="/article/2">Read more</a>
    </article>
</main>
<footer>
    <a href="/privacy">Privacy</a>
    <a href="/terms">Terms</a>
    <a href="/sitemap">Sitemap</a>
</footer>
</body>
</html>"#;

fn bench_html_parsing(c: &mut Criterion) {
    let parser = HtmlParser::new();
    let base_url = Url::parse("https://example.com").unwrap();

    let mut group = c.benchmark_group("html_parsing");

    group.bench_function("parse_small_html", |b| {
        b.iter(|| parser.extract_links(black_box(SMALL_HTML), black_box(&base_url)));
    });

    group.bench_function("parse_medium_html", |b| {
        b.iter(|| parser.extract_links(black_box(MEDIUM_HTML), black_box(&base_url)));
    });

    // Benchmark repeated parsing (cache test)
    group.bench_function("parse_repeated", |b| {
        b.iter(|| {
            for _ in 0..100 {
                let _ = parser.extract_links(black_box(SMALL_HTML), black_box(&base_url));
            }
        });
    });

    group.finish();
}

fn bench_link_extraction(c: &mut Criterion) {
    let parser = HtmlParser::new();
    let base_url = Url::parse("https://example.com").unwrap();

    c.bench_function("extract_links_small", |b| {
        b.iter(|| {
            let result = parser.extract_links(black_box(SMALL_HTML), black_box(&base_url));
            black_box(result.map(|urls| urls.len()))
        });
    });

    c.bench_function("extract_links_medium", |b| {
        b.iter(|| {
            let result = parser.extract_links(black_box(MEDIUM_HTML), black_box(&base_url));
            black_box(result.map(|urls| urls.len()))
        });
    });
}

criterion_group!(benches, bench_html_parsing, bench_link_extraction);
criterion_main!(benches);
