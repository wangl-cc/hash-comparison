use std::hint::black_box;

use blake3::Hasher as Blake3Hasher;
use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use rand::{Rng, SeedableRng, rngs::SmallRng};
use sha2::Digest;
use xxhash_rust::xxh3::Xxh3;

/// Data sizes (bytes) to test (from 16B to 8MB)
const SIZES_BYTES: &[usize] = &[
    1 << 4,  // 16 B
    1 << 8,  // 256 B
    1 << 12, // 4 KiB
    1 << 16, // 64 KiB
    1 << 20, // 1 MiB
    1 << 24, // 16 MiB
];

/// Generate random data for benchmarking
fn generate_data(size: usize) -> Vec<u8> {
    let mut rng = SmallRng::seed_from_u64(42);
    (0..size).map(|_| rng.random()).collect()
}

fn hash_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash_comparison");

    for &size in SIZES_BYTES {
        group.bench_with_input(BenchmarkId::new("sha256", size), &size, |b, &size| {
            b.iter_batched(
                || generate_data(size),
                |data| {
                    let mut hasher = sha2::Sha256::new();
                    hasher.update(&data);
                    black_box(hasher.finalize());
                },
                BatchSize::SmallInput,
            )
        });
        group.bench_with_input(BenchmarkId::new("sha512", size), &size, |b, &size| {
            b.iter_batched(
                || generate_data(size),
                |data| {
                    let mut hasher = sha2::Sha512::new();
                    hasher.update(&data);
                    black_box(hasher.finalize());
                },
                BatchSize::SmallInput,
            )
        });
        group.bench_with_input(BenchmarkId::new("blake3", size), &size, |b, &size| {
            b.iter_batched(
                || generate_data(size),
                |data| {
                    let mut hasher = Blake3Hasher::new();
                    hasher.update(&data);
                    black_box(hasher.finalize());
                },
                BatchSize::SmallInput,
            )
        });
        group.bench_with_input(BenchmarkId::new("xxhash64", size), &size, |b, &size| {
            b.iter_batched(
                || generate_data(size),
                |data| {
                    let mut hasher = Xxh3::new();
                    hasher.update(&data);
                    black_box(hasher.digest());
                },
                BatchSize::SmallInput,
            )
        });
        group.bench_with_input(BenchmarkId::new("xxhash128", size), &size, |b, &size| {
            b.iter_batched(
                || generate_data(size),
                |data| {
                    let mut hasher = Xxh3::new();
                    hasher.update(&data);
                    black_box(hasher.digest128());
                },
                BatchSize::SmallInput,
            )
        });
    }

    group.finish();
}

criterion_group!(benches, hash_comparison);
criterion_main!(benches);
