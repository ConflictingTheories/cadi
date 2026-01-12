use std::time::Instant;
use cadi_core::atomizer::{AtomExtractor, AtomizerConfig};

fn bench(name: &str, extractor: &AtomExtractor, src: &str, iterations: usize) {
    // Warm-up
    extractor.extract(src).expect("warmup failed");

    let mut total = 0u128;
    for _ in 0..iterations {
        let start = Instant::now();
        extractor.extract(src).expect("extract failed");
        let dur = start.elapsed().as_micros();
        total += dur;
    }
    let avg = total as f64 / iterations as f64;
    println!("{:<24} avg = {:.2} ms over {} iters", name, avg / 1000.0, iterations);
}

fn make_large_rust() -> String {
    let snippet = "pub fn foo() { let x = 1; println!(\"{}\", x); }\n";
    snippet.repeat(20000) // ~1M chars
}

fn make_large_jsx() -> String {
    let snippet = "export function Hello() { return <div>Hello</div>; }\n";
    snippet.repeat(200000)
}

fn make_large_tsx() -> String {
    let snippet = "export function Hello(): JSX.Element { return <div>Hello</div>; }\n";
    snippet.repeat(200000)
}

fn make_large_html() -> String {
    let snippet = "<div>\n  <p>Some content</p>\n</div>\n";
    snippet.repeat(200000)
}

fn main() {
    println!("Starting micro-benchmarks (iterations = 5)");

    let rust_src = make_large_rust();
    let js_src = make_large_jsx();
    let tsx_src = make_large_tsx();
    let html_src = make_large_html();

    let iterations = 5;

    let rust_ex = AtomExtractor::new("rust", AtomizerConfig::default());
    bench("rust::extract", &rust_ex, &rust_src, iterations);

    let jsx_ex = AtomExtractor::new("javascript", AtomizerConfig::default());
    bench("jsx::extract", &jsx_ex, &js_src, iterations);

    let tsx_ex = AtomExtractor::new("typescript", AtomizerConfig::default());
    bench("tsx::extract", &tsx_ex, &tsx_src, iterations);

    let html_ex = AtomExtractor::new("html", AtomizerConfig::default());
    bench("html::extract", &html_ex, &html_src, iterations);

    println!("Done");
}
