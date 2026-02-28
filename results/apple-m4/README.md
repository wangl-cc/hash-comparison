# Benchmark Results

Running at 2026-02-28 17:50:04 +0800.

## Environment

- CPU: Apple M4
- OS: macOS 26.3 (25D125)
- Kernel: Darwin 25.3.0
- rustc: rustc 1.92.0 (ded5c06cf 2025-12-08)
- LLVM: 21.1.3

## Collect Settings

- Run benchmarks before collect: yes
- Quick mode: disabled

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
