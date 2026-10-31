//! Plain-Rust versions of everything: the specs, the fast kernels (mirrored line by line from
//! `kernels.rs`), the comparison and the statistics. This is the "no GPU" status quo path and
//! the host side of every cross-check.

use crate::consts::*;

// ---------------------------------------------------------------------------------------------
// Kernel A: f32 -> bf16, round to nearest, ties to even.
// ---------------------------------------------------------------------------------------------

/// Fast implementation (the code under test): the usual bias-and-shift trick.
#[inline]
pub fn bf16_fast(b: u32) -> u32 {
    if (b & 0x7fff_ffff) > 0x7f80_0000 {
        (b >> 16) | 0x40
    } else {
        b.wrapping_add(0x7fff + ((b >> 16) & 1)) >> 16
    }
}

/// Spec: explicit case split, integer only. NaN keeps sign and the top payload bits and is made
/// quiet; infinities pass through; finite values keep the upper 16 bits and round up when the
/// discarded 16 bits exceed one half, or equal one half and the kept part is odd. A carry out of
/// the mantissa correctly bumps the exponent (and 0x7f7f + 1 = 0x7f80 is the RNE overflow to inf).
#[inline]
pub fn bf16_spec(b: u32) -> u32 {
    let exp = (b >> 23) & 0xff;
    let man = b & 0x7f_ffff;
    let upper = b >> 16;
    let lower = b & 0xffff;
    if exp == 0xff {
        if man != 0 {
            upper | 0x40
        } else {
            upper
        }
    } else {
        let round_up = lower > 0x8000 || (lower == 0x8000 && (upper & 1) == 1);
        upper + round_up as u32
    }
}

/// Independent semantic oracle used once to validate the spec: pick the nearer of the two
/// bf16 neighbours by exact f64 arithmetic, ties to even, with 2^128 standing in for the
/// overflow neighbour.
pub fn bf16_oracle(b: u32) -> u32 {
    let x = f32::from_bits(b);
    if x.is_nan() {
        return (b >> 16) | 0x40;
    }
    if x.is_infinite() {
        return b >> 16;
    }
    let t = b >> 16; // truncation toward zero, keeps sign
    let mag = |code: u32| -> f64 {
        if code & 0x7fff == 0x7f80 {
            2f64.powi(128)
        } else {
            (f32::from_bits((code & 0x7fff) << 16) as f64).abs()
        }
    };
    let ax = (x as f64).abs();
    let lo = mag(t);
    let hi = mag(t + 1);
    let dl = ax - lo;
    let dh = hi - ax;
    if dl < dh {
        t
    } else if dl > dh {
        t + 1
    } else if t & 1 == 0 {
        t
    } else {
        t + 1
    }
}

// ---------------------------------------------------------------------------------------------
// Kernel B: fast expf with mutant schemata. Must mirror kernels::expf_fast exactly.
// ---------------------------------------------------------------------------------------------

pub const N_MUTANTS: u32 = 32;

/// (id, description). Mutant 0 is the original.
pub const MUTANTS: [(u32, &str); 33] = [
    (0, "original"),
    (1, "range reduction: k = trunc(x*log2e) instead of round"),
    (2, "scaling: 2^(k+1) (off by one in exponent)"),
    (3, "range reduction: r uses k+1 (off by one), scaling uses k"),
    (4, "constant: C1 (ln2 high) +1 ulp"),
    (5, "Cody-Waite: drop the C2 (ln2 low) correction"),
    (6, "Cody-Waite: C2 sign flipped"),
    (7, "constant: C2 +1 ulp"),
    (8, "constant: P0 (r^7 coeff) +1 ulp"),
    (9, "constant: P1 (r^6 coeff) +1 ulp"),
    (10, "constant: P2 (r^5 coeff) +1 ulp"),
    (11, "constant: P3 (r^4 coeff) +1 ulp"),
    (12, "constant: P4 (r^3 coeff) +1 ulp"),
    (13, "constant: P5 (r^2 coeff, 0.5) +1 ulp"),
    (14, "polynomial: drop highest-order term (P0)"),
    (15, "polynomial: drop middle term (P2)"),
    (16, "polynomial: z = r instead of r*r"),
    (17, "constant: final +1 replaced by 1-2^-24"),
    (18, "overflow threshold 88.8 instead of 88.7228"),
    (19, "overflow threshold 88.7 instead of 88.7228"),
    (20, "underflow threshold -87.3365 (flushes subnormal results)"),
    (21, "underflow threshold -104.5 instead of -103.972"),
    (22, "overflow test x >= T instead of x > T"),
    (23, "NaN check removed"),
    (24, "subnormal-scaling branch boundary k >= -126 instead of -125"),
    (25, "rounding: k = floor(x*log2e + 0.5) (round half up) instead of round-to-nearest-even"),
    (26, "rounding via shifter trick (x*log2e + 1.5*2^23) - 1.5*2^23 instead of round()"),
    (27, "exponent bias 126 instead of 127"),
    (28, "range reduction: single ln2 constant, no Cody-Waite split"),
    (29, "scaling by integer add to exponent field (no subnormal handling)"),
    (30, "polynomial: z = r*x instead of r*r"),
    (31, "subnormal-scaling branch boundary k >= -127 instead of -125"),
    (32, "Cody-Waite: subtract C2 term before C1 term (operation order)"),
];

#[inline]
fn f(bits: u32) -> f32 {
    f32::from_bits(bits)
}
#[inline]
fn sel<T>(c: bool, a: T, b: T) -> T {
    if c {
        a
    } else {
        b
    }
}
/// trunc via int conversion (matches the device, which uses a cast).
#[inline]
fn trunc_cast(v: f32) -> f32 {
    (v as i32) as f32
}
#[inline]
fn floor_cast(v: f32) -> f32 {
    let t = trunc_cast(v);
    if t > v {
        t - 1.0
    } else {
        t
    }
}
#[inline]
fn pow2(k: i32, bias: i32) -> f32 {
    f32::from_bits(((k + bias) as u32) << 23)
}

#[inline]
pub fn expf_fast(b: u32, m: u32) -> u32 {
    let x = f32::from_bits(b);
    let is_nan = (b & 0x7fff_ffff) > 0x7f80_0000;
    let ovf = f(sel(m == 19, OVF_LO_BITS, sel(m == 18, OVF_HI_BITS, OVF_BITS)));
    let unf = f(sel(m == 20, UNF_FTZ_BITS, sel(m == 21, UNF_LO_BITS, UNF_BITS)));
    let over = sel(m == 22, x >= ovf, x > ovf);
    let under = x < unf;

    let xl = x * f(LOG2E_BITS);
    let mut kf = xl.round_ties_even();
    if m == 26 {
        let sh = f(SHIFTER_BITS);
        kf = (xl + sh) - sh;
    }
    if m == 1 {
        kf = trunc_cast(xl);
    }
    if m == 25 {
        kf = floor_cast(xl + 0.5);
    }
    let ki = kf as i32;
    let kr = kf + sel(m == 3, 1.0f32, 0.0f32);
    let c1 = f(sel(m == 4, C1_BITS + 1, C1_BITS));
    let c2 = f(sel(
        m == 7,
        C2_BITS + 1,
        sel(m == 6, C2_BITS ^ 0x8000_0000, C2_BITS),
    ));
    let mut r = x - kr * c1;
    if m == 28 {
        r = x - kr * f(LN2_BITS);
    } else if m == 32 {
        r = (x - kr * c2) - kr * c1;
    } else if m != 5 {
        r = r - kr * c2;
    }
    let mut z = r * r;
    if m == 16 {
        z = r;
    }
    if m == 30 {
        z = r * x;
    }
    let p0 = f(sel(m == 8, P_BITS[0] + 1, P_BITS[0]));
    let p1 = f(sel(m == 9, P_BITS[1] + 1, P_BITS[1]));
    let p2 = f(sel(m == 10, P_BITS[2] + 1, P_BITS[2]));
    let p3 = f(sel(m == 11, P_BITS[3] + 1, P_BITS[3]));
    let p4 = f(sel(m == 12, P_BITS[4] + 1, P_BITS[4]));
    let p5 = f(sel(m == 13, P_BITS[5] + 1, P_BITS[5]));
    let p0e = sel(m == 14, 0.0f32, p0);
    let p2e = sel(m == 15, 0.0f32, p2);
    let mut y = p0e * r + p1;
    y = y * r + p2e;
    y = y * r + p3;
    y = y * r + p4;
    y = y * r + p5;
    let one = f(sel(m == 17, ONE_MINUS_BITS, ONE_BITS));
    let p = y * z + r + one;

    let k2 = ki + sel(m == 2, 1i32, 0i32);
    let lim = sel(m == 24, -126i32, sel(m == 31, -127i32, -125i32));
    let bias = sel(m == 27, 126i32, 127i32);
    let res = if k2 > 127 {
        (p * pow2(k2 - 1, bias)) * 2.0
    } else if k2 >= lim {
        p * pow2(k2, bias)
    } else {
        (p * pow2(k2 + 100, bias)) * f(TWO_M100_BITS)
    };
    let mut out = res.to_bits();
    if m == 29 {
        out = p.to_bits().wrapping_add((k2 as u32) << 23);
    }
    if over {
        out = F32_INF;
    }
    if under {
        out = 0;
    }
    if is_nan && m != 23 {
        out = b | 0x0040_0000;
    }
    out
}

/// Spec for kernel B: CORE-MATH expf (correctly rounded, round-to-nearest).
#[inline]
pub fn expf_spec(b: u32) -> u32 {
    core_math::expf(f32::from_bits(b)).to_bits()
}

/// Alternative reference: f64 exp (system libm) rounded to f32. Near-correctly-rounded only.
#[inline]
pub fn expf_via_f64(b: u32) -> u32 {
    ((f32::from_bits(b) as f64).exp() as f32).to_bits()
}

/// System libm expf (glibc on Linux).
#[inline]
pub fn expf_libm(b: u32) -> u32 {
    f32::from_bits(b).exp().to_bits()
}

// ---------------------------------------------------------------------------------------------
// Comparison, statistics and hashing (mirrored in kernels.rs).
// ---------------------------------------------------------------------------------------------

#[inline]
pub fn fmix32(mut h: u32) -> u32 {
    h ^= h >> 16;
    h = h.wrapping_mul(0x85eb_ca6b);
    h ^= h >> 13;
    h = h.wrapping_mul(0xc2b2_ae35);
    h ^= h >> 16;
    h
}

#[inline]
pub fn hash_pair(b: u32, v: u32) -> (u32, u32) {
    (
        fmix32(b ^ fmix32(v)),
        fmix32(v.wrapping_add(b.wrapping_mul(0x9e37_79b9))),
    )
}

#[inline]
pub fn ordu(b: u32) -> u32 {
    if b >= 0x8000_0000 {
        0x8000_0000u32.wrapping_sub(b & 0x7fff_ffff)
    } else {
        0x8000_0000u32.wrapping_add(b)
    }
}

#[inline]
fn is_nan32(v: u32) -> bool {
    (v & 0x7fff_ffff) > 0x7f80_0000
}

pub type Stats = [u32; STATS_W];

pub fn stats_new() -> Stats {
    let mut s = [0u32; STATS_W];
    s[1] = NONE;
    s[3] = NONE;
    s[5] = NONE;
    s[17] = NONE;
    s[19] = NONE;
    s
}

#[inline]
fn is_sub32(v: u32) -> bool {
    let a = v & 0x7fff_ffff;
    a != 0 && a < 0x0080_0000
}

/// Accumulate one comparison. `vshift` is 16 for bf16 values (compared in f32 bit space, ULP
/// reported in bf16 units) and 0 for f32.
#[inline]
pub fn stats_add(s: &mut Stats, b: u32, v: u32, e: u32, vshift: u32) {
    let vv = v << vshift;
    let ee = e << vshift;
    let vn = is_nan32(vv);
    let en = is_nan32(ee);
    if vn || en {
        if vn != en {
            s[2] += 1;
            if s[3] == NONE {
                s[3] = b;
            }
        } else if vv != ee {
            s[4] += 1;
            if s[5] == NONE {
                s[5] = b;
            }
        }
    } else {
        let a = ordu(vv);
        let c = ordu(ee);
        let u = (if a > c { a - c } else { c - a }) >> vshift;
        if u > s[0] {
            s[0] = u;
            s[1] = b;
        }
        let slot = if is_sub32(ee) { 18 } else { 16 };
        if u > s[slot] {
            s[slot] = u;
            s[slot + 1] = b;
        }
        let k = match u {
            0 => 6,
            1 => 7,
            2 => 8,
            3 => 9,
            4..=15 => 10,
            16..=255 => 11,
            256..=65535 => 12,
            _ => 13,
        };
        s[k] += 1;
    }
    s[20] += 1;
    let (h1, h2) = hash_pair(b, v);
    s[14] = s[14].wrapping_add(h1);
    s[15] = s[15].wrapping_add(h2);
}

/// Merge `o` into `s`, where `o` covers inputs after those of `s` (keeps "first" semantics).
pub fn stats_merge(s: &mut Stats, o: &[u32]) {
    // argmax is set only on a strict increase above 0, so it stays NONE while max_ulp == 0
    for slot in [0usize, 16, 18] {
        if o[slot] > s[slot] {
            s[slot] = o[slot];
            s[slot + 1] = o[slot + 1];
        }
    }
    for (cnt, first) in [(2usize, 3usize), (4, 5)] {
        s[cnt] += o[cnt];
        if o[first] < s[first] {
            s[first] = o[first];
        }
    }
    for k in 6..14 {
        s[k] += o[k];
    }
    s[14] = s[14].wrapping_add(o[14]);
    s[15] = s[15].wrapping_add(o[15]);
    s[20] = s[20].wrapping_add(o[20]); // 2^32 wraps to 0: callers compare in u64 via `covered`
}

pub fn stats_reduce<'a>(parts: impl IntoIterator<Item = &'a [u32]>) -> Stats {
    let mut s = stats_new();
    for p in parts {
        stats_merge(&mut s, p);
    }
    s
}

pub fn stats_total(s: &Stats) -> u64 {
    s[6..14].iter().map(|&x| x as u64).sum::<u64>() + s[2] as u64 + s[4] as u64
}

pub fn fmt_stats(s: &Stats) -> String {
    let hx = |v: u32| {
        if v == NONE {
            "-".to_string()
        } else {
            format!("{v:#010x}")
        }
    };
    format!(
        "max_ulp={} argmax={} max_ulp_normal={} argmax_normal={} max_ulp_subnormal={} argmax_subnormal={} nan_mismatch={} first_nan_mismatch={} nan_payload_diff={} first_payload_diff={} buckets[0,1,2,3,4-15,16-255,256-65535,>=65536]={:?} hash={:08x}{:08x} n={}",
        s[0],
        hx(s[1]),
        s[16],
        hx(s[17]),
        s[18],
        hx(s[19]),
        s[2],
        hx(s[3]),
        s[4],
        hx(s[5]),
        &s[6..14],
        s[14],
        s[15],
        stats_total(s)
    )
}
