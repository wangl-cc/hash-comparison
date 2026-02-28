# PRNG Benchmarks

This crate compares throughput of non-cryptographic PRNGs for trusted-data
compute workloads.

It is not:

- a cryptographic security evaluation;
- a statistical certification suite (for example TestU01 / PractRand).

## What This Benchmark Measures

- The measured output is sustained generation throughput, not randomness quality.
- `u64_generation` reports how many `u64` values are produced per second (`Elements/s`).
- `bytes_generation` reports how many bytes are filled per second (`Bytes/s`).
- Throughput is observed across multiple batch sizes and buffer sizes.

Fixed setup:

- Deterministic seeding with `seed_from_u64(42)`.
- `black_box` to prevent dead-code elimination.
- RNG state is created once per benchmark case, then measured in steady-state.

## PRNG Included

### PCG family

- **[PCG64](https://docs.rs/rand_pcg/latest/rand_pcg/type.Pcg64.html)**:
  balanced baseline in the PCG family.
- **[PCG64-MCG](https://docs.rs/rand_pcg/latest/rand_pcg/type.Pcg64Mcg.html)**:
  multiplicative variant, often faster state transition.
- **[PCG64DXSM](https://docs.rs/rand_pcg/latest/rand_pcg/type.Pcg64Dxsm.html)**:
  DXSM output variant, commonly used for stronger quality margin.

### xoshiro family

- **[xoshiro256++](https://docs.rs/rand_xoshiro/latest/rand_xoshiro/struct.Xoshiro256PlusPlus.html)**:
  very fast general-purpose PRNG.
- **[xoshiro256\*\*](https://docs.rs/rand_xoshiro/latest/rand_xoshiro/struct.Xoshiro256StarStar.html)**:
  another high-throughput variant with a different output scrambler.

## Run

From the workspace root:

```bash
cargo bench -p bench_prng
```

Quick run:

```bash
BENCH_QUICK=1 cargo bench -p bench_prng
```

For repository-level collection (bench run + snapshot artifacts):

```bash
cargo run -p xtask -- collect-results --scope prng --run-bench
```

Quick collection:

```bash
cargo run -p xtask -- collect-results --scope prng --run-bench --quick
```

Note: `collect-results` defaults to `--scope all` when `--scope` is omitted.

## Reading the Results

- `u64_generation` reflects per-call generator core cost.
- `bytes_generation` reflects generator cost plus memory write pressure.
- Small sizes are more sensitive to loop/dispatch overhead.
- Large sizes are more bandwidth-sensitive.

## Result Artifacts

Source-of-truth snapshots are stored under [`../results`](../results):

- `../results/{kernel}_{cpu}/prng/README.md`
- `../results/{kernel}_{cpu}/prng/charts/u64_generation_lines_throughput.svg`
- `../results/{kernel}_{cpu}/prng/charts/bytes_generation_lines_throughput.svg`
- `../results/{kernel}_{cpu}/README.md` (platform-level aggregation)

Crate-local cross-platform index:

- `RESULTS.md` (generated from root results)
