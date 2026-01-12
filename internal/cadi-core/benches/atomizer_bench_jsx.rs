use criterion::{criterion_group, criterion_main, Criterion};
use cadi_core::atomizer::AtomExtractor;
use cadi_core::atomizer::AtomizerConfig;

fn make_large_jsx() -> String {
    let snippet = "export function Hello() { return <div>Hello</div>; }\n";
    snippet.repeat(200_000) // large
}

fn bench_jsx_extractor(c: &mut Criterion) {
    let src = make_large_jsx();
    let extractor = AtomExtractor::new("javascript", AtomizerConfig::default());

    c.bench_function("atomizer::extract_jsx_large", |b| {
        b.iter(|| extractor.extract(&src).expect("extract failed"))
    });
}

criterion_group!(benches, bench_jsx_extractor);
criterion_main!(benches);
