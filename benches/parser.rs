use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hack_hdl_parser::parse_hdl;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("parse_hdl", |b| b.iter(|| parse_hdl(black_box(include_str!("../test_cases/example.hdl")))));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);