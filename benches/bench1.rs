use attached::{var_ctx, Var, Vars};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use once_cell::sync::OnceCell;
use static_init::dynamic;

#[dynamic]
static VAR1: Var<usize, CTX> = Var::new();

fn criterion_benchmark(c: &mut Criterion) {
    let ctx = Vars::new();
    let cell = OnceCell::new();

    c.bench_function("var::get_or_init", |b| {
        b.iter(|| black_box(&ctx).get_or_init(&VAR1, || 1usize))
    });

    c.bench_function("var::get", |b| b.iter(|| black_box(&ctx).get(&VAR1)));

    c.bench_function("cell::get_or_init", |b| {
        b.iter(|| black_box(&cell).get_or_init(|| 1usize))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

var_ctx!(CTX);
