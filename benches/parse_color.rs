use criterion::{criterion_group, criterion_main, Criterion};
use pastel::parser::parse_color;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("parse_hex", |b| {
        b.iter(|| {
            parse_color("#ff0077");
        })
    });
    c.bench_function("parse_hex_short", |b| {
        b.iter(|| {
            parse_color("#f07");
        })
    });
    c.bench_function("parse_rgb", |b| {
        b.iter(|| {
            parse_color("rgb(255, 125, 0)");
        })
    });
    c.bench_function("parse_hsl", |b| b.iter(|| parse_color("hsl(280,20%,50%)")));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
