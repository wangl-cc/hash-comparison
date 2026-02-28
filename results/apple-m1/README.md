# Benchmark Results

Running at 2026-02-28 15:36:59 +0800.

## Environment

- CPU: Apple M1
- OS: macOS 26.3 (25D125)
- Kernel: Darwin 25.3.0
- rustc: rustc 1.93.1 (01f6ddf75 2026-02-11)
- LLVM: 21.1.8

## Collect Settings

- Run benchmarks before collect: yes
- Quick mode: disabled
- Extra cargo bench args: `u64`

## Results

### Hash

#### Non-Cryptographic Hash Throughput

![Non-Cryptographic Hash Throughput](charts/non_cryptographic_hash_lines_throughput.svg)

#### Cryptographic Hash Throughput

![Cryptographic Hash Throughput](charts/cryptographic_hash_lines_throughput.svg)

### PRNG

#### u64 Generation Throughput

![u64 Generation Throughput](charts/u64_generation_lines_throughput.svg)

#### Bytes Generation Throughput

![Bytes Generation Throughput](charts/bytes_generation_lines_throughput.svg)
