use criterion::{criterion_group, criterion_main, Criterion};

use cadi_core::atomizer::{AtomExtractor, AtomizerConfig};

fn make_large_rust() -> String {
    let snippet = "pub fn foo() { let x = 1; println!(\"{}\", x); }\n";
    snippet.repeat(20_000) // ~1M chars
}

fn bench_atom_extractor(c: &mut Criterion) {
    let src = make_large_rust();
    let extractor = AtomExtractor::new("rust", AtomizerConfig::default());

    c.bench_function("atomizer::extract_rust_large", |b| {
        b.iter(|| extractor.extract(&src).expect("extract failed"))
    });
}

criterion_group!(benches, bench_atom_extractor);
criterion_main!(benches);
