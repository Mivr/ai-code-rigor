# ai-code-rigor

Rigor levels for code written and reviewed by AI agents.

Not all code needs the same rigor. A one-off script needs to run once. A brake
controller needs every mutant killed, every runtime error proven absent and
every tool qualified. This project defines a ladder of **rigor levels, L0 to
L5**, for agent-written code, borrowing from safety-critical practice: ISO 26262,
DO-178C, IEC 61508, EN 50128/50716 and IEC 62304. Each level selects
**deterministic, machine-checkable gates** that an agent cannot talk its way past.

| Level | Kind of code | Rough analogue |
|---|---|---|
| L0 | Throwaway, run once | — |
| L1 | Scripts and internal tools | QM |
| L2 | Product code | ASIL A / DAL D |
| L3 | Core logic, HPC kernels | ASIL B / DAL C |
| L4 | High-integrity components | ASIL C / DAL B / SIL 3 |
| L5 | Proven components | ASIL D / DAL A / SIL 4 |

Levels apply **per component**, include everything below them, and a component
may depend on a lower-level one only through a verified boundary.

## Status

Research and draft specification. Nothing here is a certification tool. It
produces evidence, not certificates.

- [docs/rigor-levels.md](docs/rigor-levels.md): the draft ladder, with the
  gates per level, the CI architecture and the open decisions.
- [docs/architecture/functional-core.md](docs/architecture/functional-core.md):
  design note on functional spec, fast implementations and GPU-accelerated
  validation.
- [docs/research/deep-analysis.md](docs/research/deep-analysis.md): what the
  literature, the standards and current tooling say, and where they disagree.
- [docs/research/references.md](docs/research/references.md): every source,
  grouped by theme, with key findings and links.

## Stack

- **Rust first.** C/C++ where its high-assurance tooling is clearly ahead, for
  example MC/DC, sound abstract interpretation and verified compilation.
- **Buck2** builds, using the fork at
  [Mivr/buck2](https://github.com/Mivr/buck2) (`rigor/main`).
- **HPC and GPU compute** (WGSL, SPIR-V, CUDA) are first-class, including the
  level caps that current GPU verification tooling forces.
- **Free GitHub-hosted runners** for CI: no GPU, and noisy timing, so
  performance gates count instructions instead of measuring time.

## Research corpus policy

The research cites published work by reference: DOI, arXiv or official URL,
with short extracts only. No papers, books, standards or full texts are stored
in this repository, and CI rejects such files.

## License

Dual-licensed under MIT or Apache-2.0, at your option.
