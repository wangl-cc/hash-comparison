use std::hint::black_box;

use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use digest::Digest;
use rand::{Rng, SeedableRng, rngs::SmallRng};

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

fn cryptographic_hash(c: &mut Criterion) {
    let mut group = c.benchmark_group("cryptographic_hash");

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
                    let mut hasher = blake3::Hasher::new();
                    hasher.update(&data);
                    black_box(hasher.finalize());
                },
                BatchSize::SmallInput,
            )
        });
        group.bench_with_input(BenchmarkId::new("blake2b512", size), &size, |b, &size| {
            b.iter_batched(
                || generate_data(size),
                |data| {
                    let mut hasher = blake2::Blake2b512::new();
                    hasher.update(&data);
                    black_box(hasher.finalize());
                },
                BatchSize::SmallInput,
            )
        });
    }

    group.finish();
}

fn non_cryptographic_hash(c: &mut Criterion) {
    let mut group = c.benchmark_group("non_cryptographic_hash");

    for &size in SIZES_BYTES {
        group.bench_with_input(BenchmarkId::new("xxh3", size), &size, |b, &size| {
            b.iter_batched(
                || generate_data(size),
                |data| {
                    let mut hasher = xxhash_rust::xxh3::Xxh3::new();
                    hasher.update(&data);
                    black_box(hasher.digest128());
                },
                BatchSize::SmallInput,
            )
        });
        group.bench_with_input(BenchmarkId::new("gxhash", size), &size, |b, &size| {
            b.iter_batched(
                || generate_data(size),
                |data| {
                    black_box(gxhash::gxhash128(&data, 0));
                },
                BatchSize::SmallInput,
            )
        });
    }

    group.finish();
}

criterion_group!(benches, non_cryptographic_hash, cryptographic_hash);
criterion_main!(benches);
