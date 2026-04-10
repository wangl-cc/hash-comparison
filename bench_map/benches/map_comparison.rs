use std::{
    collections::{BTreeMap, HashMap},
    hint::black_box,
};

use criterion::{
    AxisScale, BenchmarkId, Criterion, PlotConfiguration, Throughput, criterion_group,
    criterion_main,
};
use rand::{Rng, SeedableRng, rngs::SmallRng};

/// Collection sizes (number of entries) to benchmark.
const SIZES: &[usize] = &[4, 16, 64, 256, 1_024, 4_096];

fn make_rng() -> SmallRng {
    SmallRng::seed_from_u64(42)
}

/// Generate `n` distinct (key, value) pairs with random u64 keys and values.
fn gen_pairs(n: usize) -> Vec<(u64, u64)> {
    let mut rng = make_rng();
    let mut pairs: Vec<(u64, u64)> = (0..n as u64).map(|key| (key, rng.random())).collect();
    // Shuffle so insertion order is randomized.
    use rand::seq::SliceRandom;
    pairs.shuffle(&mut rng);
    pairs
}

// ── insert ───────────────────────────────────────────────────────────────────

fn bench_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("map_insert");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    for &n in SIZES {
        let pairs = gen_pairs(n);
        group.throughput(Throughput::Elements(n as u64));

        group.bench_with_input(BenchmarkId::new("Vec", n), &pairs, |b, pairs| {
            b.iter(|| {
                let mut v: Vec<(u64, u64)> = Vec::with_capacity(pairs.len());
                for &(k, val) in pairs {
                    v.push((k, val));
                }
                black_box(v);
            });
        });

        group.bench_with_input(BenchmarkId::new("HashMap", n), &pairs, |b, pairs| {
            b.iter(|| {
                let mut m: HashMap<u64, u64> = HashMap::with_capacity(pairs.len());
                for &(k, val) in pairs {
                    m.insert(k, val);
                }
                black_box(m);
            });
        });

        group.bench_with_input(BenchmarkId::new("BTreeMap", n), &pairs, |b, pairs| {
            b.iter(|| {
                let mut m: BTreeMap<u64, u64> = BTreeMap::new();
                for &(k, val) in pairs {
                    m.insert(k, val);
                }
                black_box(m);
            });
        });
    }

    group.finish();
}

// ── lookup (hit) ─────────────────────────────────────────────────────────────

fn bench_lookup_hit(c: &mut Criterion) {
    let mut group = c.benchmark_group("map_lookup_hit");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    for &n in SIZES {
        let pairs = gen_pairs(n);
        // Keys that are guaranteed to be present.
        let lookup_keys: Vec<u64> = {
            let mut rng = make_rng();
            (0..n).map(|_idx| pairs[rng.random_range(0..n)].0).collect()
        };
        group.throughput(Throughput::Elements(n as u64));

        // Vec – linear scan
        {
            let vec: Vec<(u64, u64)> = pairs.clone();
            let keys = lookup_keys.clone();
            group.bench_with_input(
                BenchmarkId::new("Vec", n),
                &(vec, keys),
                |b, (vec, keys)| {
                    b.iter(|| {
                        let mut acc = 0u64;
                        for &k in keys {
                            if let Some(&(_, v)) = vec.iter().find(|&&(ek, _)| ek == k) {
                                acc ^= v;
                            }
                        }
                        black_box(acc);
                    });
                },
            );
        }

        // HashMap
        {
            let map: HashMap<u64, u64> = pairs.iter().copied().collect();
            let keys = lookup_keys.clone();
            group.bench_with_input(
                BenchmarkId::new("HashMap", n),
                &(map, keys),
                |b, (map, keys)| {
                    b.iter(|| {
                        let mut acc = 0u64;
                        for &k in keys {
                            if let Some(&v) = map.get(&k) {
                                acc ^= v;
                            }
                        }
                        black_box(acc);
                    });
                },
            );
        }

        // BTreeMap
        {
            let map: BTreeMap<u64, u64> = pairs.iter().copied().collect();
            let keys = lookup_keys.clone();
            group.bench_with_input(
                BenchmarkId::new("BTreeMap", n),
                &(map, keys),
                |b, (map, keys)| {
                    b.iter(|| {
                        let mut acc = 0u64;
                        for &k in keys {
                            if let Some(&v) = map.get(&k) {
                                acc ^= v;
                            }
                        }
                        black_box(acc);
                    });
                },
            );
        }
    }

    group.finish();
}

// ── lookup (miss) ────────────────────────────────────────────────────────────

fn bench_lookup_miss(c: &mut Criterion) {
    let mut group = c.benchmark_group("map_lookup_miss");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    for &n in SIZES {
        let pairs = gen_pairs(n);
        // Keys guaranteed to be absent (keys are in 0..n, so use n..2n).
        let lookup_keys: Vec<u64> = (n as u64..2 * n as u64).collect();
        group.throughput(Throughput::Elements(n as u64));

        // Vec – linear scan
        {
            let vec: Vec<(u64, u64)> = pairs.clone();
            let keys = lookup_keys.clone();
            group.bench_with_input(
                BenchmarkId::new("Vec", n),
                &(vec, keys),
                |b, (vec, keys)| {
                    b.iter(|| {
                        let mut acc = 0u64;
                        for &k in keys {
                            if let Some(&(_, v)) = vec.iter().find(|&&(ek, _)| ek == k) {
                                acc ^= v;
                            }
                        }
                        black_box(acc);
                    });
                },
            );
        }

        // HashMap
        {
            let map: HashMap<u64, u64> = pairs.iter().copied().collect();
            let keys = lookup_keys.clone();
            group.bench_with_input(
                BenchmarkId::new("HashMap", n),
                &(map, keys),
                |b, (map, keys)| {
                    b.iter(|| {
                        let mut found = 0u64;
                        for &k in keys {
                            found += map.contains_key(&k) as u64;
                        }
                        black_box(found);
                    });
                },
            );
        }

        // BTreeMap
        {
            let map: BTreeMap<u64, u64> = pairs.iter().copied().collect();
            let keys = lookup_keys.clone();
            group.bench_with_input(
                BenchmarkId::new("BTreeMap", n),
                &(map, keys),
                |b, (map, keys)| {
                    b.iter(|| {
                        let mut found = 0u64;
                        for &k in keys {
                            found += map.contains_key(&k) as u64;
                        }
                        black_box(found);
                    });
                },
            );
        }
    }

    group.finish();
}

// ── iterate ──────────────────────────────────────────────────────────────────

fn bench_iterate(c: &mut Criterion) {
    let mut group = c.benchmark_group("map_iterate");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    for &n in SIZES {
        let pairs = gen_pairs(n);
        group.throughput(Throughput::Elements(n as u64));

        // Vec
        {
            let vec: Vec<(u64, u64)> = pairs.clone();
            group.bench_with_input(BenchmarkId::new("Vec", n), &vec, |b, vec| {
                b.iter(|| {
                    let acc: u64 = vec.iter().map(|&(_, v)| v).fold(0, |a, v| a ^ v);
                    black_box(acc);
                });
            });
        }

        // HashMap
        {
            let map: HashMap<u64, u64> = pairs.iter().copied().collect();
            group.bench_with_input(BenchmarkId::new("HashMap", n), &map, |b, map| {
                b.iter(|| {
                    let acc: u64 = map.values().fold(0, |a, &v| a ^ v);
                    black_box(acc);
                });
            });
        }

        // BTreeMap
        {
            let map: BTreeMap<u64, u64> = pairs.iter().copied().collect();
            group.bench_with_input(BenchmarkId::new("BTreeMap", n), &map, |b, map| {
                b.iter(|| {
                    let acc: u64 = map.values().fold(0, |a, &v| a ^ v);
                    black_box(acc);
                });
            });
        }
    }

    group.finish();
}

fn criterion_config() -> Criterion {
    Criterion::default()
}

criterion_group! {
    name = benches;
    config = criterion_config();
    targets = bench_insert, bench_lookup_hit, bench_lookup_miss, bench_iterate
}
criterion_main!(benches);
