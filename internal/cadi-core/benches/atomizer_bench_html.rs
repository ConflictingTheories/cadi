use criterion::{criterion_group, criterion_main, Criterion};
use cadi_core::atomizer::AtomExtractor;
use cadi_core::atomizer::AtomizerConfig;

fn make_large_html() -> String {
    let snippet = "<div>\n  <p>Some content</p>\n</div>\n";
    snippet.repeat(200_000)
}

fn bench_html_extractor(c: &mut Criterion) {
    let src = make_large_html();
    let extractor = AtomExtractor::new("html", AtomizerConfig::default());

    c.bench_function("atomizer::extract_html_large", |b| {
        b.iter(|| extractor.extract(&src).expect("extract failed"))
    });
}

criterion_group!(benches, bench_html_extractor);
criterion_main!(benches);
