use std::hint::black_box;

use bench_hash::hash;
use criterion::{AxisScale, BenchmarkId, Criterion, PlotConfiguration, Throughput};
use rand::{Rng, SeedableRng, rngs::SmallRng};

/// Input size exponents `n` for `2^n` bytes (from 16 B to 256 MiB)
const SIZE_POW2_EXP: &[u32] = &[4, 8, 12, 16, 20, 24, 28];

/// Generate random data for benchmarking.
fn generate_data(size: usize) -> Vec<u8> {
    let mut rng = SmallRng::seed_from_u64(42);
    (0..size).map(|_| rng.random()).collect()
}

fn bench_in_group<O>(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    name: &str,
    data: &[u8],
    f: fn(&[u8]) -> O,
) {
    group.bench_with_input(BenchmarkId::new(name, data.len()), data, |b, data| {
        b.iter(|| black_box(f(black_box(data))))
    });
}

fn bench_xor128_simd_baselines(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    data: &[u8],
) {
    #[cfg(target_arch = "x86_64")]
    {
        bench_in_group(group, "XOR-128-SSE2", data, bench_hash::xor_hash128_sse2);
        if std::is_x86_feature_detected!("avx2") {
            bench_in_group(group, "XOR-128-AVX2", data, bench_hash::xor_hash128_avx2);
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        bench_in_group(group, "XOR-128-NEON", data, bench_hash::xor_hash128_neon);
    }
}

fn non_cryptographic_hash(c: &mut Criterion) {
    let mut group = c.benchmark_group("non_cryptographic_hash");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    for &exp in SIZE_POW2_EXP {
        let size = 1usize << exp;
        let data = generate_data(size);
        let data = data.as_slice();
        group.throughput(Throughput::Bytes(size as u64));
        bench_in_group(&mut group, "XOR-64-ILP", data, bench_hash::xor_hash64);
        bench_xor128_simd_baselines(&mut group, data);

        bench_in_group(
            &mut group,
            "RAPIDHASH-64",
            data,
            rapidhash::v3::rapidhash_v3,
        );
        bench_in_group(&mut group, "XXH3-64", data, xxhash_rust::xxh3::xxh3_64);
        bench_in_group(&mut group, "XXH3-128", data, xxhash_rust::xxh3::xxh3_128);
        bench_in_group(&mut group, "GXHASH-64", data, |input| {
            gxhash::gxhash64(input, 0)
        });
        bench_in_group(&mut group, "GXHASH-128", data, |input| {
            gxhash::gxhash128(input, 0)
        });
    }

    group.finish();
}

fn cryptographic_hash(c: &mut Criterion) {
    let mut group = c.benchmark_group("cryptographic_hash");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    for &exp in SIZE_POW2_EXP {
        let size = 1usize << exp;
        let data = generate_data(size);
        let data = data.as_slice();
        group.throughput(Throughput::Bytes(size as u64));
        bench_in_group(&mut group, "SHA2-256", data, hash::<sha2::Sha256>);
        bench_in_group(&mut group, "SHA2-512", data, hash::<sha2::Sha512>);
        bench_in_group(&mut group, "BLAKE3-256", data, blake3::hash);
        bench_in_group(&mut group, "BLAKE2B-512", data, hash::<blake2::Blake2b512>);
    }

    group.finish();
}

fn criterion_config() -> Criterion {
    Criterion::default()
}

criterion::criterion_group! {
    name = benches;
    config = criterion_config();
    targets = non_cryptographic_hash, cryptographic_hash
}
criterion::criterion_main!(benches);
