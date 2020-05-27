use criterion::{black_box, criterion_group, criterion_main, Criterion};
use nand2tetris_hdl_parser::parse_hdl;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("parse_hdl example.hdl", |b| {
        b.iter(|| parse_hdl(black_box(include_str!("../test_cases/example.hdl"))))
    });
    c.bench_function("parse_hdl Add16.hdl", |b| {
        b.iter(|| parse_hdl(black_box(include_str!("../test_cases/Add16.hdl"))))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
