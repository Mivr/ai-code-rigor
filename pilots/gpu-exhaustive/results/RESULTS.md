# Results: exhaustive 2^32 validation and mutant schemata on GPU

Measured 2026-10-31. Raw logs: [`raw/`](raw/). Every table generated from them:
[`summary.md`](summary.md). How to rerun: [`../README.md`](../README.md).

## Short answer

- **A GPU makes a full 2^32 check cheap.** On the discrete GPU a complete exhaustive check
  takes **0.11 s** for the bf16 conversion and **0.45 s** for `expf`. All 32 `expf` mutants,
  each checked on all 2^32 inputs, take **14.3 s**.
- **Speedup over the 32-thread CPU depends on how lean the CPU loop is.** Against the same
  algorithm in plain Rust with rayon, the GPU is **14x** faster on bf16, **8.2x** on `expf` and
  **8.6x** on the mutant sweep. Against a hand-trimmed CPU loop that only counts errors, it is
  **2.5x** on bf16 and **3.9x** on `expf`.
- **Most of the gain disappears unless the comparison runs on the device.** Reading the 16 GiB
  of outputs back to compare on the host takes about 13 s over PCIe, which makes the GPU about
  3x *slower* than the CPU. One-time setup also counts: building the golden table and
  uploading it takes about 4.3 s, so a single `expf` check is no faster end to end (4.8 s on GPU
  against 4.4 s on CPU). The GPU only pays off over repeated checks.
- **The biggest finding is not speed.** The GPU does not check the same program the CPU runs.
  - The Vulkan and Metal paths apply optimizations that break IEEE semantics. All of them fold
    `(a+b)-b` to `a`. The AMD GPU and Metal also fuse multiply-adds into FMAs and flush
    subnormals to zero, and Metal reassociates expressions.
  - The `expf` kernel as first written uses the textbook rounding trick. It is correct on the
    CPU (max 1 ULP) and off by 2^23 ULP on every Mesa and Metal path.
  - The corrected kernel is still wrong on the GPUs: on the AMD GPU and on Apple Metal for
    subnormal results, and on Metal also near overflow, where it returns +inf.
  - Three mutants get a different verdict depending on the backend.
- **A silent false pass in the tooling.** When a kernel fails to compile, CubeCL logs a panic
  on a background thread, and the read still returns zero-filled buffers. A checker that does
  not count how many inputs it actually evaluated reports a clean pass. This harness now
  refuses to report in that case.

## Setup

| | |
|---|---|
| Linux workstation | AMD Radeon RX 6700 XT (12 GB, PCIe), 32-thread (16-core) x86-64 desktop CPU with AVX-512, 62 GB RAM; Ubuntu 26.04, Linux 7.0, Mesa 26.0.8 (RADV, and llvmpipe on LLVM 21.1.8), Vulkan 1.4.335 |
| Mac | Apple M2 (8 CPU cores, 16 GB unified memory), macOS 26.6, wgpu on Metal |
| Build | `rust:1-bookworm` container, rustc 1.99.1, glibc 2.36; the binaries run on the host with glibc 2.43. The Mac uses rustc 1.99.1 natively. |
| Crates (exact, `Cargo.lock`) | cubecl / cubecl-wgpu / cubecl-cpu / cubecl-runtime 0.11.3; wgpu 30.0.1; naga 30.0.1; ash 0.38.0; CubeCL CPU runtime LLVM bundle 23.1.0-4; core-math 1.4.0 (CORE-MATH C sources); rayon 1.11.0; half 2.7.1 |
| Autotune | Not involved. These kernels do not use CubeCL's tunable kernels, and the variant is fixed by a comptime parameter. Persistent compile caches are off (no `persistence` feature). |

**Paths compared.** Each is one row in the timing tables.

- RADV: wgpu on the AMD GPU. Built two ways: with WGSL (then naga to SPIR-V), and with
  CubeCL's own SPIR-V compiler.
- llvmpipe: wgpu on Mesa's software Vulkan driver, built the same two ways.
- CubeCL CPU runtime: MLIR, then LLVM JIT.
- rayon: the plain-Rust twin of each kernel, run in parallel.
- M2 Metal: wgpu with WGSL, then naga to MSL.
- M2 rayon: the plain-Rust twin on the Mac's 8 cores.

**Kernels.**

- **A, f32 to bf16 with round-to-nearest-even.** This is the usual bias-and-shift trick. The
  spec is an explicit integer case split in plain Rust. It was checked once, on all 2^32
  inputs, against an independent oracle that picks the nearest bf16 neighbour with exact f64
  arithmetic: **0 mismatches**.
- **B, fast `expf`.** It follows Cephes: Cody-Waite range reduction (ln 2 split into two
  constants so the reduction stays exact) and a degree-7 polynomial. It has 32 mutation
  sites, selected by a run-time `mutant` id.
  - The spec is **CORE-MATH `expf`**, which is correctly rounded, via the `core-math` crate.
  - Cross-checked on all 2^32 inputs: f64 `exp` from glibc, rounded to f32, agrees with
    CORE-MATH on **every** input (0 differences). So for this libm the "approximate spec"
    would also have been exact.
  - glibc's own `expf` differs from CORE-MATH by 1 ULP on 170,648 inputs.

**Two check modes.**

- **On-device `check`.** The device evaluates, compares and reduces to one 21-word record per
  thread. Only those records are read back, about 350 MB per run.
  - For kernel A, the device also computes a port of the integer spec.
  - For kernel B, the device reads a golden table from memory. The table covers the windows
    where `expf` is non-trivial: positive inputs from 2^-24 to 88.7228, negative inputs from
    -2^-25 to -103.972, 520,184,780 inputs in all. The windows are derived from the spec
    itself; outside them the spec is exactly 1, +inf or +0.
  - The table holds the 2^25-input chunks that intersect those windows, 2.3 GB in all, and is
    uploaded once.
- **Readback `diff`.** Every output is moved to the host and compared exactly with the
  plain-Rust spec or with another path.

**Timing.** All times are wall clock with outputs ready on the host, and exclude JIT
compilation, which is listed separately. Times are medians of 3 runs in the same process.

## Experiment 1: exhaustive validation (all 2^32 inputs)

### Kernel A: f32 to bf16

Every path agrees with the spec on every input: 0 mismatches, NaN payload policy included.
That is a proof for this domain. Every path also gives the identical 64-bit output hash,
`95794923f73805f6`, which equals the hash of the plain-Rust spec's outputs.

| path | check (s) | JIT (s) | vs rayon, same algorithm |
|---|---|---|---|
| RADV, WGSL | **0.111** | 0.032 | 14.3x |
| RADV, CubeCL SPIR-V | 0.112 | 0.014 | 14.1x |
| llvmpipe, WGSL | 0.430 | 0.054 | 3.7x |
| llvmpipe, CubeCL SPIR-V | 0.455 | 0.031 | 3.5x |
| CubeCL CPU runtime | 4.92 (1.33 in an earlier, less loaded run) | 0.027 | 0.3x |
| rayon, same algorithm (32 threads) | 1.584 | n/a | 1x |
| rayon, minimal mismatch-count loop | 0.278 | n/a | 5.7x |
| M2 Metal | 0.408 | 0.074 | (M2 rayon: 3.77; minimal loop 1.03) |

**Readback mode.** Each run below moves all outputs to the host (16 GiB at 4 bytes per input)
and compares them exactly with the plain-Rust spec. Every path shows 0 differences.

| path | total (s) | of which: producing and reading outputs (s) |
|---|---|---|
| RADV | 17.9 | 13.3 |
| llvmpipe | 17.4 | 12.8 |
| CubeCL CPU | 11.7 | 7.1 |
| rayon | 5.8 | 0.8 |
| M2 Metal (unified memory) | 6.7 | 2.2 |

**CubeCL's own `f32 -> bf16` cast.**

- On wgpu it is unsupported: `UnsupportedType("cube.bf16")`.
- On the CPU runtime it gets **16,711,678 inputs wrong**. All are subnormal f32 inputs, or
  inputs that round to the smallest normal bf16: the cast returns ±0 for subnormal inputs
  (max error 128 bf16 ULP).
- The CPU has AVX-512 BF16, and its conversion instruction treats subnormal inputs as zero.
  That is a likely cause, but it was not confirmed in the generated code.
- The exhaustive check found this in about 4 s. A test set without subnormal inputs would not
  have found it.

### Kernel B: fast expf

ULP error against the correctly rounded spec. NaN-ness matches on every input on every path.
The spec returns a canonical NaN marker, so NaN payloads are not compared.

| path | check (s) | JIT (s) | max ULP, normal results | max ULP, subnormal results | inputs with 1 ULP error | output hash |
|---|---|---|---|---|---|---|
| rayon (plain Rust) | 3.769 | n/a | **1** (first at `0x33800000`, 5.96e-8) | 1 | 18,092,448 | `e3d54898337e0d71` |
| rayon, minimal loop | 1.757 | n/a | 1 | 1 | n/a | n/a |
| llvmpipe, WGSL / CubeCL SPIR-V | 1.317 / 1.259 | 0.038 / 0.041 | 1 | 1 | 18,092,448 | `e3d54898337e0d71` (bit-identical to rayon) |
| CubeCL CPU runtime | 9.49 (4.4 when less loaded) | 0.036 | 1 | 1 | about 17.2 M | `a3bc4f8d47190450` |
| RADV, WGSL / CubeCL SPIR-V | **0.461 / 0.445** | 0.014 / 0.022 | 1 | **8,388,582** (flushed to 0) | about 17.3 M | `02f592ab97a45af1` (both compilers) |
| M2 Metal | 0.653 | 0.39 | **4,913,921** (+inf near overflow) | 8,388,582 | n/a | `8543a325ee7c54c9` |
| M2 rayon (8 threads) | 14.03 (minimal loop 4.67) | n/a | 1 | 1 | 18,092,448 | `e3d54898337e0d71` |

- **No path has an error above 1 ULP in the normal range, except Metal.** The rayon path has
  18,092,448 inputs at 1 ULP (0.42% of all non-NaN inputs, 3.5% of the non-trivial window)
  and none at 2 ULP or more.
- **One-time setup, kernel B only.**
  - Building the golden table on the CPU with rayon takes 0.67 to 0.79 s: a scan of all 2^32
    inputs, then the table fill.
  - Uploading it takes **3.5 s** on RADV (about 0.66 GB/s), 5.4 s on llvmpipe, 4.1 s on the
    CPU runtime and 2.5 s on M2.
- **Readback mode.** Comparing the RADV outputs directly with CORE-MATH on the host, with no
  golden table, takes 18.2 s, 13.4 s of it producing and reading outputs. The same run on
  rayon takes 8.1 s.

**Special values.** All are exact on every path except the rows listed here
(`summary.md` has the full table).

| input | rayon, llvmpipe, CPU runtime | RADV | M2 Metal |
|---|---|---|---|
| ±0, ±inf, quiet and signalling NaN, ±1, -2^-25, the overflow edge `0x42b17218`, the underflow edge `0xc2cff1b5`, -87.3365 (`0xc2aeac4f`) | exact | exact | exact |
| largest input with a finite result, `0x42b17217` (88.7228) | exact | exact | **+inf (124 ULP)** |
| 88.3763, where k = 128 | 1 ULP | 1 ULP | **+inf (4,913,921 ULP)** |
| `0xc2aeac50`, result just below FLT_MIN | exact | **0 (8,388,582 ULP)** | **0** |
| -100, deep subnormal result | exact | **0 (27 ULP)** | **0** |
| -103.97208 (`0xc2cff1b4`), result 2^-149 | exact | **0 (1 ULP)** | **0** |
| 2^-24 (`0x33800000`) | 1 ULP | 1 ULP | 1 ULP |

### Kernel B as first written: the shifter trick

The first version rounded with `k = (x*log2e + 1.5*2^23) - 1.5*2^23`. In IEEE arithmetic that
is exact round-to-nearest-even. It is now mutant 26.

| path | max ULP, normal results |
|---|---|
| rayon, CubeCL CPU runtime | 1 (correct) |
| RADV, WGSL and CubeCL SPIR-V | **8,388,608** |
| llvmpipe, WGSL | **8,388,608** |
| M2 Metal | **8,388,608** |

The `fp-probe` command pins down the cause:

| probe (IEEE result) | CubeCL CPU | RADV (WGSL, SPIR-V) | llvmpipe (WGSL, SPIR-V) | M2 Metal |
|---|---|---|---|---|
| `(x+c)-c` with c = 1.5*2^23 at run time (0) | 0 | **x (folded)** | **x (folded)** | **x (folded)** |
| `a*a - fl(a*a)` (0, or 2^-24 if fused into an FMA) | **FMA** | **FMA** | 0 | **FMA** |
| `2^-100 * 2^-30` (2^-130, subnormal) | kept | **flushed to 0** | kept | **flushed to 0** |
| `round(2.5)`, `round(-0.5)`, `f32(i32(-1.75))` | 2, -0, -1 | same | same | same |

- **CubeCL emits the operations correctly.** Its WGSL contains the add and the subtract, and
  its own SPIR-V contains `OpFAdd` then `OpFSub`. Neither carries a `NoContraction` decoration
  or any float-controls execution mode. Both were checked in the compilation logs.
- **The folding happens downstream.** It appears with both compilers on both Mesa drivers.
  The CubeCL SPIR-V path skips naga, so naga is not the cause, and Mesa is what the two Linux
  paths share. This fits Mesa's inexact algebraic rewrites, which are allowed when the SPIR-V
  asks for no exactness. The NIR itself was not inspected.
- **Metal (via naga to MSL) shows the same folding, plus reassociation.** It turns
  `(p*2^127)*2` into `p*(2^127*2) = p*inf`, which explains the +inf results near overflow.
- **The fix in the original:** use `round()`, which is ties-to-even and correct everywhere.

### Cross-backend agreement: the common-mode check

| pair | outputs that differ | where |
|---|---|---|
| llvmpipe vs rayon | **0** | bit-identical: no FMA, no flush to zero |
| CubeCL CPU vs rayon | 2,078,124 | 2,076,561 normal, 1,563 subnormal: FMA contraction; 1 ULP apart |
| RADV vs CubeCL CPU | 2,180,453 | **only** inputs with subnormal results. RADV and the CubeCL CPU runtime fuse the same operations into FMAs. |
| RADV vs rayon | 4,257,014 | flush to zero (2,180,453) plus FMA (2,076,561) |
| RADV, WGSL vs CubeCL SPIR-V | 0 by output hash, for kernel B and for all 33 mutant runs | same driver behaviour from both compilers |
| M2 rayon vs Linux rayon | 0 by output hash | plain Rust is reproducible across x86-64 and ARM |

Comparing backends is what exposed the folding, the flushing, the reassociation and the
CPU runtime's bf16 cast, each within seconds. A single backend checked against itself would
have shown none of them.

## Experiment 2: mutant schemata (kernel B, exhaustive on every path)

One compiled kernel serves all 33 runs (the original plus 32 mutants): `mutant` is a run-time
parameter, so there is no recompilation.

**Oracle.** A mutant is killed if, on the same backend, it is worse than the original:
- in max ULP over normal results, or
- in max ULP over subnormal results, or
- if it changes NaN-ness anywhere.

The bound is therefore 1/1 ULP on rayon, llvmpipe and the CPU runtime; 1/8,388,582 on RADV,
because the original already flushes subnormal results; and 4,913,921/8,388,582 on Metal.

**Equivalence.** `=` means the mutant's 64-bit output hash over all 2^32 inputs equals the
original's on that backend, so its outputs are identical with overwhelming probability.

Legend: K killed, S survived, S= survived with identical outputs.

| id | mutation | rayon | CPU runtime | llvmpipe | RADV | M2 Metal |
|---|---|---|---|---|---|---|
| 1 | k = trunc instead of round | K (14) | K | K | K | S |
| 2 | scale by 2^(k+1) | K | K | K | K | K |
| 3 | r uses k+1, scaling uses k | K | K | K | K | K |
| 4 | C1 +1 ulp | K (128) | K | K | K | S |
| 5 | drop C2 correction | K | K | K | K | S |
| 6 | C2 sign flipped | K | K | K | K | S |
| 7 | C2 +1 ulp | S | S | S | S | S |
| 8 | P0 +1 ulp | S | S= | S | S= | S= |
| 9 | P1 +1 ulp | S | S | S | S | S |
| 10 | P2 +1 ulp | S | S | S | S | S |
| 11 | P3 +1 ulp | S | S | S | S | S |
| 12 | P4 +1 ulp | S | S | S | S | S |
| 13 | P5 +1 ulp | S | S | S | S | S |
| 14 | drop highest-order term | K (3) | K | K | K | S |
| 15 | drop middle term | K (700) | K | K | K | S |
| 16 | z = r instead of r*r | K | K | K | K | K |
| 17 | +1 becomes 1-2^-24 | K (2) | K | K | K | S |
| 18 | overflow threshold 88.8 | S= | S= | S= | S= | S= |
| 19 | overflow threshold 88.7 | K | K | K | K | S= |
| 20 | underflow threshold -87.34 (flushes subnormal results) | K | K | K | **S=** | S= |
| 21 | underflow threshold -104.5 | S= | S= | S= | S= | S= |
| 22 | overflow test `>=` instead of `>` | S= | S= | S= | S= | S= |
| 23 | NaN check removed | S= | S= | S= | S (NaN payload only) | S |
| 24 | subnormal branch boundary -126 | S= | S= | S= | S= | S= |
| 25 | k = floor(x*log2e + 0.5) | S | S | S | S | S |
| 26 | shifter trick instead of round() | **S=** | **S=** | **K** | **K** | **K** |
| 27 | exponent bias 126 | K | K | K | K | K |
| 28 | single ln2 constant, no Cody-Waite split | K (68) | K (5) | K (68) | K (5) | S |
| 29 | scale by integer add to the exponent field | K (NaN) | K (NaN) | K (NaN) | K (NaN) | K (NaN) |
| 30 | z = r*x instead of r*r | K | K | K | K | K |
| 31 | subnormal branch boundary -127 | K | K | K | **S=** | S= |
| 32 | subtract the C2 term before the C1 term | K (61) | K (61) | K (68) | K (5) | S |
| | **killed** | **18/32** | **18/32** | **19/32** | **17/32** | **7/32** |
| | **whole sweep (s)** | **122.3** | 277.7 (loaded) | 44.5 | **14.3** | 20.8 |
| | **per mutant (s)** | 3.3 to 4.1 | 3.3 to 14.4 | 1.33 to 1.48 | 0.43 to 0.47 | 0.65 to 0.67 |

Numbers in parentheses are the max ULP over normal results, where informative.
Per-mutant times and max ULP for every path are in `summary.md`. RADV gave the same
verdicts and the same output hash per mutant with both compilers and in a second process.

**Observations.**

- **Equivalent mutants.** These have identical outputs on all 2^32 inputs on every IEEE-faithful
  path, so they are equivalent on that compiled artifact:
  - 18 and 22 (overflow handling): the scaling overflows to +inf by itself.
  - 21 and 24 (underflow and branch boundaries).
  - 23 (NaN propagates through the arithmetic anyway).
  - 26 on rayon and the CPU runtime.
  - 8 on the paths that contract FMAs.
- **Accuracy-neutral survivors: 7, 9 to 13 and 25.** Their outputs differ from the original's,
  but they never exceed 1 ULP. A ULP-bound oracle cannot kill them. Comparing outputs exactly
  with the original's would.
- **Verdicts that depend on the backend.**
  - 20 and 31 are killed on every CPU path but equivalent on RADV and Metal, because flush to
    zero already erases the region they break.
  - 26 is equivalent on CPU paths and killed wherever `(a+b)-b` is folded.
- **Mesa factors 32 into 28.** On both Mesa drivers, mutant 32, `(x - k*C2) - k*C1`, gives
  exactly mutant 28's outputs: the same output hash on RADV, and on llvmpipe the same hash as
  rayon's mutant 28. That fits an inexact rewrite `k*C2 + k*C1 -> k*(C1+C2)`, which turns the
  two-constant reduction back into a single ln2 constant.
- **The per-backend oracle collapses on Metal.** Metal's original is already 4.9 M ULP off near
  overflow and flushes subnormals, so only 7 of 32 mutants are killed.
- **A 2^24 sample (stride 256, seed 7) gave the same verdict for every mutant** on rayon, the
  CPU runtime and RADV. The sample sweeps took 0.27 s on RADV, 0.52 s on rayon and 3.9 s on the
  CPU runtime. For these mutants exhaustive checking was not needed to kill them, and at sample
  sizes the GPU's lead nearly vanishes, because fixed per-launch costs dominate.

## Experiment 3: determinism

- **Repeated runs.** Every repeated on-device check gave a bit-identical statistics record and
  output hash: 3 repeats per path for both kernels, on every path, including Metal.
- **Second process.** The RADV mutant sweep run again in a new process gave identical hashes and
  verdicts for all 33 runs.
- **Hash design.** It is a pair of wrapping sums of mixed `(input, output)` values, so the order
  of evaluation does not affect it, and the device reduction needs no atomics.
- **Kernel A.** Same hash on all paths, on both machines, and equal to the hash of the plain-Rust
  spec. Readback diffs confirm 0 differing outputs on every path.
- **Kernel B.** Results are deterministic per path but not portable across paths. Each path is
  bit-stable, and the paths split into four groups:
  - rayon (x86-64 and ARM) = llvmpipe;
  - the CubeCL CPU runtime;
  - RADV, with either compiler;
  - Metal.

## Problems hit

1. **CubeCL 0.11.3's SPIR-V compiler rejects `bool != bool`** (`cmp.bool_not_equal ... doesn't
   support conversion to SPIR-V`). Worked around by comparing u32 flags instead.
2. **A kernel that fails to compile still yields a "result".** The panic happens on CubeCL's
   server thread, the read returns zero-filled buffers, and the harness printed "0 mismatches".
   Fixed with a per-thread count of evaluated inputs, checked against 2^32; on a mismatch the
   run exits with code 3 (`coverage_error evaluated=0`).
3. **Platform limits.**
   - llvmpipe caps a buffer at 128 MiB, so launches cover 2^25 inputs.
   - The CubeCL CPU runtime caps a workgroup at 32 units.
   - wgpu has no bf16, so the native-cast kernel only runs on the CPU runtime.
4. **Choosing the compiler takes two builds.** In a build with `--features spirv`,
   `WgpuRuntime<WgslCompiler>` still reported `wgpu<spirv>`, so WGSL and SPIR-V are compared
   as two builds (`--features spirv`).
5. **API differences from the docs.** The `Runtime` trait is not re-exported by `cubecl`, so the
   project depends on `cubecl-runtime` directly. `bf16` needs the `half` crate.
6. **Toolchain and build environment.**
   - CubeCL 0.11.3 needs rustc 1.95 or newer; the Mac had 1.94. A pinned 1.99.1 toolchain
     was installed for the build, and rustup updated itself while doing so.
   - The Linux host has no C compiler, so everything is built in Docker.
7. **Slow data movement in CubeCL's wgpu path.** On the discrete GPU, readback runs at about
   1.2 GiB/s and the golden upload at about 0.66 GB/s. Both are far below PCIe bandwidth, and
   neither was tuned.
8. **The workstation was shared.** Other jobs pushed the load average to 15 to 38 during the
   final run. CPU-bound paths suffered most: the CubeCL CPU runtime ran 2 to 4x slower than
   in quieter runs, and one early sampled run took 20 s, which did not reproduce. GPU timings
   were stable to within about 5%.

## Conclusions

**What GPU validation buys for this class of checks.** The class is element-wise numeric
kernels on a 32-bit domain, with an oracle that is cheap to evaluate on the device.

- A complete exhaustive check, which for a 32-bit domain is a proof, drops to about 0.1 to 0.5 s.
- A 32-mutant exhaustive sweep drops to about 14 s.
- Against a 32-thread desktop CPU that is a **3 to 9x** speedup for `expf`-like kernels and
  **2.5 to 14x** for bit-manipulation kernels like bf16. The range depends on how well the CPU
  baseline is written.
- On the Mac, the M2 GPU against its own 8 CPU cores gives about **7 to 21x** on `expf`.
- In a tight edit-and-check loop that repeats checks, this turns seconds into sub-second
  feedback.

**What it does not buy.**

- **A one-off check.** Setup (golden table and upload) costs as much as one CPU check.
- **Any workflow that moves outputs to the host.** Readback over PCIe is slower than computing
  on the CPU.
- **Sampled checks.** Fixed overheads dominate, and here a 2^24 sample already gave the same
  mutant verdicts.
- **Larger domains.** Two-argument f32 functions and f64 have 2^64 inputs, out of reach of
  exhaustive checking on any device.
- **The CPU program.** An exhaustive pass on the GPU proves a property of the artifact that
  driver produced on that device, not of the CPU build. Here the two differ, by up to 2^23 ULP.

**What it covers in addition.** Running the same kernel on several backends is a cheap
common-mode check. In one afternoon it found:
- a toolchain-wide break of a standard rounding trick;
- GPU flush to zero and FMA contraction;
- Metal reassociation;
- a subnormal bug in CubeCL's own bf16 conversion on the CPU runtime;
- a harness false pass.

For rigor levels this suggests:
- **Name the artifact.** Evidence must state the backend, compiler, driver and their versions.
- **Prefer GPU paths for mutation sweeps** and exhaustive regression of 32-bit kernels,
  provided:
  - the semantics are pinned (no fast math, FTZ declared or prevented), or
  - the GPU path is the deployment target.
- **Keep an IEEE-faithful path in the loop.** Here that is rayon, or llvmpipe, which matched
  rayon bit for bit except for the folding.
- **Add the coverage guard as a gate.**

## Threats to validity

- **Hashes stand in for some comparisons.** "Identical outputs" for mutants, compilers and
  machines rests on a 64-bit non-cryptographic order-independent hash. Exact output diffs
  were run for kernel A on every path and for the kernel B pairs listed above.
- **Kernel A's on-device check uses a spec built by the same compiler.** The device compares
  against a port of the spec compiled by the same toolchain, which is a common-mode risk. It is
  mitigated by exact readback diffs against the plain-Rust spec on every path, and by hash
  equality with the plain-Rust spec.
- **Kernel B's verdicts depend on CORE-MATH.** They rest on CORE-MATH being correctly rounded
  for binary32 `expf`. It was cross-checked against f64 `exp` with 0 differences, but the two
  are not fully independent: both target correct rounding.
- **One machine per platform.** One GPU model, one driver version (Mesa 26.0.8), one Metal
  version and one CubeCL version, which is alpha. Driver updates may change the folding and
  flushing behaviour.
- **Timings come from a shared machine.** CPU-path numbers are pessimistic, especially for the
  CubeCL CPU runtime. Nothing was tuned: the GPU check kernel, the per-thread work split
  (1024 inputs per thread; 256 to 4096 changed GPU time by up to about 30%) and the readback path
  were all left at defaults. The "minimal" rayon loops are a best-effort lower bound, not an
  optimized one.
- **The mutants are a small hand-picked set.** I picked all 32 myself, which risks selection
  bias. The verdicts also depend on the oracle chosen (per-backend envelope vs a universal
  bound).
- **Only two kernels, both element-wise.** Kernels with reductions, memory-access patterns or
  inter-thread communication would behave differently.
