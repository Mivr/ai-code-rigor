//! CubeCL kernels. Every function here has a plain-Rust twin in `host.rs`; the two must agree
//! bit for bit (checked by the `diff` command).
//!
//! Values are produced by assigning a pre-initialised mutable inside `if`/`else` because that is
//! the form `#[cube]` translates reliably; rustc sees the initial value as dead.
#![allow(unused_assignments)]

use crate::consts::*;
use cubecl::prelude::*;
use half::bf16;

#[cube]
fn fmix32(h0: u32) -> u32 {
    let mut h = h0;
    h ^= h >> 16u32;
    h *= 0x85eb_ca6bu32;
    h ^= h >> 13u32;
    h *= 0xc2b2_ae35u32;
    h ^= h >> 16u32;
    h
}

#[cube]
fn ordu(b: u32) -> u32 {
    let mut r = 0u32;
    if b >= 0x8000_0000u32 {
        r = 0x8000_0000u32 - (b & 0x7fff_ffffu32);
    } else {
        r = 0x8000_0000u32 + b;
    }
    r
}

// ----------------------------------------- kernel A ------------------------------------------

#[cube]
pub fn bf16_fast(b: u32) -> u32 {
    let mut r = 0u32;
    if (b & 0x7fff_ffffu32) > 0x7f80_0000u32 {
        r = (b >> 16u32) | 0x40u32;
    } else {
        r = (b + 0x7fffu32 + ((b >> 16u32) & 1u32)) >> 16u32;
    }
    r
}

/// Device port of `host::bf16_spec` (used only for the on-device comparison).
#[cube]
pub fn bf16_spec(b: u32) -> u32 {
    let exp = (b >> 23u32) & 0xffu32;
    let man = b & 0x7f_ffffu32;
    let upper = b >> 16u32;
    let lower = b & 0xffffu32;
    let mut r = 0u32;
    if exp == 0xffu32 {
        if man != 0u32 {
            r = upper | 0x40u32;
        } else {
            r = upper;
        }
    } else {
        let mut up = 0u32;
        if lower > 0x8000u32 {
            up = 1u32;
        }
        if lower == 0x8000u32 {
            up = upper & 1u32;
        }
        r = upper + up;
    }
    r
}

/// CubeCL's own f32 -> bf16 conversion (emulated in software on backends without bf16).
#[cube]
pub fn bf16_native(b: u32) -> u32 {
    let x = f32::reinterpret(b);
    let h = bf16::cast_from(x);
    u32::cast_from(u16::reinterpret(h))
}

// ----------------------------------------- kernel B ------------------------------------------

#[cube]
fn fb(bits: u32) -> f32 {
    f32::reinterpret(bits)
}

#[cube]
fn trunc_cast(v: f32) -> f32 {
    f32::cast_from(i32::cast_from(v))
}

#[cube]
fn floor_cast(v: f32) -> f32 {
    let t = trunc_cast(v);
    let mut r = t;
    if t > v {
        r = t - 1.0f32;
    }
    r
}

#[cube]
fn pow2(k: i32, bias: i32) -> f32 {
    f32::reinterpret(u32::cast_from(k + bias) << 23u32)
}

/// Fast expf with ~30 mutation sites selected at run time by `m` (0 = original).
#[cube]
pub fn expf_fast(b: u32, m: u32) -> u32 {
    let x = f32::reinterpret(b);
    let is_nan = (b & 0x7fff_ffffu32) > 0x7f80_0000u32;
    let ovf = fb(select(
        m == 19u32,
        OVF_LO_BITS,
        select(m == 18u32, OVF_HI_BITS, OVF_BITS),
    ));
    let unf = fb(select(
        m == 20u32,
        UNF_FTZ_BITS,
        select(m == 21u32, UNF_LO_BITS, UNF_BITS),
    ));
    let over = select(m == 22u32, x >= ovf, x > ovf);
    let under = x < unf;

    let xl = x * fb(LOG2E_BITS);
    let mut kf = f32::round(xl);
    if m == 26u32 {
        // the classic shifter trick; exact in IEEE arithmetic, but see RESULTS.md
        let sh = fb(SHIFTER_BITS);
        kf = (xl + sh) - sh;
    }
    if m == 1u32 {
        kf = trunc_cast(xl);
    }
    if m == 25u32 {
        kf = floor_cast(xl + 0.5f32);
    }
    let ki = i32::cast_from(kf);
    let kr = kf + select(m == 3u32, 1.0f32, 0.0f32);
    let c1 = fb(select(m == 4u32, C1_BITS + 1u32, C1_BITS));
    let c2 = fb(select(
        m == 7u32,
        C2_BITS + 1u32,
        select(m == 6u32, C2_BITS ^ 0x8000_0000u32, C2_BITS),
    ));
    let mut r = x - kr * c1;
    if m == 28u32 {
        r = x - kr * fb(LN2_BITS);
    } else if m == 32u32 {
        r = (x - kr * c2) - kr * c1;
    } else if m != 5u32 {
        r -= kr * c2;
    }
    let mut z = r * r;
    if m == 16u32 {
        z = r;
    }
    if m == 30u32 {
        z = r * x;
    }
    let p0 = fb(select(m == 8u32, P_BITS[0] + 1u32, P_BITS[0]));
    let p1 = fb(select(m == 9u32, P_BITS[1] + 1u32, P_BITS[1]));
    let p2 = fb(select(m == 10u32, P_BITS[2] + 1u32, P_BITS[2]));
    let p3 = fb(select(m == 11u32, P_BITS[3] + 1u32, P_BITS[3]));
    let p4 = fb(select(m == 12u32, P_BITS[4] + 1u32, P_BITS[4]));
    let p5 = fb(select(m == 13u32, P_BITS[5] + 1u32, P_BITS[5]));
    let p0e = select(m == 14u32, 0.0f32, p0);
    let p2e = select(m == 15u32, 0.0f32, p2);
    let mut y = p0e * r + p1;
    y = y * r + p2e;
    y = y * r + p3;
    y = y * r + p4;
    y = y * r + p5;
    let one = fb(select(m == 17u32, ONE_MINUS_BITS, ONE_BITS));
    let p = y * z + r + one;

    let k2 = ki + select(m == 2u32, 1i32, 0i32);
    let lim = select(m == 24u32, -126i32, select(m == 31u32, -127i32, -125i32));
    let bias = select(m == 27u32, 126i32, 127i32);
    let mut res = 0.0f32;
    if k2 > 127i32 {
        res = (p * pow2(k2 - 1i32, bias)) * 2.0f32;
    } else if k2 >= lim {
        res = p * pow2(k2, bias);
    } else {
        res = (p * pow2(k2 + 100i32, bias)) * fb(TWO_M100_BITS);
    }
    let mut out = u32::reinterpret(res);
    if m == 29u32 {
        out = u32::reinterpret(p) + (u32::cast_from(k2) << 23u32);
    }
    if over {
        out = F32_INF;
    }
    if under {
        out = 0u32;
    }
    if is_nan && m != 23u32 {
        out = b | 0x0040_0000u32;
    }
    out
}

// ------------------------------------------ drivers ------------------------------------------

#[cube]
fn eval(b: u32, m: u32, #[comptime] kind: u32) -> u32 {
    let mut v = 0u32;
    if comptime![kind == KIND_BF16_FAST] {
        v = bf16_fast(b);
    } else if comptime![kind == KIND_BF16_NATIVE] {
        v = bf16_native(b);
    } else {
        v = expf_fast(b, m);
    }
    v
}

/// Evaluate the kernel under test and write raw outputs (readback mode).
/// Thread t handles inputs base + t*ept + j, j < ept.
#[cube(launch_unchecked)]
pub fn out_kernel(out: &mut [u32], base: u32, ept: u32, m: u32, #[comptime] kind: u32) {
    let t = ABSOLUTE_POS as u32;
    for j in 0..ept {
        let i = t * ept + j;
        out[i as usize] = eval(base + i, m, kind);
    }
}

/// Evaluate and compare on the device; write one 16-word statistics record per thread.
/// Input of element j of thread t: base + (t*ept + j)*stride + offset.
/// Expected value: kernel A -> device port of the integer spec; kernel B -> golden table for
/// the non-trivial windows [lo_p, hi_p], [lo_n, hi_n], constant rules outside them.
#[cube(launch_unchecked)]
pub fn check_kernel(
    golden: &[u32],
    stats: &mut [u32],
    base: u32,
    ept: u32,
    stride: u32,
    offset: u32,
    lo_p: u32,
    hi_p: u32,
    lo_n: u32,
    hi_n: u32,
    m: u32,
    #[comptime] kind: u32,
) {
    let t = ABSOLUTE_POS as u32;
    let vshift = comptime![if kind == KIND_EXPF { 0u32 } else { 16u32 }];
    let mut maxu = 0u32;
    let mut argmax = 0xffff_ffffu32;
    let mut nanmis = 0u32;
    let mut nanfirst = 0xffff_ffffu32;
    let mut pay = 0u32;
    let mut payfirst = 0xffff_ffffu32;
    let mut b0 = 0u32;
    let mut b1 = 0u32;
    let mut b2 = 0u32;
    let mut b3 = 0u32;
    let mut b4 = 0u32;
    let mut b5 = 0u32;
    let mut b6 = 0u32;
    let mut b7 = 0u32;
    let mut h1 = 0u32;
    let mut h2 = 0u32;
    let mut maxn = 0u32;
    let mut argn = 0xffff_ffffu32;
    let mut maxs = 0u32;
    let mut args = 0xffff_ffffu32;
    let mut count = 0u32;
    for j in 0..ept {
        count += 1u32;
        let li = (t * ept + j) * stride + offset;
        let b = base + li;
        let v = eval(b, m, kind);
        let mut e = 0u32;
        if comptime![kind == KIND_EXPF] {
            if (b & 0x7fff_ffffu32) > 0x7f80_0000u32 {
                e = F32_NAN_MARK;
            } else if (b >= lo_p && b <= hi_p) || (b >= lo_n && b <= hi_n) {
                e = golden[li as usize];
            } else if b < 0x8000_0000u32 {
                e = select(b < lo_p, ONE_BITS, F32_INF);
            } else {
                e = select(b < lo_n, ONE_BITS, 0u32);
            }
        } else {
            e = bf16_spec(b);
        }
        let vv = v << vshift;
        let ee = e << vshift;
        // NaN flags as u32: CubeCL 0.11.3's SPIR-V compiler rejects `bool != bool`.
        let vn = select((vv & 0x7fff_ffffu32) > 0x7f80_0000u32, 1u32, 0u32);
        let en = select((ee & 0x7fff_ffffu32) > 0x7f80_0000u32, 1u32, 0u32);
        if (vn | en) != 0u32 {
            if vn != en {
                nanmis += 1u32;
                if nanfirst == 0xffff_ffffu32 {
                    nanfirst = b;
                }
            } else if vv != ee {
                pay += 1u32;
                if payfirst == 0xffff_ffffu32 {
                    payfirst = b;
                }
            }
        } else {
            let a = ordu(vv);
            let c = ordu(ee);
            let mut u = 0u32;
            if a > c {
                u = a - c;
            } else {
                u = c - a;
            }
            u >>= vshift;
            if u > maxu {
                maxu = u;
                argmax = b;
            }
            let ea = ee & 0x7fff_ffffu32;
            if ea != 0u32 && ea < 0x0080_0000u32 {
                if u > maxs {
                    maxs = u;
                    args = b;
                }
            } else if u > maxn {
                maxn = u;
                argn = b;
            }
            if u == 0u32 {
                b0 += 1u32;
            } else if u == 1u32 {
                b1 += 1u32;
            } else if u == 2u32 {
                b2 += 1u32;
            } else if u == 3u32 {
                b3 += 1u32;
            } else if u < 16u32 {
                b4 += 1u32;
            } else if u < 256u32 {
                b5 += 1u32;
            } else if u < 65536u32 {
                b6 += 1u32;
            } else {
                b7 += 1u32;
            }
        }
        h1 += fmix32(b ^ fmix32(v));
        h2 += fmix32(v + b * 0x9e37_79b9u32);
    }
    let o = t as usize * STATS_W;
    stats[o] = maxu;
    stats[o + 1] = argmax;
    stats[o + 2] = nanmis;
    stats[o + 3] = nanfirst;
    stats[o + 4] = pay;
    stats[o + 5] = payfirst;
    stats[o + 6] = b0;
    stats[o + 7] = b1;
    stats[o + 8] = b2;
    stats[o + 9] = b3;
    stats[o + 10] = b4;
    stats[o + 11] = b5;
    stats[o + 12] = b6;
    stats[o + 13] = b7;
    stats[o + 14] = h1;
    stats[o + 15] = h2;
    stats[o + 16] = maxn;
    stats[o + 17] = argn;
    stats[o + 18] = maxs;
    stats[o + 19] = args;
    stats[o + 20] = count;
}

/// Floating-point semantics probe: does the backend fold (a+b)-b, contract a*b-c into an FMA,
/// flush subnormals? See `fp-probe` in main.rs for the inputs and the IEEE-expected outputs.
#[cube(launch_unchecked)]
pub fn fp_probe(inp: &[f32], out: &mut [f32]) {
    if ABSOLUTE_POS == 0 {
        let x = inp[0];
        let c = inp[1];
        let a = inp[2];
        let z = inp[3];
        let s1 = inp[4];
        let s2 = inp[5];
        out[0] = (x + c) - c;
        out[1] = (x + fb(SHIFTER_BITS)) - fb(SHIFTER_BITS);
        out[2] = a * a - z;
        out[3] = s1 * s2;
        out[4] = s1 * 1.0f32;
        out[5] = f32::round(inp[6]);
        out[6] = f32::round(inp[7]);
        out[7] = f32::cast_from(i32::cast_from(inp[8]));
    }
}
