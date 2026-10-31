//! Bit patterns of every float constant used by the kernels.
//!
//! Constants are passed to the device as integer bit patterns and bit-cast to `f32` there, so a
//! shader compiler that prints float literals in decimal cannot silently change them.

/// log2(e) rounded to f32 (1.44269502).
pub const LOG2E_BITS: u32 = 0x3fb8_aa3b;
/// 1.5 * 2^23: adding and subtracting it rounds an f32 in [-2^22, 2^22] to the nearest integer.
pub const SHIFTER_BITS: u32 = 0x4b40_0000;
/// Cody-Waite high part of ln 2 (Cephes): 0.693359375, only 9 significant bits.
pub const C1_BITS: u32 = 0x3f31_8000;
/// Cody-Waite low part (Cephes): -2.12194440e-4.
pub const C2_BITS: u32 = 0xb95e_8083;
/// ln 2 rounded to f32, for the "no Cody-Waite split" mutant.
pub const LN2_BITS: u32 = 0x3f31_7218;
/// Cephes expf polynomial, highest order first:
/// 1.9875691500e-4, 1.3981999507e-3, 8.3334519073e-3, 4.1665795894e-2, 1.6666665459e-1,
/// 5.0000001201e-1.
pub const P_BITS: [u32; 6] = [
    0x3950_6967,
    0x3ab7_43ce,
    0x3c08_8908,
    0x3d2a_a9c1,
    0x3e2a_aaaa,
    0x3f00_0000,
];
pub const ONE_BITS: u32 = 0x3f80_0000;
/// Mutant: 1 - 2^-24 instead of 1.
pub const ONE_MINUS_BITS: u32 = 0x3f7f_ffff;
pub const TWO_M100_BITS: u32 = (127 - 100) << 23;
/// Overflow threshold: largest f32 whose exp is finite is just below this (88.7228394).
pub const OVF_BITS: u32 = 0x42b1_7218;
/// Mutant thresholds.
pub const OVF_HI_BITS: u32 = 0x42b1_999a; // 88.8
pub const OVF_LO_BITS: u32 = 0x42b1_6666; // 88.7
/// Underflow threshold: below this exp rounds to +0 (-103.972076).
pub const UNF_BITS: u32 = 0xc2cf_f1b4;
pub const UNF_FTZ_BITS: u32 = 0xc2ae_ac4f; // -87.33654: flushes subnormal results
pub const UNF_LO_BITS: u32 = 0xc2d1_0000; // -104.5

pub const F32_NAN_MARK: u32 = 0x7fc0_0000;
pub const F32_INF: u32 = 0x7f80_0000;

/// Patterns per device launch (2^25; llvmpipe caps a buffer at 128 MiB) and number of launches covering 2^32.
pub const CHUNK_LOG2: u32 = 25;
pub const CHUNK: u64 = 1 << CHUNK_LOG2;
pub const N_CHUNKS: u32 = 1 << (32 - CHUNK_LOG2);

/// Statistics record per device thread, 20 u32:
/// 0 max_ulp, 1 argmax (first input reaching max), 2 nan-ness mismatches, 3 first such input,
/// 4 NaN payload differences (both NaN, bits differ), 5 first such input,
/// 6..14 ULP buckets [0, 1, 2, 3, 4-15, 16-255, 256-65535, >=65536], 14 hash1, 15 hash2,
/// 16 max_ulp where the expected value is not subnormal, 17 its argmax,
/// 18 max_ulp where the expected value is subnormal, 19 its argmax,
/// 20 number of inputs evaluated (coverage check: a launch that silently failed reads as 0).
pub const STATS_W: usize = 21;
pub const NONE: u32 = u32::MAX;

/// Kernel kinds.
pub const KIND_BF16_FAST: u32 = 0;
pub const KIND_BF16_NATIVE: u32 = 1;
pub const KIND_EXPF: u32 = 2;
