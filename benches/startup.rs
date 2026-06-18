use criterion::{criterion_group, criterion_main, Criterion};

fn bench_config_load(c: &mut Criterion) {
    c.bench_function("config_load", |b| b.iter(|| edit::config::load_config()));
}

fn bench_config_validate(c: &mut Criterion) {
    c.bench_function("config_validate", |b| {
        b.iter(|| {
            let mut cfg = edit::config::load_config();
            edit::config::validate_config(&mut cfg);
            cfg
        })
    });
}

criterion_group!(benches, bench_config_load, bench_config_validate);
criterion_main!(benches);
