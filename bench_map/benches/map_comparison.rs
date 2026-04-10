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

/// String key byte lengths to benchmark.
const STR_KEY_LENS: &[usize] = &[16, 64, 256];

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

/// Generate `n` distinct `(String, String)` pairs.
///
/// Keys are zero-padded decimal indices padded to exactly `key_len` bytes,
/// guaranteeing uniqueness and a fixed key size. Values are random lowercase
/// ASCII strings of the same length.
fn gen_string_pairs(n: usize, key_len: usize) -> Vec<(String, String)> {
    let mut rng = make_rng();
    let mut pairs: Vec<(String, String)> = (0..n)
        .map(|i| {
            let key = format!("{i:0>key_len$}");
            let val: String = (0..key_len)
                .map(|_| rng.random_range(b'a'..=b'z') as char)
                .collect();
            (key, val)
        })
        .collect();
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

// ── string insert ─────────────────────────────────────────────────────────────

fn str_bench_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("str_map_insert");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    for &key_len in STR_KEY_LENS {
        for &n in SIZES {
            let pairs = gen_string_pairs(n, key_len);
            group.throughput(Throughput::Elements(n as u64));

            group.bench_with_input(
                BenchmarkId::new(format!("Vec@key={key_len}b"), n),
                &pairs,
                |b, pairs| {
                    b.iter(|| {
                        let mut v: Vec<(String, String)> = Vec::with_capacity(pairs.len());
                        for (k, val) in pairs {
                            v.push((k.clone(), val.clone()));
                        }
                        black_box(v);
                    });
                },
            );

            group.bench_with_input(
                BenchmarkId::new(format!("HashMap@key={key_len}b"), n),
                &pairs,
                |b, pairs| {
                    b.iter(|| {
                        let mut m: HashMap<String, String> = HashMap::with_capacity(pairs.len());
                        for (k, val) in pairs {
                            m.insert(k.clone(), val.clone());
                        }
                        black_box(m);
                    });
                },
            );

            group.bench_with_input(
                BenchmarkId::new(format!("BTreeMap@key={key_len}b"), n),
                &pairs,
                |b, pairs| {
                    b.iter(|| {
                        let mut m: BTreeMap<String, String> = BTreeMap::new();
                        for (k, val) in pairs {
                            m.insert(k.clone(), val.clone());
                        }
                        black_box(m);
                    });
                },
            );
        }
    }

    group.finish();
}

// ── string lookup (hit) ───────────────────────────────────────────────────────

fn str_bench_lookup_hit(c: &mut Criterion) {
    let mut group = c.benchmark_group("str_map_lookup_hit");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    for &key_len in STR_KEY_LENS {
        for &n in SIZES {
            let pairs = gen_string_pairs(n, key_len);
            // Keys guaranteed to be present.
            let lookup_keys: Vec<String> = {
                let mut rng = make_rng();
                (0..n)
                    .map(|_idx| pairs[rng.random_range(0..n)].0.clone())
                    .collect()
            };
            group.throughput(Throughput::Elements(n as u64));

            // Vec – linear scan
            {
                let vec = pairs.clone();
                let keys = lookup_keys.clone();
                group.bench_with_input(
                    BenchmarkId::new(format!("Vec@key={key_len}b"), n),
                    &(vec, keys),
                    |b, (vec, keys)| {
                        b.iter(|| {
                            let mut acc = 0usize;
                            for k in keys {
                                if let Some((_, v)) = vec.iter().find(|(ek, _)| ek == k) {
                                    acc ^= v.len();
                                }
                            }
                            black_box(acc);
                        });
                    },
                );
            }

            // HashMap
            {
                let map: HashMap<String, String> = pairs.iter().cloned().collect();
                let keys = lookup_keys.clone();
                group.bench_with_input(
                    BenchmarkId::new(format!("HashMap@key={key_len}b"), n),
                    &(map, keys),
                    |b, (map, keys)| {
                        b.iter(|| {
                            let mut acc = 0usize;
                            for k in keys {
                                if let Some(v) = map.get(k) {
                                    acc ^= v.len();
                                }
                            }
                            black_box(acc);
                        });
                    },
                );
            }

            // BTreeMap
            {
                let map: BTreeMap<String, String> = pairs.iter().cloned().collect();
                let keys = lookup_keys.clone();
                group.bench_with_input(
                    BenchmarkId::new(format!("BTreeMap@key={key_len}b"), n),
                    &(map, keys),
                    |b, (map, keys)| {
                        b.iter(|| {
                            let mut acc = 0usize;
                            for k in keys {
                                if let Some(v) = map.get(k) {
                                    acc ^= v.len();
                                }
                            }
                            black_box(acc);
                        });
                    },
                );
            }
        }
    }

    group.finish();
}

// ── string lookup (miss) ──────────────────────────────────────────────────────

fn str_bench_lookup_miss(c: &mut Criterion) {
    let mut group = c.benchmark_group("str_map_lookup_miss");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    for &key_len in STR_KEY_LENS {
        for &n in SIZES {
            let pairs = gen_string_pairs(n, key_len);
            // Keys guaranteed absent: pad indices starting from n with a
            // distinct prefix so they cannot collide with existing zero-padded keys.
            let lookup_keys: Vec<String> = (n..2 * n)
                .map(|i| format!("x{i:0>width$}", width = key_len.saturating_sub(1)))
                .collect();
            group.throughput(Throughput::Elements(n as u64));

            // Vec – linear scan
            {
                let vec = pairs.clone();
                let keys = lookup_keys.clone();
                group.bench_with_input(
                    BenchmarkId::new(format!("Vec@key={key_len}b"), n),
                    &(vec, keys),
                    |b, (vec, keys)| {
                        b.iter(|| {
                            let mut found = 0usize;
                            for k in keys {
                                found += vec.iter().any(|(ek, _)| ek == k) as usize;
                            }
                            black_box(found);
                        });
                    },
                );
            }

            // HashMap
            {
                let map: HashMap<String, String> = pairs.iter().cloned().collect();
                let keys = lookup_keys.clone();
                group.bench_with_input(
                    BenchmarkId::new(format!("HashMap@key={key_len}b"), n),
                    &(map, keys),
                    |b, (map, keys)| {
                        b.iter(|| {
                            let mut found = 0usize;
                            for k in keys {
                                found += map.contains_key(k) as usize;
                            }
                            black_box(found);
                        });
                    },
                );
            }

            // BTreeMap
            {
                let map: BTreeMap<String, String> = pairs.iter().cloned().collect();
                let keys = lookup_keys.clone();
                group.bench_with_input(
                    BenchmarkId::new(format!("BTreeMap@key={key_len}b"), n),
                    &(map, keys),
                    |b, (map, keys)| {
                        b.iter(|| {
                            let mut found = 0usize;
                            for k in keys {
                                found += map.contains_key(k) as usize;
                            }
                            black_box(found);
                        });
                    },
                );
            }
        }
    }

    group.finish();
}

// ── string iterate ────────────────────────────────────────────────────────────

fn str_bench_iterate(c: &mut Criterion) {
    let mut group = c.benchmark_group("str_map_iterate");
    group.plot_config(PlotConfiguration::default().summary_scale(AxisScale::Logarithmic));

    for &key_len in STR_KEY_LENS {
        for &n in SIZES {
            let pairs = gen_string_pairs(n, key_len);
            group.throughput(Throughput::Elements(n as u64));

            // Vec
            {
                let vec = pairs.clone();
                group.bench_with_input(
                    BenchmarkId::new(format!("Vec@key={key_len}b"), n),
                    &vec,
                    |b, vec| {
                        b.iter(|| {
                            let acc: usize = vec.iter().map(|(_, v)| v.len()).fold(0, |a, v| a ^ v);
                            black_box(acc);
                        });
                    },
                );
            }

            // HashMap
            {
                let map: HashMap<String, String> = pairs.iter().cloned().collect();
                group.bench_with_input(
                    BenchmarkId::new(format!("HashMap@key={key_len}b"), n),
                    &map,
                    |b, map| {
                        b.iter(|| {
                            let acc: usize = map.values().map(|v| v.len()).fold(0, |a, v| a ^ v);
                            black_box(acc);
                        });
                    },
                );
            }

            // BTreeMap
            {
                let map: BTreeMap<String, String> = pairs.iter().cloned().collect();
                group.bench_with_input(
                    BenchmarkId::new(format!("BTreeMap@key={key_len}b"), n),
                    &map,
                    |b, map| {
                        b.iter(|| {
                            let acc: usize = map.values().map(|v| v.len()).fold(0, |a, v| a ^ v);
                            black_box(acc);
                        });
                    },
                );
            }
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
    targets = bench_insert, bench_lookup_hit, bench_lookup_miss, bench_iterate,
              str_bench_insert, str_bench_lookup_hit, str_bench_lookup_miss, str_bench_iterate
}
criterion_main!(benches);
