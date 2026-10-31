# Generated tables

Produced by `python3 scripts/summarize.py results/raw/linux results/raw/mac` from the logs in `results/raw/`. Curated discussion: [RESULTS.md](RESULTS.md).

## fp-probe

| case | cpu on AMD Ryzen 9 7950X 16-Core Processor | wgpu<spirv> on AMD Radeon RX 6700 XT (RADV NAVI22) | wgpu<wgsl> on AMD Radeon RX 6700 XT (RADV NAVI22) | wgpu<spirv> on llvmpipe (LLVM 21.1.8, 256 bits) | wgpu<wgsl> on llvmpipe (LLVM 21.1.8, 256 bits) | wgpu<wgsl> on Apple M2 |
|---|---|---|---|---|---|---|
| (x+c)-c with runtime c [IEEE: 0] | ok (0e0) | DIFFERS (3e-1) | DIFFERS (3e-1) | DIFFERS (3e-1) | DIFFERS (3e-1) | DIFFERS (3e-1) |
| (x+C)-C with literal C [IEEE: 0] | ok (0e0) | DIFFERS (3e-1) | DIFFERS (3e-1) | DIFFERS (3e-1) | DIFFERS (3e-1) | DIFFERS (3e-1) |
| a*a - fl(a*a) [IEEE: 0; FMA-contracted: 2^-24] | DIFFERS (5.9604645e-8) | DIFFERS (5.9604645e-8) | DIFFERS (5.9604645e-8) | ok (0e0) | ok (0e0) | DIFFERS (5.9604645e-8) |
| 2^-100 * 2^-30 [IEEE: 2^-130 subnormal; FTZ: 0] | ok (7.34684e-40) | DIFFERS (0e0) | DIFFERS (0e0) | ok (7.34684e-40) | ok (7.34684e-40) | DIFFERS (0e0) |
| 2^-100 * 1 [IEEE: 2^-100] | ok (7.888609e-31) | ok (7.888609e-31) | ok (7.888609e-31) | ok (7.888609e-31) | ok (7.888609e-31) | ok (7.888609e-31) |
| round(2.5) [ties-to-even: 2] | ok (2e0) | ok (2e0) | ok (2e0) | ok (2e0) | ok (2e0) | ok (2e0) |
| round(-0.5) [ties-to-even: -0] | ok (-0e0) | ok (-0e0) | ok (-0e0) | ok (-0e0) | ok (-0e0) | ok (-0e0) |
| f32(i32(-1.75)) [trunc: -1] | ok (-1e0) | ok (-1e0) | ok (-1e0) | ok (-1e0) | ok (-1e0) | ok (-1e0) |

## Kernel B special values (verdict vs CORE-MATH)

| input | cpu on AMD Ryzen 9 7950X 16-Core Processor | wgpu<spirv> on AMD Radeon RX 6700 XT (RADV NAVI22) | wgpu<wgsl> on AMD Radeon RX 6700 XT (RADV NAVI22) | wgpu<wgsl> on llvmpipe (LLVM 21.1.8, 256 bits) | rayon (32 threads) | wgpu<wgsl> on Apple M2 |
|---|---|---|---|---|---|---|
| +0 `0x00000000` | exact | exact | exact | exact | exact | exact |
| -0 `0x80000000` | exact | exact | exact | exact | exact | exact |
| +inf `0x7f800000` | exact | exact | exact | exact | exact | exact |
| -inf `0xff800000` | exact | exact | exact | exact | exact | exact |
| qNaN `0x7fc00000` | exact | exact | exact | exact | exact | exact |
| sNaN `0x7f800001` | exact | exact | exact | exact | exact | exact |
| -qNaN `0xffc00000` | exact | exact | exact | exact | exact | exact |
| 1 `0x3f800000` | exact | exact | exact | exact | exact | exact |
| -1 `0xbf800000` | exact | exact | exact | exact | exact | exact |
| 2^-25 (exp rounds to 1) `0x33000000` | exact | exact | exact | exact | exact | exact |
| 2^-24 (first input with exp > 1) `0x33800000` | 1 ulp (got 0x3f800000, spec 0x3f800001) | 1 ulp (got 0x3f800000, spec 0x3f800001) | 1 ulp (got 0x3f800000, spec 0x3f800001) | 1 ulp (got 0x3f800000, spec 0x3f800001) | 1 ulp (got 0x3f800000, spec 0x3f800001) | 1 ulp (got 0x3f800000, spec 0x3f800001) |
| -2^-25 (exp rounds to 1) `0xb3000000` | exact | exact | exact | exact | exact | exact |
| -2^-25-ulp (first input with exp < 1) `0xb3000001` | exact | exact | exact | exact | exact | exact |
| largest input with finite exp (88.7228) `0x42b17217` | exact | exact | exact | exact | exact | 124 ulp (got 0x7f800000, spec 0x7f7fff84) |
| first input with exp = +inf `0x42b17218` | exact | exact | exact | exact | exact | exact |
| 88.3763 (k = 128 path) `0x42b0c0a6` | 1 ulp (got 0x7f3504fe, spec 0x7f3504ff) | 1 ulp (got 0x7f3504fe, spec 0x7f3504ff) | 1 ulp (got 0x7f3504fe, spec 0x7f3504ff) | 1 ulp (got 0x7f3504fe, spec 0x7f3504ff) | 1 ulp (got 0x7f3504fe, spec 0x7f3504ff) | 4913921 ulp (got 0x7f800000, spec 0x7f3504ff) |
| -87.3365 (exp just above FLT_MIN) `0xc2aeac4f` | exact | exact | exact | exact | exact | exact |
| -87.3365-ulp (exp subnormal) `0xc2aeac50` | exact | 8388582 ulp (got 0x00000000, spec 0x007fffe6) | 8388582 ulp (got 0x00000000, spec 0x007fffe6) | exact | exact | 8388582 ulp (got 0x00000000, spec 0x007fffe6) |
| -100 (deep subnormal) `0xc2c80000` | exact | 27 ulp (got 0x00000000, spec 0x0000001b) | 27 ulp (got 0x00000000, spec 0x0000001b) | exact | exact | 27 ulp (got 0x00000000, spec 0x0000001b) |
| most negative input with exp > 0 (-103.97208) `0xc2cff1b4` | exact | 1 ulp (got 0x00000000, spec 0x00000001) | 1 ulp (got 0x00000000, spec 0x00000001) | exact | exact | 1 ulp (got 0x00000000, spec 0x00000001) |
| first negative input with exp = 0 `0xc2cff1b5` | exact | exact | exact | exact | exact | exact |
| -104 `0xc2d00000` | exact | exact | exact | exact | exact | exact |

## spec validation

- spec_validate_bf16 spec_vs_semantic_oracle_mismatches=0 secs=0.494
- spec_validate_expf ref=f64_exp_rounded vs=core_math secs=2.618 max_ulp=0 argmax=- max_ulp_normal=0 argmax_normal=- max_ulp_subnormal=0 argmax_subnormal=- nan_mismatch=0 first_nan_mismatch=- nan_payload_diff=0 first_payload_diff=- buckets[0,1,2,3,4-15,16-255,256-65535,>=65536]=[4278190082, 0, 0, 0, 0, 0, 0, 0] hash=ee71bf0b3ddbc48f n=4278190082
- spec_validate_expf ref=system_libm_expf vs=core_math secs=2.366 max_ulp=1 argmax=0x37ff7f01 max_ulp_normal=1 argmax_normal=0x37ff7f01 max_ulp_subnormal=1 argmax_subnormal=0xc2aeb433 nan_mismatch=0 first_nan_mismatch=- nan_payload_diff=0 first_payload_diff=- buckets[0,1,2,3,4-15,16-255,256-65535,>=65536]=[4278019434, 170648, 0, 0, 0, 0, 0, 0] hash=44b9a1eb0b2cf49e n=4278190082

## Kernel A check

| path | backend | runs (s) | median (s) | JIT (s) | setup (s) | max ULP (normal / subnormal) | hash | identical across reps |
|---|---|---|---|---|---|---|---|---|
| a-check-cpu | cpu on AMD Ryzen 9 7950X 16-Core Processor | 5.373, 4.916, 4.583 | 4.916 | 0.0274 |  | 0 / 0 | 95794923f73805f6 | yes |
| a-check-gpu-spirv | wgpu<spirv> on AMD Radeon RX 6700 XT (RADV NAVI22) | 0.112, 0.110, 0.112 | 0.112 | 0.0144 |  | 0 / 0 | 95794923f73805f6 | yes |
| a-check-gpu-wgsl | wgpu<wgsl> on AMD Radeon RX 6700 XT (RADV NAVI22) | 0.108, 0.111, 0.111 | 0.111 | 0.0324 |  | 0 / 0 | 95794923f73805f6 | yes |
| a-check-llvmpipe-spirv | wgpu<spirv> on llvmpipe (LLVM 21.1.8, 256 bits) | 0.604, 0.455, 0.445 | 0.455 | 0.0312 |  | 0 / 0 | 95794923f73805f6 | yes |
| a-check-llvmpipe-wgsl | wgpu<wgsl> on llvmpipe (LLVM 21.1.8, 256 bits) | 0.763, 0.430, 0.430 | 0.430 | 0.0543 |  | 0 / 0 | 95794923f73805f6 | yes |
| a-check-rayon | rayon (32 threads) | 1.629, 1.555, 1.584 | 1.584 |  |  | 0 / 0 | 95794923f73805f6 | yes |
| a-check-rayon (minimal loop) | rayon | 0.273, 0.278, 0.282 | 0.278 | | | mismatches=0 | | |
| a-check-metal | wgpu<wgsl> on Apple M2 | 0.412, 0.408, 0.390 | 0.408 | 0.0740 |  | 0 / 0 | 95794923f73805f6 | yes |
| a-check-rayon-mac | rayon (8 threads) | 3.768, 3.741, 3.783 | 3.768 |  |  | 0 / 0 | 95794923f73805f6 | yes |
| a-check-rayon-mac (minimal loop) | rayon | 1.026, 1.025, 1.028 | 1.026 | | | mismatches=0 | | |

## Kernel A native check

| path | backend | runs (s) | median (s) | JIT (s) | setup (s) | max ULP (normal / subnormal) | hash | identical across reps |
|---|---|---|---|---|---|---|---|---|
| a-native-check-cpu | cpu on AMD Ryzen 9 7950X 16-Core Processor | 4.865, 3.550 | 4.207 | 0.0259 |  | 128 / 127 | bbd266d18cedeaa9 | yes |
| a-native-check-gpu | wgpu<wgsl> on AMD Radeon RX 6700 XT (RADV NAVI22) | failed: thread 'DSD-0-0' panicked at <registry>/cubecl-wgpu-0.11.3/src/compiler/wgsl/to_ | | | | | | |

## Kernel B check

| path | backend | runs (s) | median (s) | JIT (s) | setup (s) | max ULP (normal / subnormal) | hash | identical across reps |
|---|---|---|---|---|---|---|---|---|
| b-check-cpu | cpu on AMD Ryzen 9 7950X 16-Core Processor | 9.490, 9.870, 7.529 | 9.490 | 0.0362 | golden 0.76 + upload 4.065 | 1 / 1 | a3bc4f8d47190450 | yes |
| b-check-gpu-spirv | wgpu<spirv> on AMD Radeon RX 6700 XT (RADV NAVI22) | 0.445, 0.446, 0.435 | 0.445 | 0.0216 | golden 0.69 + upload 3.499 | 1 / 8388582 | 02f592ab97a45af1 | yes |
| b-check-gpu-wgsl | wgpu<wgsl> on AMD Radeon RX 6700 XT (RADV NAVI22) | 0.449, 0.464, 0.461 | 0.461 | 0.0140 | golden 0.79 + upload 3.494 | 1 / 8388582 | 02f592ab97a45af1 | yes |
| b-check-llvmpipe-spirv | wgpu<spirv> on llvmpipe (LLVM 21.1.8, 256 bits) | 1.376, 1.256, 1.259 | 1.259 | 0.0414 | golden 0.68 + upload 5.436 | 1 / 1 | e3d54898337e0d71 | yes |
| b-check-llvmpipe-wgsl | wgpu<wgsl> on llvmpipe (LLVM 21.1.8, 256 bits) | 1.562, 1.317, 1.316 | 1.317 | 0.0378 | golden 0.75 + upload 5.402 | 1 / 1 | e3d54898337e0d71 | yes |
| b-check-rayon | rayon (32 threads) | 3.769, 3.757, 4.175 | 3.769 |  | golden 0.67 | 1 / 1 | e3d54898337e0d71 | yes |
| b-check-rayon (minimal loop) | rayon | 1.689, 1.799, 1.757 | 1.757 | | | max_ulp=1 | | |
| b-check-metal | wgpu<wgsl> on Apple M2 | 0.668, 0.653, 0.650 | 0.653 | 0.3934 | golden 2.25 + upload 2.526 | 4913921 / 8388582 | 8543a325ee7c54c9 | yes |
| b-check-rayon-mac | rayon (8 threads) | 13.955, 14.027, 14.076 | 14.027 |  | golden 2.20 | 1 / 1 | e3d54898337e0d71 | yes |
| b-check-rayon-mac (minimal loop) | rayon | 4.670, 4.679, 4.665 | 4.670 | | | max_ulp=1 | | |

## Kernel B shifter check

| path | backend | runs (s) | median (s) | JIT (s) | setup (s) | max ULP (normal / subnormal) | hash | identical across reps |
|---|---|---|---|---|---|---|---|---|
| b-shifter-check-cpu | cpu on AMD Ryzen 9 7950X 16-Core Processor | 5.096 | 5.096 | 0.0358 | golden 0.74 + upload 4.090 | 1 / 1 | a3bc4f8d47190450 | yes |
| b-shifter-check-gpu | wgpu<wgsl> on AMD Radeon RX 6700 XT (RADV NAVI22) | 0.444 | 0.444 | 0.0108 | golden 0.71 + upload 3.483 | 8388608 / 8388582 | f3f52c1b312d69b6 | yes |
| b-shifter-check-llvmpipe | wgpu<wgsl> on llvmpipe (LLVM 21.1.8, 256 bits) | 1.607 | 1.607 | 0.0112 | golden 0.74 + upload 5.367 | 8388608 / 4194265 | f94a0243a02b729c | yes |
| b-shifter-check-rayon | rayon (32 threads) | 3.548 | 3.548 |  | golden 0.80 | 1 / 1 | e3d54898337e0d71 | yes |
| b-shifter-check-metal | wgpu<wgsl> on Apple M2 | 0.665 | 0.665 | 0.0198 | golden 2.28 + upload 1.988 | 8388608 / 8388582 | f3f52c1b312d69b6 | yes |

## Kernel A diff

| diff | A | B | differing outputs | by class of B [zero, sub, normal, inf, NaN] | max ULP (normal / subnormal) | A time (s) | B time (s) | compare (s) | total (s) |
|---|---|---|---|---|---|---|---|---|---|
| a-diff-cpu-spec | cpu on AMD Ryzen 9 7950X 16-Core Processor | spec (host) | 0 | [0, 0, 0, 0, 0] | 0 / 0 | 7.102 | 0.819 | 1.588 | 11.702 |
| a-diff-gpu-spec | wgpu<wgsl> on AMD Radeon RX 6700 XT (RADV NAVI22) | spec (host) | 0 | [0, 0, 0, 0, 0] | 0 / 0 | 13.281 | 0.821 | 1.601 | 17.903 |
| a-diff-llvmpipe-spec | wgpu<wgsl> on llvmpipe (LLVM 21.1.8, 256 bits) | spec (host) | 0 | [0, 0, 0, 0, 0] | 0 / 0 | 12.798 | 0.802 | 1.585 | 17.356 |
| a-diff-rayon-spec | rayon (32 threads) | spec (host) | 0 | [0, 0, 0, 0, 0] | 0 / 0 | 0.835 | 0.905 | 1.629 | 5.816 |
| a-diff-metal-spec | wgpu<wgsl> on Apple M2 | spec (host) | 0 | [0, 0, 0, 0, 0] | 0 / 0 | 2.184 | 0.947 | 3.545 | 6.677 |

## Kernel A native diff

| diff | A | B | differing outputs | by class of B [zero, sub, normal, inf, NaN] | max ULP (normal / subnormal) | A time (s) | B time (s) | compare (s) | total (s) |
|---|---|---|---|---|---|---|---|---|---|
| a-native-diff-cpu-spec | cpu on AMD Ryzen 9 7950X 16-Core Processor | spec (host) | 16711678 | [0, 16646142, 65536, 0, 0] | 128 / 127 | 7.058 | 0.813 | 1.565 | 11.603 |

## Kernel B diff

| diff | A | B | differing outputs | by class of B [zero, sub, normal, inf, NaN] | max ULP (normal / subnormal) | A time (s) | B time (s) | compare (s) | total (s) |
|---|---|---|---|---|---|---|---|---|---|
| b-diff-cpu-rayon | cpu on AMD Ryzen 9 7950X 16-Core Processor | rayon (32 threads) | 2078124 | [0, 1563, 2076561, 0, 0] | 1 / 1 | 7.692 | 3.063 | 1.690 | 14.669 |
| b-diff-cpu-spec | cpu on AMD Ryzen 9 7950X 16-Core Processor | spec (host) | 17240866 | [0, 45171, 17195695, 0, 0] | 1 / 1 | 7.852 | 1.062 | 1.713 | 12.912 |
| b-diff-gpu-cpu | wgpu<wgsl> on AMD Radeon RX 6700 XT (RADV NAVI22) | cpu on AMD Ryzen 9 7950X 16-Core Processor | 2180453 | [0, 2180453, 0, 0, 0] | 0 / 8388582 | 13.302 | 7.786 | 1.377 | 24.546 |
| b-diff-gpu-rayon | wgpu<wgsl> on AMD Radeon RX 6700 XT (RADV NAVI22) | rayon (32 threads) | 4257014 | [0, 2180453, 2076561, 0, 0] | 1 / 8388582 | 13.637 | 3.119 | 1.710 | 20.799 |
| b-diff-gpu-spec | wgpu<wgsl> on AMD Radeon RX 6700 XT (RADV NAVI22) | spec (host) | 19376148 | [0, 2180453, 17195695, 0, 0] | 1 / 8388582 | 13.390 | 1.004 | 1.633 | 18.231 |
| b-diff-llvmpipe-rayon | wgpu<wgsl> on llvmpipe (LLVM 21.1.8, 256 bits) | rayon (32 threads) | 0 | [0, 0, 0, 0, 0] | 0 / 0 | 13.684 | 3.035 | 1.683 | 20.683 |
| b-diff-rayon-spec | rayon (32 threads) | spec (host) | 18092448 | [0, 45182, 18047266, 0, 0] | 1 / 1 | 2.984 | 1.027 | 1.665 | 8.088 |
| b-diff-metal-spec | wgpu<wgsl> on Apple M2 | spec (host) | 19416450 | [0, 2180453, 17235997, 0, 0] | 4913921 / 8388582 | 2.198 | 1.976 | 3.475 | 7.649 |

## Mutants (exhaustive)

| id | description | mut-cpu: verdict (max ULP normal/sub, s) | mut-gpu-spirv: verdict (max ULP normal/sub, s) | mut-gpu-wgsl-rerun: verdict (max ULP normal/sub, s) | mut-gpu-wgsl: verdict (max ULP normal/sub, s) | mut-llvmpipe-wgsl: verdict (max ULP normal/sub, s) | mut-rayon: verdict (max ULP normal/sub, s) | mut-metal: verdict (max ULP normal/sub, s) |
|---|---|---|---|---|---|---|---|---|
| 0 | original | original (1/1, 11.29) | original (1/8388582, 0.47) | original (1/8388582, 0.44) | original (1/8388582, 0.45) | original (1/1, 1.65) | original (1/1, 3.76) | original (4913921/8388582, 0.66) |
| 1 | range reduction: k = trunc(x*log2e) instead of round | killed (14/4, 12.86) | killed (14/8388582, 0.44) | killed (14/8388582, 0.47) | killed (14/8388582, 0.44) | killed (14/4, 1.42) | killed (14/4, 3.64) | survived (14/8388582, 0.65) |
| 2 | scaling: 2^(k+1) (off by one in exponent) | killed (8388609/8388582, 13.60) | killed (8388609/8388582, 0.45) | killed (8388609/8388582, 0.47) | killed (8388609/8388582, 0.44) | killed (8388609/8388582, 1.44) | killed (8388609/8388582, 3.78) | killed (13302422/8388582, 0.65) |
| 3 | range reduction: r uses k+1 (off by one), scaling uses k | killed (8389287/4194294, 11.46) | killed (16777140/8388582, 0.44) | killed (16777140/8388582, 0.45) | killed (16777140/8388582, 0.45) | killed (8389288/4194294, 1.42) | killed (8389288/4194294, 3.44) | killed (16777140/8388582, 0.65) |
| 4 | constant: C1 (ln2 high) +1 ulp | killed (128/65, 12.25) | killed (128/8388518, 0.44) | killed (128/8388518, 0.45) | killed (128/8388518, 0.47) | killed (128/65, 1.40) | killed (128/65, 3.75) | survived (4913921/8388518, 0.65) |
| 5 | Cody-Waite: drop the C2 (ln2 low) correction | killed (449548/227306, 10.27) | killed (449548/8167235, 0.44) | killed (449548/8167235, 0.45) | killed (449548/8167235, 0.43) | killed (449548/227306, 1.45) | killed (449548/227306, 3.52) | survived (4913921/8167235, 0.65) |
| 6 | Cody-Waite: C2 sign flipped | killed (887053/460772, 11.10) | killed (887053/7951790, 0.44) | killed (887053/7951790, 0.44) | killed (887053/7951790, 0.46) | killed (887053/460772, 1.43) | killed (887053/460772, 3.76) | survived (4913921/7951790, 0.65) |
| 7 | constant: C2 +1 ulp | survived (1/1, 14.42) | survived (1/8388582, 0.44) | survived (1/8388582, 0.45) | survived (1/8388582, 0.44) | survived (1/1, 1.40) | survived (1/1, 3.77) | survived (4913921/8388582, 0.65) |
| 8 | constant: P0 (r^7 coeff) +1 ulp | survived= (1/1, 13.93) | survived= (1/8388582, 0.46) | survived= (1/8388582, 0.43) | survived= (1/8388582, 0.44) | survived (1/1, 1.45) | survived (1/1, 3.70) | survived= (4913921/8388582, 0.65) |
| 9 | constant: P1 (r^6 coeff) +1 ulp | survived (1/1, 9.33) | survived (1/8388582, 0.44) | survived (1/8388582, 0.46) | survived (1/8388582, 0.47) | survived (1/1, 1.40) | survived (1/1, 3.76) | survived (4913921/8388582, 0.65) |
| 10 | constant: P2 (r^5 coeff) +1 ulp | survived (1/1, 10.26) | survived (1/8388582, 0.45) | survived (1/8388582, 0.44) | survived (1/8388582, 0.44) | survived (1/1, 1.36) | survived (1/1, 3.73) | survived (4913921/8388582, 0.67) |
| 11 | constant: P3 (r^4 coeff) +1 ulp | survived (1/1, 10.82) | survived (1/8388582, 0.45) | survived (1/8388582, 0.44) | survived (1/8388582, 0.45) | survived (1/1, 1.35) | survived (1/1, 3.79) | survived (4913921/8388582, 0.65) |
| 12 | constant: P4 (r^3 coeff) +1 ulp | survived (1/1, 10.63) | survived (1/8388582, 0.45) | survived (1/8388582, 0.44) | survived (1/8388582, 0.45) | survived (1/1, 1.35) | survived (1/1, 3.75) | survived (4913921/8388582, 0.66) |
| 13 | constant: P5 (r^2 coeff, 0.5) +1 ulp | survived (1/1, 9.86) | survived (1/8388582, 0.47) | survived (1/8388582, 0.46) | survived (1/8388582, 0.44) | survived (1/1, 1.33) | survived (1/1, 3.98) | survived (4913921/8388582, 0.65) |
| 14 | polynomial: drop highest-order term (P0) | killed (3/2, 12.11) | killed (3/8388582, 0.44) | killed (3/8388582, 0.44) | killed (3/8388582, 0.44) | killed (3/2, 1.37) | killed (3/2, 3.78) | survived (4913921/8388582, 0.65) |
| 15 | polynomial: drop middle term (P2) | killed (700/350, 9.85) | killed (700/8388582, 0.44) | killed (700/8388582, 0.44) | killed (700/8388582, 0.47) | killed (700/350, 1.36) | killed (700/350, 3.73) | survived (4913921/8388582, 0.65) |
| 16 | polynomial: z = r instead of r*r | killed (3523862/1749619, 11.13) | killed (11875644/8388582, 0.44) | killed (11875644/8388582, 0.44) | killed (11875644/8388582, 0.45) | killed (3523862/1749619, 1.37) | killed (3523862/1749619, 3.30) | killed (11875644/8388582, 0.65) |
| 17 | constant: final +1 replaced by 1-2^-24 | killed (2/1, 12.07) | killed (2/8388582, 0.44) | killed (2/8388582, 0.45) | killed (2/8388582, 0.44) | killed (2/1, 1.39) | killed (2/1, 3.69) | survived (4913921/8388582, 0.65) |
| 18 | overflow threshold 88.8 instead of 88.7228 | survived= (1/1, 11.98) | survived= (1/8388582, 0.45) | survived= (1/8388582, 0.45) | survived= (1/8388582, 0.44) | survived= (1/1, 1.48) | survived= (1/1, 4.00) | survived= (4913921/8388582, 0.65) |
| 19 | overflow threshold 88.7 instead of 88.7228 | killed (378759/1, 8.57) | killed (378759/8388582, 0.44) | killed (378759/8388582, 0.44) | killed (378759/8388582, 0.44) | killed (378759/1, 1.41) | killed (378759/1, 3.85) | survived= (4913921/8388582, 0.65) |
| 20 | underflow threshold -87.3365 (flushes subnormal results) | killed (1/8388582, 4.68) | survived= (1/8388582, 0.44) | survived= (1/8388582, 0.44) | survived= (1/8388582, 0.45) | killed (1/8388582, 1.37) | killed (1/8388582, 3.83) | survived= (4913921/8388582, 0.65) |
| 21 | underflow threshold -104.5 instead of -103.972 | survived= (1/1, 3.57) | survived= (1/8388582, 0.44) | survived= (1/8388582, 0.45) | survived= (1/8388582, 0.44) | survived= (1/1, 1.38) | survived= (1/1, 3.98) | survived= (4913921/8388582, 0.65) |
| 22 | overflow test x >= T instead of x > T | survived= (1/1, 3.48) | survived= (1/8388582, 0.44) | survived= (1/8388582, 0.46) | survived= (1/8388582, 0.44) | survived= (1/1, 1.35) | survived= (1/1, 4.14) | survived= (4913921/8388582, 0.65) |
| 23 | NaN check removed | survived= (1/1, 5.56) | survived (1/8388582, 0.45) | survived (1/8388582, 0.44) | survived (1/8388582, 0.44) | survived= (1/1, 1.36) | survived= (1/1, 4.06) | survived (4913921/8388582, 0.65) |
| 24 | subnormal-scaling branch boundary k >= -126 instead of -125 | survived= (1/1, 5.67) | survived= (1/8388582, 0.43) | survived= (1/8388582, 0.44) | survived= (1/8388582, 0.44) | survived= (1/1, 1.35) | survived= (1/1, 4.06) | survived= (4913921/8388582, 0.65) |
| 25 | rounding: k = floor(x*log2e + 0.5) (round half up) instead of round-to-nearest-even | survived (1/1, 3.83) | survived (1/8388582, 0.44) | survived (1/8388582, 0.44) | survived (1/8388582, 0.44) | survived (1/1, 1.39) | survived (1/1, 4.03) | survived (4913921/8388582, 0.65) |
| 26 | rounding via shifter trick (x*log2e + 1.5*2^23) - 1.5*2^23 instead of round() | survived= (1/1, 4.24) | killed (8388608/8388582, 0.47) | killed (8388608/8388582, 0.44) | killed (8388608/8388582, 0.45) | killed (8388608/4194265, 1.40) | survived= (1/1, 3.67) | killed (8388608/8388582, 0.65) |
| 27 | exponent bias 126 instead of 127 | killed (8388609/4194291, 3.71) | killed (16777140/8388582, 0.47) | killed (16777140/8388582, 0.44) | killed (16777140/8388582, 0.47) | killed (8388609/4194291, 1.47) | killed (8388609/4194291, 4.00) | killed (16777140/8388582, 0.65) |
| 28 | range reduction: single ln2 constant, no Cody-Waite split | killed (5/3, 3.82) | killed (5/8388582, 0.43) | killed (5/8388582, 0.44) | killed (5/8388582, 0.44) | killed (68/27, 1.35) | killed (68/27, 3.91) | survived (4913921/8388582, 0.65) |
| 29 | scaling by integer add to exponent field (no subnormal handling) | killed (1/2141192187, NaN-mismatch 90853, 3.32) | killed (1/2141192187, NaN-mismatch 90853, 0.44) | killed (1/2141192187, NaN-mismatch 90853, 0.46) | killed (1/2141192187, NaN-mismatch 90853, 0.46) | killed (1/2141192187, NaN-mismatch 90853, 1.37) | killed (1/2141192187, NaN-mismatch 90853, 3.97) | killed (1/2141192187, NaN-mismatch 90853, 0.65) |
| 30 | polynomial: z = r*x instead of r*r | killed (4277455667/39240434, 3.94) | killed (4277455667/39240434, 0.46) | killed (4277455667/39240434, 0.44) | killed (4277455667/39240434, 0.44) | killed (4277455667/39240434, 1.35) | killed (4277455667/39240434, 3.94) | killed (4277821458/34160300, 0.65) |
| 31 | subnormal-scaling branch boundary k >= -127 instead of -125 | killed (1/5931582, 5.56) | survived= (1/8388582, 0.44) | survived= (1/8388582, 0.44) | survived= (1/8388582, 0.44) | killed (1/5931582, 1.40) | killed (1/5931582, 4.00) | survived= (4913921/8388582, 0.65) |
| 32 | Cody-Waite: subtract C2 term before C1 term (operation order) | killed (61/27, 9.82) | killed (5/8388582, 0.45) | killed (5/8388582, 0.45) | killed (5/8388582, 0.44) | killed (68/27, 1.36) | killed (61/27, 3.99) | survived (4913921/8388518, 0.65) |

- mut-cpu: killed 18 of 32, mutant sweep 277.677 s; bound max_ulp_normal<=1 max_ulp_subnormal<=1 no NaN-ness mismatch
- mut-gpu-spirv: killed 17 of 32, mutant sweep 14.256 s; bound max_ulp_normal<=1 max_ulp_subnormal<=8388582 no NaN-ness mismatch
- mut-gpu-wgsl-rerun: killed 17 of 32, mutant sweep 14.293 s; bound max_ulp_normal<=1 max_ulp_subnormal<=8388582 no NaN-ness mismatch
- mut-gpu-wgsl: killed 17 of 32, mutant sweep 14.311 s; bound max_ulp_normal<=1 max_ulp_subnormal<=8388582 no NaN-ness mismatch
- mut-llvmpipe-wgsl: killed 19 of 32, mutant sweep 44.488 s; bound max_ulp_normal<=1 max_ulp_subnormal<=1 no NaN-ness mismatch
- mut-rayon: killed 18 of 32, mutant sweep 122.305 s; bound max_ulp_normal<=1 max_ulp_subnormal<=1 no NaN-ness mismatch
- mut-metal: killed 7 of 32, mutant sweep 20.838 s; bound max_ulp_normal<=4913921 max_ulp_subnormal<=8388582 no NaN-ness mismatch

## Mutants (2^24 sample)

| id | description | mut-cpu-sample: verdict (max ULP normal/sub, s) | mut-gpu-sample: verdict (max ULP normal/sub, s) | mut-rayon-sample: verdict (max ULP normal/sub, s) |
|---|---|---|---|---|
| 0 | original | original (1/1, 0.09) | original (1/8374514, 0.01) | original (1/1, 0.02) |
| 1 | range reduction: k = trunc(x*log2e) instead of round | killed (8/4, 0.06) | killed (8/8374514, 0.01) | killed (8/4, 0.02) |
| 2 | scaling: 2^(k+1) (off by one in exponent) | killed (8388609/8374514, 0.07) | killed (8388609/8374514, 0.01) | killed (8388609/8374514, 0.02) |
| 3 | range reduction: r uses k+1 (off by one), scaling uses k | killed (8389286/4187261, 0.08) | killed (16752582/8374514, 0.01) | killed (8389287/4187261, 0.02) |
| 4 | constant: C1 (ln2 high) +1 ulp | killed (128/64, 0.07) | killed (128/8374514, 0.01) | killed (128/64, 0.02) |
| 5 | Cody-Waite: drop the C2 (ln2 low) correction | killed (449195/226925, 0.07) | killed (449195/8164556, 0.01) | killed (449195/226925, 0.02) |
| 6 | Cody-Waite: C2 sign flipped | killed (886353/459999, 0.07) | killed (886353/7944331, 0.01) | killed (886353/459999, 0.02) |
| 7 | constant: C2 +1 ulp | survived (1/1, 0.07) | survived (1/8374514, 0.01) | survived (1/1, 0.02) |
| 8 | constant: P0 (r^7 coeff) +1 ulp | survived= (1/1, 0.22) | survived= (1/8374514, 0.01) | survived= (1/1, 0.02) |
| 9 | constant: P1 (r^6 coeff) +1 ulp | survived= (1/1, 0.15) | survived= (1/8374514, 0.01) | survived= (1/1, 0.02) |
| 10 | constant: P2 (r^5 coeff) +1 ulp | survived (1/1, 0.19) | survived (1/8374514, 0.01) | survived (1/1, 0.02) |
| 11 | constant: P3 (r^4 coeff) +1 ulp | survived (1/1, 0.21) | survived (1/8374514, 0.01) | survived (1/1, 0.02) |
| 12 | constant: P4 (r^3 coeff) +1 ulp | survived (1/1, 0.44) | survived (1/8374514, 0.01) | survived (1/1, 0.02) |
| 13 | constant: P5 (r^2 coeff, 0.5) +1 ulp | survived (1/1, 0.26) | survived (1/8374514, 0.01) | survived (1/1, 0.02) |
| 14 | polynomial: drop highest-order term (P0) | killed (3/1, 0.26) | killed (3/8374514, 0.01) | killed (3/1, 0.02) |
| 15 | polynomial: drop middle term (P2) | killed (700/343, 0.06) | killed (700/8374514, 0.01) | killed (700/344, 0.02) |
| 16 | polynomial: z = r instead of r*r | killed (3523699/1742988, 0.06) | killed (3523699/8374514, 0.01) | killed (3523699/1742988, 0.01) |
| 17 | constant: final +1 replaced by 1-2^-24 | killed (2/1, 0.06) | killed (2/8374514, 0.01) | killed (2/1, 0.02) |
| 18 | overflow threshold 88.8 instead of 88.7228 | survived= (1/1, 0.05) | survived= (1/8374514, 0.01) | survived= (1/1, 0.02) |
| 19 | overflow threshold 88.7 instead of 88.7228 | killed (369623/1, 0.10) | killed (369623/8374514, 0.01) | killed (369623/1, 0.02) |
| 20 | underflow threshold -87.3365 (flushes subnormal results) | killed (1/8374514, 0.07) | survived= (1/8374514, 0.01) | killed (1/8374514, 0.02) |
| 21 | underflow threshold -104.5 instead of -103.972 | survived= (1/1, 0.06) | survived= (1/8374514, 0.01) | survived= (1/1, 0.02) |
| 22 | overflow test x >= T instead of x > T | survived= (1/1, 0.06) | survived= (1/8374514, 0.01) | survived= (1/1, 0.02) |
| 23 | NaN check removed | survived= (1/1, 0.05) | survived (1/8374514, 0.01) | survived= (1/1, 0.02) |
| 24 | subnormal-scaling branch boundary k >= -126 instead of -125 | survived= (1/1, 0.07) | survived= (1/8374514, 0.01) | survived= (1/1, 0.02) |
| 25 | rounding: k = floor(x*log2e + 0.5) (round half up) instead of round-to-nearest-even | survived= (1/1, 0.08) | survived= (1/8374514, 0.01) | survived= (1/1, 0.02) |
| 26 | rounding via shifter trick (x*log2e + 1.5*2^23) - 1.5*2^23 instead of round() | survived= (1/1, 0.08) | killed (8388580/8358173, 0.01) | survived= (1/1, 0.01) |
| 27 | exponent bias 126 instead of 127 | killed (8388609/4187257, 0.39) | killed (16752582/8374514, 0.01) | killed (8388609/4187257, 0.02) |
| 28 | range reduction: single ln2 constant, no Cody-Waite split | killed (5/3, 0.06) | killed (5/8374514, 0.01) | killed (67/26, 0.02) |
| 29 | scaling by integer add to exponent field (no subnormal handling) | killed (1/2141189308, NaN-mismatch 354, 0.12) | killed (1/2141189308, NaN-mismatch 354, 0.01) | killed (1/2141189308, NaN-mismatch 354, 0.02) |
| 30 | polynomial: z = r*x instead of r*r | killed (4277440375/39190148, 0.18) | killed (4277440375/39190148, 0.01) | killed (4277440375/39190148, 0.01) |
| 31 | subnormal-scaling branch boundary k >= -127 instead of -125 | killed (1/5926832, 0.07) | survived= (1/8374514, 0.01) | killed (1/5926832, 0.02) |
| 32 | Cody-Waite: subtract C2 term before C1 term (operation order) | killed (61/26, 0.08) | killed (5/8374514, 0.01) | killed (61/26, 0.02) |

- mut-cpu-sample: killed 18 of 32, mutant sweep 3.930 s; bound max_ulp_normal<=1 max_ulp_subnormal<=1 no NaN-ness mismatch
- mut-gpu-sample: killed 17 of 32, mutant sweep 0.272 s; bound max_ulp_normal<=1 max_ulp_subnormal<=8374514 no NaN-ness mismatch
- mut-rayon-sample: killed 18 of 32, mutant sweep 0.522 s; bound max_ulp_normal<=1 max_ulp_subnormal<=1 no NaN-ness mismatch
