use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

fn bench_rope_from_str(c: &mut Criterion) {
    let mut group = c.benchmark_group("rope_construction");
    for size in [1_000usize, 10_000, 100_000] {
        let content = "a".repeat(size);
        group.bench_with_input(BenchmarkId::new("from_str", size), &content, |b, s| {
            b.iter(|| edit::buffer::rope::EditorRope::from_str(s));
        });
    }
    group.finish();
}

fn bench_rope_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("rope_insert");
    for size in [1_000usize, 10_000, 100_000] {
        let content = "a".repeat(size);
        group.bench_with_input(
            BenchmarkId::new("insert_at_middle", size),
            &content,
            |b, s| {
                b.iter(|| {
                    let mut rope = edit::buffer::rope::EditorRope::from_str(s);
                    rope.insert_str(s.chars().count() / 2, "x");
                    rope
                });
            },
        );
    }
    group.finish();
}

fn bench_rope_delete(c: &mut Criterion) {
    let mut group = c.benchmark_group("rope_delete");
    for size in [1_000usize, 10_000, 100_000] {
        let content = "a".repeat(size);
        group.bench_with_input(
            BenchmarkId::new("delete_middle_char", size),
            &content,
            |b, s| {
                b.iter(|| {
                    let mut rope = edit::buffer::rope::EditorRope::from_str(s);
                    let mid = s.chars().count() / 2;
                    rope.delete_range(mid..mid + 1);
                    rope
                });
            },
        );
    }
    group.finish();
}

fn bench_rope_line_slice(c: &mut Criterion) {
    let line = "The quick brown fox jumps over the lazy dog.\n";
    let content = line.repeat(1_000);
    let rope = edit::buffer::rope::EditorRope::from_str(&content);

    c.bench_function("rope_line_slice_1000_lines", |b| {
        b.iter(|| {
            for i in 0..rope.line_count() {
                let _ = rope.line_slice(i);
            }
        })
    });
}

criterion_group!(
    benches,
    bench_rope_from_str,
    bench_rope_insert,
    bench_rope_delete,
    bench_rope_line_slice
);
criterion_main!(benches);
