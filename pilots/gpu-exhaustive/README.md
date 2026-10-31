# Pilot: exhaustive 2^32 validation and mutant schemata on GPU (CubeCL)

Question: does running validation on a GPU instead of a CPU shorten verification cycles for
numeric kernels? Results and conclusions: [results/RESULTS.md](results/RESULTS.md).

Two kernels, each written once as a CubeCL `#[cube]` kernel (`src/kernels.rs`) and once in
plain Rust (`src/host.rs`):

- **A: f32 to bf16, round to nearest even.** Spec: an integer case split in plain Rust,
  itself validated once against an independent f64 "nearest neighbour" oracle. Exact spec.
- **B: fast `expf`** (Cephes-style Cody-Waite reduction and degree-7 polynomial) with 32
  mutation sites selected by a run-time `mutant` id, so one compiled kernel serves every
  mutant. Spec: CORE-MATH `expf` (correctly rounded) through the `core-math` crate.

Every run covers all 2^32 inputs unless `--stride` asks for a seeded systematic sample.

## Paths

| backend flag | what runs |
|---|---|
| `gpu` | wgpu on the discrete GPU (Vulkan) |
| `llvmpipe` | wgpu on Mesa's software Vulkan driver |
| `cpu` | CubeCL CPU runtime (MLIR, LLVM JIT) |
| `rayon` | the plain-Rust twin, parallel with rayon (the "no GPU" status quo) |
| `default` | wgpu default adapter (used for Metal on Apple silicon) |
| `spec` | the spec on the host (only as a `diff` side) |

The wgpu shader compiler is chosen at build time: the default build compiles kernels to WGSL
(naga turns that into SPIR-V or MSL); `--features spirv` uses CubeCL's own SPIR-V compiler on
Vulkan. `scripts/build.sh` builds both binaries.

## Commands

```sh
gpu-exhaustive check   --backend gpu --kernel bf16|bf16-native|expf [--mutant M] [--repeat R]
gpu-exhaustive diff    --a gpu --b spec --kernel expf      # move all outputs to the host, compare
gpu-exhaustive mutants --backend gpu [--stride 256 --seed 7]
gpu-exhaustive fp-probe --backend gpu                      # folding, FMA contraction, FTZ, rounding
gpu-exhaustive specials --backend gpu                      # kernel B on special inputs vs the spec
gpu-exhaustive spec-validate                               # spec vs independent references
gpu-exhaustive info    --backend gpu
```

`check` evaluates and compares on the device and returns one 21-word statistics record per
thread (max ULP split by normal and subnormal expected results, NaN mismatches, ULP buckets, an
order-independent 64-bit output hash, and the number of inputs evaluated: a run that did not
evaluate all of them exits with code 3 instead of reporting). For kernel B the expected values come from a golden
table: the windows of non-trivial results are derived from the spec itself (outside them the
spec is exactly 1, +inf or +0), the table covers the 2^25-input chunks that intersect them
(2.3 GB) and is uploaded once. `diff` instead reads every output back and compares on the host.

## Build and run

Linux, Docker, a Vulkan driver on the host:

```sh
scripts/build.sh /path/to/workdir        # expects this directory at /path/to/workdir/src
scripts/run-all.sh /path/to/workdir/bin /path/to/workdir/runs/r1
python3 scripts/summarize.py /path/to/workdir/runs/r1 > summary.md
```

The build runs in `rust:1-bookworm`, so the binaries also run on newer-glibc hosts. The `cpu`
feature downloads a prebuilt LLVM bundle at build time. Without Docker:
`cargo build --release --locked [--features spirv]` (Rust 1.95 or newer, a C compiler for
CORE-MATH). On macOS: `cargo build --release --locked --no-default-features` (wgpu/Metal only,
about 1.7 GB of build output).

CubeCL keeps a small cache database under `./target/environment/` in the working directory;
run the binary from a scratch directory.

Versions are pinned in `Cargo.toml` and `Cargo.lock`: CubeCL 0.11.3, wgpu 30.0.1, naga 30.0.1,
core-math 1.4.0. Autotune is not involved: none of these kernels uses CubeCL's tunable kernels,
and the kernel variant is fixed by the `kind` comptime parameter.
