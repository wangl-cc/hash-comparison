use std::hint::black_box;

use criterion::{
    AxisScale, BenchmarkId, Criterion, PlotConfiguration, Throughput, criterion_group,
    criterion_main,
};
use rand::{RngCore, SeedableRng};

/// Sizes of the buffers to generate (bytes).
pub const SIZES_BYTES: &[usize] = &[
    1 << 4,  // 16 B
    1 << 8,  // 256 B
    1 << 12, // 4 KiB
    1 << 16, // 64 KiB
    1 << 20, // 1 MiB
];

const U64_BATCH_COUNTS: &[usize] = &[1, 16, 64, 256, 1024];
const SEED: u64 = 42;

type BenchGroup<'a> = criterion::BenchmarkGroup<'a, criterion::measurement::WallTime>;

trait BenchFn {
    fn set_throughput(&self, group: &mut BenchGroup<'_>);
    fn bench<R: RngCore>(&self, group: &mut BenchGroup<'_>, name: &str, rng: R);
}

fn bench_all_prngs(group: &mut BenchGroup<'_>, bench_fn: &impl BenchFn) {
    bench_fn.set_throughput(group);
    bench_fn.bench(group, "PCG64", rand_pcg::Pcg64::seed_from_u64(SEED));
    bench_fn.bench(group, "PCG64-MCG", rand_pcg::Pcg64Mcg::seed_from_u64(SEED));
    bench_fn.bench(group, "PCG64DXSM", rand_pcg::Pcg64Dxsm::seed_from_u64(SEED));
    bench_fn.bench(
        group,
        "xoshiro256++",
        rand_xoshiro::Xoshiro256PlusPlus::seed_from_u64(SEED),
    );
    bench_fn.bench(
        group,
        "xoshiro256**",
        rand_xoshiro::Xoshiro256StarStar::seed_from_u64(SEED),
    );
}

struct U64Gen {
    count: usize,
}

impl BenchFn for U64Gen {
    fn set_throughput(&self, group: &mut BenchGroup<'_>) {
        group.throughput(Throughput::Elements(self.count as u64));
    }

    fn bench<R: RngCore>(&self, group: &mut BenchGroup<'_>, name: &str, mut rng: R) {
        let count = self.count;
        let mut acc = 0u64;
        group.bench_with_input(BenchmarkId::new(name, count), &count, |b, &count| {
            b.iter(|| {
                for _ in 0..count {
                    acc ^= rng.next_u64();
                }
                black_box(acc);
            })
        });
    }
}

struct FillBytes {
    size: usize,
}

impl BenchFn for FillBytes {
    fn set_throughput(&self, group: &mut BenchGroup<'_>) {
        group.throughput(Throughput::Bytes(self.size as u64));
    }

    fn bench<R: RngCore>(&self, group: &mut BenchGroup<'_>, name: &str, mut rng: R) {
        let size = self.size;
        let mut buf = vec![0u8; size];
        group.bench_with_input(BenchmarkId::new(name, size), &size, |b, &_size| {
            b.iter(|| {
                rng.fill_bytes(&mut buf);
                black_box(&buf);
            })
        });
    }
}

fn u64_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("u64_generation");

    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    for &count in U64_BATCH_COUNTS {
        let bench_fn = U64Gen { count };
        bench_all_prngs(&mut group, &bench_fn);
    }

    group.finish();
}

fn bytes_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("bytes_generation");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    for &size in SIZES_BYTES {
        let bench_fn = FillBytes { size };
        bench_all_prngs(&mut group, &bench_fn);
    }

    group.finish();
}

fn criterion_config() -> Criterion {
    Criterion::default()
}

criterion_group! {
    name = benches;
    config = criterion_config();
    targets = u64_generation, bytes_generation
}
criterion_main!(benches);
