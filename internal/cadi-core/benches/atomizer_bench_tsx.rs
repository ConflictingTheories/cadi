use criterion::{criterion_group, criterion_main, Criterion};
use cadi_core::atomizer::AtomExtractor;
use cadi_core::atomizer::AtomizerConfig;

fn make_large_tsx() -> String {
    let snippet = "export function Hello(): JSX.Element { return <div>Hello</div>; }\n";
    snippet.repeat(200_000)
}

fn bench_tsx_extractor(c: &mut Criterion) {
    let src = make_large_tsx();
    let extractor = AtomExtractor::new("typescript", AtomizerConfig::default());

    c.bench_function("atomizer::extract_tsx_large", |b| {
        b.iter(|| extractor.extract(&src).expect("extract failed"))
    });
}

criterion_group!(benches, bench_tsx_extractor);
criterion_main!(benches);
