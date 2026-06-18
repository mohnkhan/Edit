# Benchmark Results

**Date**: 2026-06-18  
**Toolchain**: Rust stable (cargo bench, release profile)  
**Machine**: x86_64 Linux  
**Criterion**: 100 samples, 3s warmup per benchmark

---

## Startup (`benches/startup.rs`)

| Benchmark | Time (median) | CI |
|---|---|---|
| `config_load` | 2.07 µs | [2.05 µs – 2.09 µs] |
| `config_validate` | 2.11 µs | [2.10 µs – 2.12 µs] |

Config loading + validation completes in ~4 µs total — well within the imperceptible range for startup.

---

## Encoding / Keystroke (`benches/keystroke.rs`)

### Encoding detection (by input size)

| Input size | Time (median) |
|---|---|
| 64 bytes | 17.6 ns |
| 512 bytes | 17.6 ns |
| 4 KB | 104 ns |
| 64 KB | 1.54 µs |

Detection is effectively O(n) in byte count. Flat at small sizes (BOM check short-circuits).

### Decode / encode

| Benchmark | Time (median) |
|---|---|
| `decode_utf8` (~2 KB sample) | 2.69 µs |
| `decode_windows1252` (~2 KB sample) | 572 ns |
| `encode_utf8` (~2 KB sample) | 118 ns |

---

## Rope / Large File (`benches/large_file.rs`)

### Construction (`EditorRope::from_str`)

| Size | Time (median) |
|---|---|
| 1 000 chars | 1.06 µs |
| 10 000 chars | 4.65 µs |
| 100 000 chars | 58.3 µs |

### Insert at midpoint

| Size | Time (median) |
|---|---|
| 1 000 chars | 1.64 µs |
| 10 000 chars | 5.87 µs |
| 100 000 chars | 55.5 µs |

### Delete at midpoint

| Size | Time (median) |
|---|---|
| 1 000 chars | 1.65 µs |
| 10 000 chars | 5.58 µs |
| 100 000 chars | 53.2 µs |

### Line iteration

| Benchmark | Time (median) |
|---|---|
| `rope_line_slice` (iterate 1 000 lines) | 2.40 ms |

Rope insert/delete scales sub-linearly as expected from the O(log n) balanced tree structure. A 100× increase in buffer size yields only a ~34× increase in operation time.

---

## Notes

- HTML reports with per-sample graphs are written to `target/criterion/` by Criterion.
- Re-run with `cargo bench` to update; `cargo bench -- <name>` to run a single benchmark.
- Gnuplot was not available; plots use the Criterion `plotters` backend instead.
