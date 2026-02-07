# Benchmarks for Commonly Used Algorithms

## Scope

- Pseudo-Random Number Generators (PRNGs)
- Hash Functions

## Tested Algorithms

### PRNGs

- **xoshiro**: A family of fast PRNGs based on XOR, shift, and rotate operations. Designed by David Blackman and Sebastiano Vigna as successors to the xorshift family, offering excellent statistical quality and speed.
  - **xoshiro256++**: 256-bit state with an addition-based scrambler. Default PRNG in Julia and Rust (`SmallRng`).
  - **xoshiro256\*\***: 256-bit state with a multiplication-based scrambler. Default PRNG in .NET 6+, Lua, and GNU Fortran.
- **PCG**: Permuted Congruential Generator, designed by Melissa O'Neill. Combines a linear congruential generator with a permutation output function to achieve good statistical properties and performance.
  - **PCG64**: 128-bit state LCG with XSL-RR (xorshift low, random rotation) output function.
  - **PCG64 MCG**: Multiplicative congruential variant with smaller state (128-bit) and no stream support, but faster due to simpler state transition.
  - **PCG64 DXSM**: Uses DXSM (double xorshift multiply) output function, providing better statistical properties than XSL-RR. Default PRNG in NumPy.

### Hash Functions

#### Non-Cryptographic

- **XXH3**: From the xxHash family, designed by Yann Collet. Optimized for speed on modern CPUs, leveraging SIMD instructions (SSE2, AVX2, NEON) for high throughput.
  - **XXH3-64**: 64-bit output variant.
  - **XXH3-128**: 128-bit output variant.
- **GxHash**: Designed by Olivier Giniaux. Uses hardware-accelerated AES instructions (AES-NI on x86, AES on ARM) for extremely fast hashing. Passes all SMHasher tests.
  - **GxHash-128**: 128-bit output variant.

#### Cryptographic

- **SHA-2**: A family of cryptographic hash functions designed by the NSA, widely used for integrity verification and digital signatures. Hardware-accelerated on x86 (SHA-NI) and ARM (SHA2 instructions).
  - **SHA-256**: 256-bit output, uses 32-bit operations internally.
  - **SHA-512**: 512-bit output, uses 64-bit operations internally.
- **BLAKE2**: A cryptographic hash function designed by Jean-Philippe Aumasson et al. SHA-3 finalist, offering high performance and security. Faster than MD5 while providing security similar to SHA-3.
  - **BLAKE2b-256**: 256-bit output, optimized for 64-bit platforms.
  - **BLAKE2s-256**: 256-bit output, optimized for 32-bit platforms and small messages.
- **BLAKE3**: A modern cryptographic hash function designed by Jack O'Connor et al. Based on BLAKE2, featuring a Merkle tree structure that enables parallelization and incremental updates.
  - **BLAKE3**: 256-bit default output (extendable). Generally faster than SHA-2 on large inputs due to parallelization and efficient design.

## Test Environment

### Software

- rustc 1.92.0 (ded5c06cf 2025-12-08)
- Criterion (benchmark framework)

### Platform

- **Zen 5** (x86-64 Linux):
  - OS: Fedora 43
  - Kernel: 6.18.5-200.fc43.x86_64
  - CPU: AMD Ryzen 9 9950X @ 5.76 GHz
  - Memory: Dual-channel DDR5 6000 MT/s

- **M4** (aarch64 macOS):
  - OS: macOS Tahoe 26.2 (25C56)
  - Kernel: Darwin 25.2.0
  - CPU: Apple M4 @ 4.46 GHz

## Results

### PRNGs

#### Overall

- xoshiro256 variants have higher throughput than the PCG family.
- Within the PCG family, PCG64 MCG is the fastest; PCG64 DXSM performs similarly to or slightly better than the base PCG64.

#### Platform-specific

- **Zen 5**: xoshiro256\*\* is slightly faster than xoshiro256++.
- **M4**: xoshiro256++ is slightly faster than xoshiro256\*\*.

### Hash Functions

#### Non-Cryptographic

**Overall:**

- XXH3-64 and XXH3-128 share the same internal state and update logic; they only differ in the final digest computation. This results in nearly identical performance, with minor platform-dependent differences.
- Non-cryptographic hashes are significantly faster than cryptographic hashes. At larger input sizes, they can be 10x+ faster than the fastest cryptographic hash.

**Platform-specific:**

- **Zen 5**:
  - XXH3-128 is slightly faster than XXH3-64.
  - GxHash-128 performance comparison TBD (awaiting benchmark results).
- **M4**:
  - XXH3-64 is slightly faster than XXH3-128.
  - GxHash-128 performance comparison TBD (awaiting benchmark results).

#### Cryptographic

**Overall:**

- SHA-256 outperforms BLAKE3 at small sizes on both architectures.
- BLAKE2b-256 and BLAKE2s-256 performance comparison TBD (awaiting benchmark results).
- SHA-512 is consistently the slowest among the tested cryptographic hashes.

**Platform-specific:**

- **Zen 5**:
  - SHA-256 performs better at small sizes (16 B, 256 B).
  - BLAKE2b-256 is optimized for 64-bit platforms; performance comparison TBD.
  - BLAKE3 and SHA-256 are close around 4 KiB (BLAKE3 slightly ahead), with BLAKE3 clearly faster at larger sizes.
- **M4**:
  - SHA-256 performs better at small sizes (16 B, 256 B).
  - BLAKE2b-256 vs BLAKE2s-256 performance comparison TBD.
  - SHA-256 remains faster than BLAKE3 even at 16 MiB, though the gap narrows as size increases.
