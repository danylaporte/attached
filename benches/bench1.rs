use attached::{container, var, Container};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::sync::OnceLock;

var!(VAR1: usize, CTX);

fn criterion_benchmark(c: &mut Criterion) {
    let ctx = Container::new();
    let cell = OnceLock::new();

    c.bench_function("var::get_or_init", |b| {
        b.iter(|| black_box(&ctx).get_or_init(*VAR1, || 1usize))
    });

    c.bench_function("var::get", |b| b.iter(|| black_box(&ctx).get(*VAR1)));

    c.bench_function("cell::get_or_init", |b| {
        b.iter(|| black_box(&cell).get_or_init(|| 1usize))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

container!(CTX);
