use std::hint::black_box;

use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use rand::{RngCore, SeedableRng};

fn bench_u64(rng: &mut impl RngCore, acc: &mut u64) {
    let x = rng.next_u64();
    *acc ^= x;
    black_box(*acc);
}

fn u64_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("u64_generation");

    group.bench_function("small_rng", |b| {
        b.iter_batched(
            || (rand::rngs::SmallRng::seed_from_u64(42), 0u64),
            |(mut rng, mut acc)| bench_u64(&mut rng, &mut acc),
            BatchSize::LargeInput,
        )
    });

    group.bench_function("pcg64", |b| {
        b.iter_batched(
            || (rand_pcg::Pcg64::seed_from_u64(42), 0u64),
            |(mut rng, mut acc)| bench_u64(&mut rng, &mut acc),
            BatchSize::LargeInput,
        )
    });

    group.bench_function("pcg64mcg", |b| {
        b.iter_batched(
            || (rand_pcg::Pcg64Mcg::seed_from_u64(42), 0u64),
            |(mut rng, mut acc)| bench_u64(&mut rng, &mut acc),
            BatchSize::LargeInput,
        )
    });

    group.bench_function("pcg64dxsm", |b| {
        b.iter_batched(
            || (rand_pcg::Pcg64Dxsm::seed_from_u64(42), 0u64),
            |(mut rng, mut acc)| bench_u64(&mut rng, &mut acc),
            BatchSize::LargeInput,
        )
    });

    group.bench_function("xoshiro256plusplus", |b| {
        b.iter_batched(
            || (rand_xoshiro::Xoshiro256PlusPlus::seed_from_u64(42), 0u64),
            |(mut rng, mut acc)| bench_u64(&mut rng, &mut acc),
            BatchSize::LargeInput,
        )
    });

    group.bench_function("xoshiro256starstar", |b| {
        b.iter_batched(
            || (rand_xoshiro::Xoshiro256StarStar::seed_from_u64(42), 0u64),
            |(mut rng, mut acc)| bench_u64(&mut rng, &mut acc),
            BatchSize::LargeInput,
        )
    });

    group.finish();
}

/// Sizes of the buffers to generate (bytes)
const SIZES_BYTES: &[usize] = &[
    1 << 4,  // 16 B
    1 << 8,  // 256 B
    1 << 12, // 4 KiB
    1 << 16, // 64 KiB
    1 << 20, // 1 MiB
];

fn bytes_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("bytes_generation");

    for &size in SIZES_BYTES {
        group.bench_with_input(BenchmarkId::new("small_rng", size), &size, |b, &size| {
            b.iter_batched(
                || (rand::rngs::SmallRng::seed_from_u64(42), vec![0u8; size]),
                |(mut rng, mut buf)| {
                    rng.fill_bytes(&mut buf);
                    black_box(buf);
                },
                BatchSize::LargeInput,
            )
        });

        group.bench_with_input(BenchmarkId::new("pcg64", size), &size, |b, &size| {
            b.iter_batched(
                || (rand_pcg::Pcg64::seed_from_u64(42), vec![0u8; size]),
                |(mut rng, mut buf)| {
                    rng.fill_bytes(&mut buf);
                    black_box(buf);
                },
                BatchSize::LargeInput,
            )
        });

        group.bench_with_input(BenchmarkId::new("pcg64mcg", size), &size, |b, &size| {
            b.iter_batched(
                || (rand_pcg::Pcg64Mcg::seed_from_u64(42), vec![0u8; size]),
                |(mut rng, mut buf)| {
                    rng.fill_bytes(&mut buf);
                    black_box(buf);
                },
                BatchSize::LargeInput,
            )
        });

        group.bench_with_input(BenchmarkId::new("pcg64dxsm", size), &size, |b, &size| {
            b.iter_batched(
                || (rand_pcg::Pcg64Dxsm::seed_from_u64(42), vec![0u8; size]),
                |(mut rng, mut buf)| {
                    rng.fill_bytes(&mut buf);
                    black_box(buf);
                },
                BatchSize::LargeInput,
            )
        });

        group.bench_with_input(
            BenchmarkId::new("xoshiro256plusplus", size),
            &size,
            |b, &size| {
                b.iter_batched(
                    || {
                        (
                            rand_xoshiro::Xoshiro256PlusPlus::seed_from_u64(42),
                            vec![0u8; size],
                        )
                    },
                    |(mut rng, mut buf)| {
                        rng.fill_bytes(&mut buf);
                        black_box(buf);
                    },
                    BatchSize::LargeInput,
                )
            },
        );

        group.bench_with_input(
            BenchmarkId::new("xoshiro256starstar", size),
            &size,
            |b, &size| {
                b.iter_batched(
                    || {
                        (
                            rand_xoshiro::Xoshiro256StarStar::seed_from_u64(42),
                            vec![0u8; size],
                        )
                    },
                    |(mut rng, mut buf)| {
                        rng.fill_bytes(&mut buf);
                        black_box(buf);
                    },
                    BatchSize::LargeInput,
                )
            },
        );
    }

    group.finish();
}

criterion_group!(benches, u64_generation, bytes_generation);
criterion_main!(benches);
