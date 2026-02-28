pub fn hash<D: digest::Digest>(data: &[u8]) -> digest::Output<D> {
    let mut hasher = D::new();
    hasher.update(data);
    hasher.finalize()
}

/// Extremely simple 64-bit XOR fold hash.
///
/// This is intended as a lightweight upper-bound style baseline for throughput experiments.
pub fn xor_hash64(data: &[u8]) -> u64 {
    use std::ptr;

    let mut i = 0usize;
    let len = data.len();
    let mut acc0 = 0u64;
    let mut acc1 = 0u64;
    let mut acc2 = 0u64;
    let mut acc3 = 0u64;

    // SAFETY: all unaligned reads are guarded by explicit bounds checks.
    unsafe {
        while i + 32 <= len {
            let p = data.as_ptr().add(i);
            acc0 ^= u64::from_le(ptr::read_unaligned(p as *const u64));
            acc1 ^= u64::from_le(ptr::read_unaligned(p.add(8) as *const u64));
            acc2 ^= u64::from_le(ptr::read_unaligned(p.add(16) as *const u64));
            acc3 ^= u64::from_le(ptr::read_unaligned(p.add(24) as *const u64));
            i += 32;
        }
    }

    let mut acc = acc0 ^ acc1 ^ acc2 ^ acc3;

    // SAFETY: bounds-checked before each unaligned 8-byte read.
    unsafe {
        while i + 8 <= len {
            acc ^= u64::from_le(ptr::read_unaligned(data.as_ptr().add(i) as *const u64));
            i += 8;
        }
    }

    if i < len {
        let mut tail = [0u8; 8];
        tail[..(len - i)].copy_from_slice(&data[i..]);
        acc ^= u64::from_le_bytes(tail);
    }

    acc
}

/// 128-bit XOR fold hash optimized as a throughput baseline.
///
/// Uses SIMD XOR with multiple independent accumulators on modern x86_64/aarch64 targets.
pub fn xor_hash128(data: &[u8]) -> u128 {
    // Use AVX2 when available on this build target.
    #[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
    return xor_hash128_avx2(data);

    // SSE2 is baseline on x86_64.
    #[cfg(all(target_arch = "x86_64", not(target_feature = "avx2")))]
    return xor_hash128_sse2(data);

    // NEON is baseline on aarch64.
    #[cfg(target_arch = "aarch64")]
    return xor_hash128_neon(data);

    #[cfg(not(any(target_arch = "x86_64", target_arch = "aarch64")))]
    compile_error!("xor_hash128 requires SIMD support on x86_64 or aarch64");
}

#[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
pub fn xor_hash128_avx2(data: &[u8]) -> u128 {
    use std::arch::x86_64::{
        __m128i, __m256i, _mm_loadu_si128, _mm_storeu_si128, _mm_xor_si128, _mm256_castsi256_si128,
        _mm256_extracti128_si256, _mm256_loadu_si256, _mm256_setzero_si256, _mm256_xor_si256,
    };

    let mut i = 0usize;
    let mut out = [0u8; 16];

    // SAFETY: all pointer reads/writes are in-bounds for the checked ranges.
    unsafe {
        // Four independent accumulators reduce dependency chains and improve ILP.
        let mut acc0: __m256i = _mm256_setzero_si256();
        let mut acc1: __m256i = _mm256_setzero_si256();
        let mut acc2: __m256i = _mm256_setzero_si256();
        let mut acc3: __m256i = _mm256_setzero_si256();

        while i + 128 <= data.len() {
            let b0 = _mm256_loadu_si256(data.as_ptr().add(i) as *const __m256i);
            let b1 = _mm256_loadu_si256(data.as_ptr().add(i + 32) as *const __m256i);
            let b2 = _mm256_loadu_si256(data.as_ptr().add(i + 64) as *const __m256i);
            let b3 = _mm256_loadu_si256(data.as_ptr().add(i + 96) as *const __m256i);
            acc0 = _mm256_xor_si256(acc0, b0);
            acc1 = _mm256_xor_si256(acc1, b1);
            acc2 = _mm256_xor_si256(acc2, b2);
            acc3 = _mm256_xor_si256(acc3, b3);
            i += 128;
        }

        let mut acc = _mm256_xor_si256(_mm256_xor_si256(acc0, acc1), _mm256_xor_si256(acc2, acc3));
        while i + 32 <= data.len() {
            let block = _mm256_loadu_si256(data.as_ptr().add(i) as *const __m256i);
            acc = _mm256_xor_si256(acc, block);
            i += 32;
        }

        let lo = _mm256_castsi256_si128(acc);
        let hi = _mm256_extracti128_si256(acc, 1);
        let mut acc128 = _mm_xor_si128(lo, hi);
        while i + 16 <= data.len() {
            let block = _mm_loadu_si128(data.as_ptr().add(i) as *const __m128i);
            acc128 = _mm_xor_si128(acc128, block);
            i += 16;
        }
        _mm_storeu_si128(out.as_mut_ptr() as *mut __m128i, acc128);
    }

    if i < data.len() {
        for (idx, b) in data[i..].iter().enumerate() {
            out[idx] ^= *b;
        }
    }

    u128::from_le_bytes(out)
}

#[cfg(target_arch = "x86_64")]
pub fn xor_hash128_sse2(data: &[u8]) -> u128 {
    use std::arch::x86_64::{
        __m128i, _mm_loadu_si128, _mm_setzero_si128, _mm_storeu_si128, _mm_xor_si128,
    };

    let mut i = 0usize;
    let mut out = [0u8; 16];

    // SAFETY: all pointer reads/writes are in-bounds for the checked ranges.
    unsafe {
        // Four independent accumulators reduce dependency chains and improve ILP.
        let mut acc0 = _mm_setzero_si128();
        let mut acc1 = _mm_setzero_si128();
        let mut acc2 = _mm_setzero_si128();
        let mut acc3 = _mm_setzero_si128();

        while i + 64 <= data.len() {
            let b0 = _mm_loadu_si128(data.as_ptr().add(i) as *const __m128i);
            let b1 = _mm_loadu_si128(data.as_ptr().add(i + 16) as *const __m128i);
            let b2 = _mm_loadu_si128(data.as_ptr().add(i + 32) as *const __m128i);
            let b3 = _mm_loadu_si128(data.as_ptr().add(i + 48) as *const __m128i);
            acc0 = _mm_xor_si128(acc0, b0);
            acc1 = _mm_xor_si128(acc1, b1);
            acc2 = _mm_xor_si128(acc2, b2);
            acc3 = _mm_xor_si128(acc3, b3);
            i += 64;
        }

        let mut acc = _mm_xor_si128(_mm_xor_si128(acc0, acc1), _mm_xor_si128(acc2, acc3));
        while i + 16 <= data.len() {
            let block = _mm_loadu_si128(data.as_ptr().add(i) as *const __m128i);
            acc = _mm_xor_si128(acc, block);
            i += 16;
        }
        _mm_storeu_si128(out.as_mut_ptr() as *mut __m128i, acc);
    }

    if i < data.len() {
        for (idx, b) in data[i..].iter().enumerate() {
            out[idx] ^= *b;
        }
    }

    u128::from_le_bytes(out)
}

#[cfg(target_arch = "aarch64")]
pub fn xor_hash128_neon(data: &[u8]) -> u128 {
    use std::arch::aarch64::{uint8x16_t, vdupq_n_u8, veorq_u8, vld1q_u8, vst1q_u8};

    let mut i = 0usize;
    let mut out = [0u8; 16];

    // SAFETY: all pointer reads/writes are in-bounds for the checked ranges.
    unsafe {
        // Four independent accumulators reduce dependency chains and improve ILP.
        let mut acc0: uint8x16_t = vdupq_n_u8(0);
        let mut acc1: uint8x16_t = vdupq_n_u8(0);
        let mut acc2: uint8x16_t = vdupq_n_u8(0);
        let mut acc3: uint8x16_t = vdupq_n_u8(0);

        while i + 64 <= data.len() {
            let b0 = vld1q_u8(data.as_ptr().add(i));
            let b1 = vld1q_u8(data.as_ptr().add(i + 16));
            let b2 = vld1q_u8(data.as_ptr().add(i + 32));
            let b3 = vld1q_u8(data.as_ptr().add(i + 48));
            acc0 = veorq_u8(acc0, b0);
            acc1 = veorq_u8(acc1, b1);
            acc2 = veorq_u8(acc2, b2);
            acc3 = veorq_u8(acc3, b3);
            i += 64;
        }

        let mut acc = veorq_u8(veorq_u8(acc0, acc1), veorq_u8(acc2, acc3));
        while i + 16 <= data.len() {
            let block = vld1q_u8(data.as_ptr().add(i));
            acc = veorq_u8(acc, block);
            i += 16;
        }
        vst1q_u8(out.as_mut_ptr(), acc);
    }

    if i < data.len() {
        for (idx, b) in data[i..].iter().enumerate() {
            out[idx] ^= *b;
        }
    }

    u128::from_le_bytes(out)
}
