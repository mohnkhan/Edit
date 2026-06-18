use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

fn bench_encoding_detect(c: &mut Criterion) {
    let sample = "Hello, World! こんにちは 🌍".repeat(100);
    let bytes = sample.as_bytes();
    c.bench_function("encoding_detect_utf8", |b| {
        b.iter(|| edit::encoding::detect_encoding(bytes))
    });
}

fn bench_encoding_detect_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("encoding_detect_by_size");
    for size in [64usize, 512, 4096, 65536] {
        let content: Vec<u8> = "Hello, World! ".bytes().cycle().take(size).collect();
        group.bench_with_input(
            BenchmarkId::new("ascii_bytes", size),
            &content,
            |b, bytes| {
                b.iter(|| edit::encoding::detect_encoding(bytes));
            },
        );
    }
    group.finish();
}

fn bench_decode_utf8(c: &mut Criterion) {
    use edit::encoding::{decode, EncodingId};
    let sample = "Hello, World! こんにちは 🌍".repeat(100);
    let bytes = sample.as_bytes().to_vec();
    c.bench_function("decode_utf8", |b| {
        b.iter(|| decode(&bytes, EncodingId::Utf8))
    });
}

fn bench_decode_windows1252(c: &mut Criterion) {
    use edit::encoding::{decode, EncodingId};
    // Windows-1252 compatible bytes (pure ASCII portion)
    let bytes: Vec<u8> = b"Hello, World! "
        .iter()
        .cycle()
        .copied()
        .take(10_000)
        .collect();
    c.bench_function("decode_windows1252", |b| {
        b.iter(|| decode(&bytes, EncodingId::Windows1252))
    });
}

fn bench_encode_utf8(c: &mut Criterion) {
    use edit::encoding::{encode, EncodingId};
    let sample = "Hello, World! こんにちは 🌍".repeat(100);
    c.bench_function("encode_utf8", |b| {
        b.iter(|| encode(&sample, EncodingId::Utf8))
    });
}

criterion_group!(
    benches,
    bench_encoding_detect,
    bench_encoding_detect_sizes,
    bench_decode_utf8,
    bench_decode_windows1252,
    bench_encode_utf8
);
criterion_main!(benches);
