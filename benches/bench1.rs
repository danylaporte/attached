use attached::{Var, VarCtx};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use once_cell::sync::OnceCell;
use static_init::dynamic;

#[dynamic]
static VAR1: Var<usize> = Var::new();

fn criterion_benchmark(c: &mut Criterion) {
    let ctx = VarCtx::new();
    let cell = OnceCell::new();

    c.bench_function("var::get_or_init", |b| {
        b.iter(|| VAR1.get_or_init(black_box(&ctx), || 1usize))
    });

    c.bench_function("var::get", |b| b.iter(|| VAR1.get(black_box(&ctx))));

    c.bench_function("cell::get_or_init", |b| {
        b.iter(|| black_box(&cell).get_or_init(|| 1usize))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
