# Design note: functional core, fast implementations, GPU-accelerated validation

Status: proposal. Measured by the pilot in [`pilots/gpu-exhaustive/`](../../pilots/gpu-exhaustive/).
Citations use the keys in [references.md](../research/references.md).

## Problem

The research ([deep-analysis.md](../research/deep-analysis.md)) shows that the cost of high rigor is dominated by
three things:

1. **Spec size and statefulness.** Proof effort grows roughly with the square of spec size (Matichuk et al. 2015,
   R² 0.69–0.94). Typical proof-to-code ratios: about 5:1 for Verus systems, about 23:1 for seL4.
2. **The trusted base.** No GPU compiler is verified or qualified, and shader compilers have long bug histories
   (Donaldson et al. 2017; wgslsmith/naga findings).
3. **Verification loop time on small CI machines**, especially mutation testing and proofs.

The goal is to lower the cost of reaching L3–L5, not just to raise rigor.

## Decision

Structure rigorous HPC code as **functional spec → fast implementation(s) → mechanical bridge → small result
checker**:

| Part | What it is | Rigor work it carries |
|---|---|---|
| Executable functional spec | Small, pure reference in a proof-friendly subset (pure Rust, Verus, or Lean) | Proofs; at L4–L5 the only artifact a human reviews (decision D1, option a) |
| Fast implementation | CPU SIMD and/or GPU kernels written as pure data-parallel combinators (map, reduce, scan, stencil) where possible | Tests, mutation, exhaustive checks; usually L3 |
| Bridge | Differential testing against the spec; exhaustive enumeration for small domains; equivalence checking where a tool exists (VOLTA for PTX); determinism hashes under permuted work-group orders | Turns "fast and untrusted" into "checked against spec" |
| Result checker | Small CPU-side function that verifies an output cheaply: residual for solves, sortedness plus permutation for sorts, Freivalds' check for matmul | Carries the L4–L5 claim (ISO 26262-9 decomposition pattern: complex function + small, separately specified monitor) |

Rigor cost concentrates in the spec and the checker, which are small. The fast code stays at L3 and is
checked, not trusted.

## Why functional kernels are cheaper to make rigorous

- **Race freedom by construction.** GPU verifiers work on barrier-synchronised kernels with simple indexing.
  Faial verified 218 of 227 such kernels (Cogumbreiro et al. 2021). They fail on atomics, warp-level tricks and
  data-dependent writes. Pure combinators stay inside the verifiable fragment, and a type system can enforce
  that (Descend, Köpcke et al. 2024).
- **Smaller specs and proofs**: no framing or heap invariants.
- **The spec is executable.** It can be tested, reviewed and used as the differential oracle. Precedent: Cedar's
  Lean model plus differential random testing, where testing found 21 bugs and proofs found 4
  (Disselkoen et al. 2024).
- **Deterministic and hermetic**, so test results can be cached and mutation runs are cheap.

Floating point stays hard. Purity does not make addition associative. Kernels need explicit, fixed reduction
trees. The cost is about 1× for large bandwidth-bound sums and 2–7× for small sums (Demmel & Nguyen 2015;
Collange et al. 2015). Tolerances come from Muller-style bounds, for example (n−1)·u·Σ|aᵢ| for summation in any
order (Muller et al. 2018).

## Where the GPU speeds up validation

| Workload | GPU speedup? | Notes |
|---|---|---|
| Differential and property testing of numeric kernels | Yes, large | Embarrassingly parallel |
| Exhaustive enumeration of small domains (all 2³² unary f32 inputs, all 16-bit pairs) | Yes, and it is a proof for that domain | No annotations, no solver |
| Mutation testing of kernels | Yes, with mutant schemata | One compile; the runtime `mutant_id` selects the mutant (Untch et al. 1993) |
| Metamorphic checks, determinism under permuted schedules | Yes | Many independent runs |
| SMT/SAT proofs (Verus, Kani, CBMC, Frama-C) | No | Branchy and sequential. GPUs help only preprocessing ([Osama & Wijs](https://doi.org/10.1007/978-3-030-17462-0_2), 66× on SAT simplification). Proof speed comes from CPU cores plus caching unchanged obligations. |
| Compilation, coverage-guided fuzzing of general code | No | CPU-bound |

Split: **proofs on CPU cores for the small spec and checkers; brute-force validation on GPU for the large
fast code.**

## The trust problem: common-mode failure

If a kernel and its check go through the same compiler and driver, one miscompilation can corrupt both and
pass. ISO 26262-9:2018 Table C.1 lists "same tools" as a common cause. Proofs do not help, because they are about
source semantics.

Countermeasure: **diversity of compiler paths, not trust in one compiler**.

- Run all validation on the GPU path.
- Also run a seeded random sample, about 1% and gate-owned, through an independent path: a CPU backend
  (MLIR/LLVM) or a second GPU backend (Vulkan vs CUDA vs Metal). Any disagreement fails the gate.
- Nightly, run the exhaustive domains on two paths.
- GPU results must be bit-reproducible (fixed reduction order) to be gateable.

## Pilot results (2026-10-31)

The pilot ([results](../../pilots/gpu-exhaustive/results/RESULTS.md)) ran two kernels on all 2³² f32 inputs:

- an exact f32→bf16 conversion;
- a fast `expf`, checked against CORE-MATH, which is correctly rounded.

Each kernel ran on an AMD RX 6700 XT (Vulkan), llvmpipe, CubeCL's CPU backend, 32-thread rayon and an Apple M2
(Metal). The pilot also checked 32 `expf` mutants.

- **Speed.** A full 2³² check takes 0.11 s (bf16) and 0.45 s (`expf`) on the discrete GPU. All 32 mutants, each
  checked exhaustively, take 14.3 s, against 122 s on 32 CPU threads. The GPU is 2.5–14× faster than the CPU,
  depending on how lean the CPU loop is. **The comparison must stay on the device**: reading 16 GiB of outputs
  back makes the GPU about 3× slower than the CPU. One-time setup, such as uploading a 2.3 GB golden table, means
  the GPU only pays off across repeated checks. A 2²⁴ sample gave the same mutant verdicts as exhaustive checking.
- **The GPU does not run the same program as the CPU.** The drivers change floating-point semantics:
  - every Vulkan and Metal path folded `(a+b)-b` into `a`;
  - the AMD GPU and Metal fused multiply-adds and flushed subnormals to zero;
  - Metal turned an overflow-safe scaling into `x*inf`.

  The shader code CubeCL emitted was correct, so the rewrites happen in the driver. An `expf` that is within
  1 ULP on the CPU was off by 2²³ ULP on the GPUs, and three mutants got different verdicts on different backends.
- **Exact integer kernels behave identically everywhere.** The bf16 conversion had zero mismatches and an
  identical output hash on every path.
- **The checker itself can pass silently.** When a kernel failed to compile, CubeCL panicked on a background
  thread and the read returned zero-filled buffers, which a naive comparison reports as "0 mismatches". The
  harness now counts the inputs it actually evaluated and fails if the count is short. This is a concrete case
  of principle P4 in the [rigor-levels spec](../rigor-levels.md): checkers need seeded-fault validation.
- **The pilot found two bugs in CubeCL.** The CPU backend's f32→bf16 cast is wrong for subnormal inputs
  (16.7M wrong outputs), and the SPIR-V compiler rejects `bool != bool`.

Consequences for the ladder:

1. **Cross-backend validation is valid for exact (integer or bit-level) kernels.** Disagreement between
   backends is then a real bug.
2. **Floating-point kernels must be validated on every backend they ship on**, each with its own oracle and
   tolerance. A GPU pass is not evidence about the CPU build, and the reverse also holds. Where the semantics
   must match, prefer integer or fixed-point formulations, or kernels written to be robust to FMA contraction,
   flush-to-zero and reassociation.
3. **Every exhaustive or sampled check reports how many inputs it evaluated**, and gates fail on a short
   count.

## CubeCL as the kernel layer

[CubeCL](https://github.com/tracel-ai/cubecl) compiles Rust `#[cube]` functions to wgpu (Vulkan, Metal, WebGPU),
CUDA, HIP, and a CPU backend via MLIR/LLVM. One source compiled through independent compiler paths is what the
diversity countermeasure needs. The CPU backend also makes kernels testable on GPU-less free runners.

Caveats for rigor use:

- **Alpha.** Breaking API changes between minor versions, so pin exact versions. The CPU backend is not yet fully
  optimised for matmul and convolution.
- **Autotune** chooses kernel configurations at runtime. Disable it, or ship a pinned autotune cache, and record
  it in the evidence.
- **JIT compilation** means the device binary is not a build artifact. Record the generated IR or binary hash in
  the gate report.

## Level mapping

| Component | Typical level | Evidence |
|---|---|---|
| Functional spec | L4–L5 | Proofs, spec tests, human review of the frozen spec (D1a) |
| Result checker | L4–L5 | Proofs, exhaustive or bounded checks |
| Fast CPU implementation | L3–L4 | Differential, exhaustive and mutation checks; Kani on `unsafe` |
| Fast GPU kernel (WGSL, rust-gpu, CubeCL-wgpu) | ≤ L3 | Cross-backend differential, exhaustive, determinism |
| Fast CUDA kernel, structured | L4 under the conditions in [rigor-levels.md §5.2](../rigor-levels.md) | Plus PTX equivalence (VOLTA-style) |

## Open questions

1. Is the spec language pure Rust (reusing the kernel types), Verus or Lean? Lean has better proof tooling but
   needs a second language and a Lean-to-Rust differential harness.
2. Which result checkers belong in the project's standard library: residual, Freivalds, sort-permutation,
   checksum-based ABFT?
3. What sample rate on the diverse path is enough? Calibrate it with seeded miscompilation canaries.
4. Can the type system enforce the functional-kernel fragment (Descend-style), or is it a lint?
