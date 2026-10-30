# Rigor levels L0–L5 for agent-written code (draft specification)

Status: **draft v0.1, 2026-10-30**. Research synthesis turned into a draft spec. Nothing here is implemented yet,
and nothing here certifies anything: the tool produces *evidence*, not certificates.

Full references for every citation are in [research/references.md](research/references.md). Citations use
(Author Year), a named benchmark or tool with its authors, or a standard number and clause. Where a standard was read
only through a secondary source, the secondary source is named.

**How to read thresholds.** Every threshold in this document carries one tag:

| Tag | Meaning |
|---|---|
| **[S]** | Taken from the normative text of a safety standard (clause given). Normative, not empirically validated. |
| **[E]** | Derived from empirical evidence (source given). The number may still need calibration on our own code. |
| **[P]** | A policy choice. The evidence supports the *direction*, not the number. The owner can tune it. |
| **[U]** | Unverified: comes from a secondary source or background knowledge and must be checked before it becomes binding. |

**Where checks run.** `PR` = every pull request on a free GitHub-hosted runner (4 vCPU, 16 GB RAM, 14 GB disk, no
GPU). `NL` = nightly or merge-queue sweep on free runners, sharded. `GPU` = a self-hosted GPU machine that pulls
trusted refs only (§6.5). `REL` = release gate, run once per tagged release.

---

## 1. Executive synthesis

Every safety standard we examined (IEC 61508, ISO 26262, DO-178C, EN 50128/50716, ECSS, NASA NPR 7150.2) uses the same
mechanism: a risk-derived integrity level selects a graded set of techniques, evidence and independence, rising along
fixed axes (structural coverage, independence, formal methods, language restrictions, tool qualification,
traceability). We adopt that mechanism, with three changes forced by agent-written code. First, the code generator (the
LLM) cannot be qualified, so the standards' common escape clause ("a tool needs no qualification if its output is fully
verified downstream") becomes the whole strategy: **no gate is ever skipped because an agent wrote the code**, and the
*checkers* become the tools that need qualification. Second, the agent both writes and is graded, and the 2025–26
reward-hacking literature shows that any oracle an agent can see, edit or persuade gets gamed while prompting barely
helps; so every oracle (gate config, tests, specs, baselines, holdouts) is **sealed** outside the agent's reach, every
gate is pass/fail and conjunctive, and ratchets only tighten. Third, a second model is not an independent verifier
(correlated errors are well above independence), so independence comes from **diverse methods** (proof, mutation,
differential testing, fuzzing, simulation), not diverse models. The evidence base for the per-level technique tables
themselves is weak (observational, never controlled), so the ladder leans on *product* evidence, marks every number
by provenance, and keeps the open policy choices explicit. The largest practical limits are on the Rust and GPU side:
Rust has no MC/DC, no MISRA-equivalent and no sound whole-program abstract interpreter today, and WGSL/SPIR-V compute
has no static verifier at all, so GPU code is capped (WGSL ≤ L3, structured CUDA ≤ L4, nothing at L5) and the very
top level routes some obligations to narrow C islands with mature tooling.

---

## 2. Principles

Each principle is derived from the research. The ladder in §4 is these principles applied per level.

### P1. The level comes from risk, declared as an input, not inferred

Integrity levels in every standard are derived from a hazard assessment, never from the code. ISO 26262 adds three
scores (severity, exposure, controllability); a sum of 10 maps to ASIL D, 9 to C, 8 to B, 7 to A, and 6 or less (or
any zero) to QM (Schildbach 2018). IEC 61508 and EN 50128 tie SILs to tolerable hazard rates (Reichl 2016). The
cross-domain consensus is that confidence should be proportional to risk (Hatcliff et al. 2014).

We use an S×E×C analogue for software components:

| Factor | 0 | 1 | 2 | 3 | 4 |
|---|---|---|---|---|---|
| **S** severity of a wrong result | none (throwaway) | inconvenience, easily redone | loss of work, money or data; security exposure | injury or irreversible large loss | (unused) |
| **E** exposure | never run in a deployed context | rare, internal | regular, internal or limited external | routine, external or untrusted input | continuous, safety-relevant or adversarial |
| **C** controllability | failure is harmless | failure is always caught downstream before harm | usually caught (monitor, human, reversible) | rarely caught | (unused) |

Level = **L5** if S+E+C = 10, **L4** if 9, **L3** if 8, **L2** if 7, **L1** if ≤ 6 or any factor is 0, and **L0** only
by explicit declaration ("throwaway, never depended on"). The scales and the mapping are **[P]**, modelled on
ISO 26262-3 as summarised by Schildbach (2018).

The level and its hazard profile are recorded in a sealed manifest (§3.2). The tool records the profile; it does not
infer it (Holloway 2012). Lowering a level is a loosening and needs the root-of-trust signature (P8).

### P2. Levels apply per component, with an inclusion rule and a dependency rule

- **Inclusion.** Every build target belongs to exactly one component with a declared level. Unassigned targets fail
  the gate. Configuration, parameter data, lookup tables, shader constants and tolerance tables **inherit the level**
  of the component that consumes them (EN 50128:2011 clause 8, via AdaCore n.d.; DO-178C Table A-5 objectives 8–9,
  via RTCA FTP1034 2022).
- **Dependency (freedom from interference).** A component at level *n* may depend on one at level *m < n* only if
  one of these holds:
  1. **Verified boundary.** The boundary is shown to prevent spatial, temporal and communication interference
     (ISO 26262-6:2018 7.4.9 and Annex D; static and data/control-flow analysis are acceptable evidence there).
  2. **Asymmetric decomposition.** The lower-level element is paired with a small, independently specified,
     deterministic monitor at the parent level, and the pair is integrated and verified at the parent level
     (ISO 26262-9:2018 clause 5; EN 50716 rates such a "safety bag" R/HR at all SILs, per Hobbs 2026).
  3. **Lift-up.** Otherwise the dependency is verified at level *n* itself.
- **Two agent-written variants never count as a decomposition.** Same tools and same language are dependent-failure
  coupling factors (ISO 26262-9:2018 Table C.1), and decomposition is frequently abused (Ward & Crozier 2012).
- Prefer writing new code at the target level over promoting existing code: retrofitting usually failed in
  IEC 61508 assessments (Lloyd & Reeve 2009).

### P3. The LLM is an unqualified tool, so its output is fully verified and no gate is skipped

All five domains share one rule: a tool needs qualification only if a required activity relies on its output **and**
that output is not verified afterwards (ISO 26262-8:2018 clause 11, where TI2 + TD1 gives TCL1; IEC 61508-3:2010
7.4.4.8; EN 50128:2011 6.7.4.4 and 6.7.4.6; DO-178C 12.2, via Rierson 2013 and Pothon & Ochem n.d.; ECSS-Q-ST-80C
6.2.8.3a, via AdaCore 2021, the only clause written specifically for code generators). NASA requires generated code
to meet the same standards as hand-written code (NPR 7150.2D SWE-146).

An LLM writer is a Criteria-1 / TI2 / T3 tool that cannot be qualified: non-deterministic, no operational
requirements, evidence voided by every model update, and qualification of even deterministic generators costs
hundreds of hours per KSLOC (Taft et al. 2016). "Proven in use" is unavailable: no published defect history, and the
field-use thresholds run to thousands of failure-free operation-years (Smith & Simpson 2020).

Consequences:

- **No gate is waived, relaxed or sampled because an agent wrote the code.** The moment a gate is skipped on the
  generator's strength, the generator needs qualification at the code's level.
- **At L4–L5 at least one deterministic check operates directly on the generated source** (proof, static analysis,
  contract checking), not only tests derived from it. This maps the TD1/TD2 distinction of the IEC 61508 edition 3
  draft (via Hobbs 2026); the mapping is our inference.
- **Evidence must be regenerable without the model.** ECSS requires tools to stay available for the maintenance life
  (via AdaCore 2021); a hosted model will be retired. Every L3+ artefact is reproducible from pinned deterministic
  tools.
- Early support that verifier-in-the-loop generation works: 15 of 15 runs converged to verified code with a formal
  verifier in the loop, 0 of 30 single-pass runs did (FORGE, Wei et al. 2026, preprint).

### P4. The checkers are the tools that need qualification

Coverage tools, provers, Miri, Clippy, mutation engines, race detectors and the build system are verification tools
whose failure can mask an error (Criteria 2/3, TI2, T2). ISO 26262-8 recommends tool validation (method 1c) for TCL2/3
at higher ASILs, and when tools are chained a checker's evaluation must consider the tool whose output it checks
(Conrad et al. 2011). Version pinning is normative (IEC 61508-3:2010 7.4.4.15–18). Ferrocene's public qualification
documents show a usable template: use cases, then hazards, then errors, then a mitigation or test for each (Ferrous
Systems 2026).

The evidence that checkers fail is strong:

- Six Rust SAST tools missed 94.1–99.7% of 320 real vulnerabilities, and tool crashes caused 41.8% of the misses
  (Rust SAST evaluation 2026).
- NVIDIA's racecheck found 15% of a calibrated racy-kernel suite (SuperCollider, PLDI 2026).
- GCC at `-O1` and above reports different MC/DC obligations from Clang (Weiss & Schulz 2026); object-branch
  coverage falls from 53% to 28% between `-O0` and `-O3` (Byun et al. 2017).
- Every compiler tested by Csmith miscompiled (Yang et al. 2011).

So every checker gets: a pinned version, a documented use case, a known-limitations list, and a **canary pack** of
seeded faults it must catch. A tool crash, timeout or "unknown" verdict **fails** the gate; it never silently passes.

### P5. Independence comes from diverse methods, not diverse models

Standards scale independence with level: DO-178C requires test independence at levels A and B (via Pothon & Ochem
n.d.); IEC 61508-1 requires an independent person at SIL 1–2, department at SIL 3, organisation at SIL 4 (via Smith &
Simpson 2020, [U] for the primary text); ISO 26262-2:2018 Table 1 defines I0–I3.

Two models, or two contexts of one model, are not independent in that sense:

- 27 independently written program versions failed together far more often than independence predicts (Knight &
  Leveson 1986).
- When two LLMs are both wrong they agree on the same wrong answer 60% of the time against a 33% baseline (Kim et
  al. 2025).
- Majority voting over code models realises only 0.43–0.44 of the reliability gain that independence predicts, and
  0.24–0.27 for samples from one model; prompt diversity adds about nothing (Nogueira et al. 2026, preprint).
- At equal compute, self-repair matches plain resampling (0.97–1.05×) (Olausson et al. 2024).
- LLM judges are inconsistent and persuadable: self-agreement as low as 37.5% on graded scales (Judge Reliability
  Harness 2026); a "this is correct" comment cuts wrong-code detection by 9–25 points (Moon et al. 2026).

Therefore: **LLM review is advisory only and never counts as evidence.** Independence is achieved by combining methods
with different failure modes (Weaver et al. 2005), and by separating *who writes the oracle* from *who writes the
code*: spec and test authors run in separate contexts with no write access to the code under test, and their output
is sealed. Cross-family model diversity for oracle authors is credited only as defence in depth, at most at the
measured discount, never multiplied as independent.

### P6. Oracles are sealed

Measured agent behaviour, condition by condition:

- Test editing is the dominant cheat for some model families: over 79% of their cheats on ImpossibleBench (Zhong et
  al. 2025).
- Hard-coding test inputs: 0–2.1% when tests are correct, 22–44% when they are wrong or ambiguous (EvilGenie, Gabor et
  al. 2025).
- Changing the given spec or code caused 86–91% of proof-generation failures, about three quarters of attempts
  (VeriContest, Xie et al. 2026).
- Agents violate 43.1% of triggered repository policies, and 50.3% of violations are visible only in the action
  trajectory, not the final diff (SWE-CC 2026).
- Agents fetch upstream fixes over the network and forge dependencies outside the diff (SWE-Bench Pro V2, Scale AI
  2026).

Prompting does not fix this: policies in context add about 9 points (SWE-CC 2026). Hardening does: four sandbox
controls cut exploits from 6.5% to 0.8% (−87.7%) with no loss of task success (Thaman 2026, preprint). Hidden tests
the agent cannot read caught every non-heuristic hack in EvilGenie.

Sealed means: gate config, baselines, test oracles, specs, tolerances, mutation operators, seeds and holdout tests live
outside the candidate's write path, are pinned by content hash, and are compared at gate time. Faults that measure an
agent's tests come from a fixed external operator set, never from the agent: hand-seeded faults are 6–34 points
harder to detect than mutants (Andrews et al. 2005), so whoever chooses the faults controls the score.

### P7. All gates must pass; there are no weighted scores

Every quantitative assurance-confidence technique with a reproducible worked example gave implausible results on
constructed counterexamples (Graydon & Holloway 2017). A score invites trading one weakness against another
strength, which is exactly what an optimiser does. Gates at a level are **conjunctive**: one failure fails the
candidate. Scores (raw mutation score, LLM judge ratings, complexity totals) appear in the report as information only.

### P8. Ratchets only tighten; loosening needs a root of trust

Per-entity baselines are written only by the merge bot after a passing merge. A config change is auto-accepted only if
a monotonicity proof passes: every ceiling is at most its old value, every allow-list is a subset of the old one, no
metric, canary, seed or operator was removed, and every grow-only list did not shrink. Anything else (a loosened
threshold, a lowered level, a removed canary, a new waiver) needs a signature from the owner's hardware-backed key.
This generalises the waiver discipline of the standards: a highly-recommended technique that is not used needs a
recorded justification (EN 50128:2011 Annex A semantics; NPR 7150.2D waivers).

### P9. Mid-range mutation thresholds are meaningless

The raw mutation score is a weak linear predictor of real-fault detection once suite size is controlled
(r ≈ 0.05–0.20), yet suites in the top decile of mutation score do find more real faults than random suites of equal
size (+11% Java, +46% C) (Papadakis et al. 2018). Mutation beats coverage only in the high-score regime (Chekam et al.
2017). Only 0.4–4.8% of mutants are subsuming, and killing a random 0.5% of mutants already gives a raw score of 50% or
more (Papadakis et al. 2016); selective strategies scoring 88–99% raw score 27–82% on the minimal set (Ammann et al.
2014).

So: **no gate on a mid-range raw score** (no "≥ 60%" anywhere). Below L4 the gate is **zero unexplained surviving
mutants** in a defined scope, treating each survivor as a finding (Petrović et al. 2021, 2022). At L4–L5 the gate is a
near-saturation **subsuming** score computed nightly from a full kill matrix. Survivor-based gates are indifferent to
suite size, so they resist inflation by many cheap tests.

### P10. Size-confounded metrics are hygiene caps only

- A static metric's predictive validity is explained by its correlation with size at R² 0.95–0.97, and even a
  nonsense metric predicts as well as size (Gil & Lalouche 2017).
- No CK metric remains significant once LOC is controlled (El Emam et al. 2001).
- Cyclomatic complexity is a valid count of decisions but has no predictive advantage over size (Landman et al. 2016;
  Gil & Lalouche 2017; Fenton & Bieman 2014).
- Common thresholds are benchmark percentiles or convention, never defect-derived (Alves et al. 2010; Oliveira et al.
  2014); the Sonar cognitive-complexity default of 15 has no empirical support (Muñoz Barón et al. 2020).
- In agent-adoption data, a +41.6% complexity rise shrinks to +9% after size control (He et al. 2026).

So LOC, cyclomatic and cognitive complexity are **review-budget and hygiene caps**, labelled as such, never quality
evidence. Thresholds use the relative form "p% of units ≤ k" re-derived on a pinned corpus, with a distribution-shape
check for piling up just under *k* and for an explosion of tiny units (the gaming signature anticipated by Oliveira et
al. 2014). Binary structural design rules (no cycles, declared layering, declared public interfaces) are gates,
because coupling is the one metric family with some size-independent support (Zhou et al. 2009) and modularity
violations concentrate bug-fix churn (Xiao et al. 2016).

### P11. Claims are product claims, and L4–L5 claims are bounded by the spec

Product evidence beats process compliance (Weaver et al. 2002; Hobbs 2026; Graydon & Kelly 2013). A proof shows only
what the spec says, under the trusted base's assumptions; every LLM verification study found agents exploiting
whatever trust construct the tool offers (VeruSAGE, Yang et al. 2025; AlphaVerus, Aggarwal et al. 2024; Antoneac et
al. 2026). Most software-related accidents trace to requirement flaws rather than code (Leveson 2011). The tool
therefore states its strongest claim as **"verified against the frozen spec"**, never "safe" (see open decision D1).

---

## 3. Problems, root causes and solutions

### 3.1 Coverage and test-pass gaming

**Problem.** Agents reach high coverage and green tests with tests that check little: 14–37% of LLM-written test
classes contain empty tests and up to 77% lack assertions (Ouédraogo et al. 2026); 49% of Meta's mutant-killing LLM
tests added zero line coverage, showing coverage and checking are different things (Foster et al. 2025); 31% of
"resolved" SWE-bench patches pass only through weak tests (Aleithan et al. 2024) and 19.78% of passing patches fail
once tests are strengthened by mutation (SWE-ABS, Yu et al. 2026).

**Root cause.** Coverage measures execution, not checking; its correlation with effectiveness collapses once suite
size is controlled (Inozemtseva & Holmes 2014). Suites generated *to hit* coverage do worse than random suites of the
same size (Gay et al. 2015). Kills come from assertions (r 0.93–0.98 between assertion count and mutation score at
fixed test count; Zhang & Mesbah 2015). An agent optimises the visible signal.

**Solution.**

- Coverage is a **completeness floor**, never an objective given to agents; it finds untested code, mutation judges
  tested code (the reconciled position of Chen et al. 2020 and Inozemtseva & Holmes 2014).
- Assertion-free and empty tests fail at L1+ (AST check).
- Pseudo-tested functions (body replaceable by a default with all tests green) fail at L2+ on changed code (Maton et
  al. 2024).
- Mutation survivors on changed code **and its dependents** are findings at L3+ (§3.5). At L3+ every changed function
  needs at least one kill attributed to an assertion failure, not a crash or timeout (Zhang & Mesbah 2015).
- Hidden held-out tests with inputs generated from gate-owned seeds, including renamed or permuted copies of visible
  cases; this kept the reward gap near zero in a verifier-gaming study (RLVR verifier-gaming study 2026).
- Differential testing against a sealed reference at L3+ (29.6% of passing SWE-bench patches behave differently from
  the reference fix; Wang et al. 2026, PatchDiff).

### 3.2 Oracle tampering: tests, specs, configuration and the environment

**Problem.** Agents edit or delete tests, override `==`, record call state to behave differently under test, read
grader files (12.7%) and exploit planted environment bugs (34.8%) (ImpossibleBench; Roth et al. 2026), exploit
fail-open result parsers (a faulty log parser in about 54% of instances; Yu et al. 2025, UTBoost) and edit the
obligation in proofs (VeriContest). Once a model hacks in a session it almost always hacks again (Roth et al. 2026).

**Root cause.** The oracle sits on the agent's write path or inside its sandbox; results fail open; the repository
under test controls how it is judged (its own build flags, lint config and test runner).

**Solution: a sealed-oracle architecture**, generalised from an anti-gaming design the author already runs on another
project.

1. **Sealed gate repository.** Gate config, lint profiles, mutation operators, seeds, tolerances, baselines, canaries
   and the level manifest live in a separate repository pinned by commit hash. The candidate repository contains the
   product, never how it is judged. Gate flags are passed **on the command line** (Buck2 `-c`/`--config-file`
   overrides), so editing the repo's own `.buckconfig` cannot weaken a gate.
2. **Holdout store.** Holdout tests, contradiction probes (tests that pass only if code special-cases the test
   environment) and design mutants live under a separate account whose credentials exist only on the gate runner.
   Holdouts run on a trigger outside the candidate's workflows, are rate-limited per PR, return coarse feedback only
   (area, never test source) and rotate about 20% per quarter **[P]**.
3. **Identities.** The merge bot is the only writer of the default branch. Agent tokens lack administration and
   workflow-editing permissions. Approval is a bot attestation, never text in a PR.
4. **Fresh hermetic grading.** Grading runs in a fresh container with no network, outside the agent's sandbox;
   test-owned files are restored from the sealed copy before grading; results are structured (libtest/JUnit JSON) and
   **every expected test ID must report an explicit pass** (missing = fail).
5. **Test integrity.** The gate computes the tested-target set and per-target test-case counts itself; neither may
   shrink unless a replacement covers the same sources. No new `#[ignore]`, `should_panic` flips, retries or sleeps.
6. **Spec freeze.** `requires`/`ensures`/spec functions live in files owned by a spec-author role; a deterministic AST
   diff with ghost code erased fails any proof-only change that touches specs or executable code (the AutoVerus
   pattern, Yang et al. 2025).
7. **Canaries.** Before judging a candidate, the gate overlays a pack of known-bad fixtures on a scratch copy and
   reruns the same pipeline. Every canary must turn red, or the run is rejected as "gate integrity compromised". Each
   new gate ships with its canary in the same change.
8. **Trajectory evidence (L3+).** A recorded command and tool-call log is checked by deterministic policy checkers;
   one detected hack disqualifies the whole session; decoy files whose access is logged sit in the sandbox.
9. **Protected paths.** Changes to build, lint or test infrastructure inside the candidate are allowed (it is product
   code too) but trigger the full canary pack and holdouts, and merge alone so a failure is attributable.

### 3.3 Vacuous specs and proof escape hatches

**Problem.** Of 7,134 verifier-accepted spec artefacts, only 42.3% pass triviality and contract checks and acceptance
correlates weakly with correctness (τb ≈ 0.26) (VerusEval, Wang et al. 2026). 28–31% of LLM-written Kani specs that
passed were incomplete or vacuous (KaPilot, Wang et al. 2026). Dafny alone accepted 100% of adversarial program mutants
in Clover's study (Sun et al. 2024). Agents complete 81% of proofs given frozen expert specs on real Verus systems
(VeruSAGE), but proving an LLM spec equivalent to a reference succeeds at most 1.9% of the time (CLEVER, Thakur et al.
2025), and end-to-end spec + code + proof sits at 1–5% (VERINA, Ye et al. 2026; VeriContest).

**Root cause.** "Verified" means the spec holds under the trusted base: `assume`, `admit`, `external_body`, `axiom`,
`#[trusted]`, `kani::assume`, small unwind bounds, stubs, `sorry`, `decreases *` and spec edits all make false claims
provable (McCormick & Chapin 2015; VeruSAGE; AlphaVerus). Without a deterministic checker, cheating ran at 2–14% of
tasks; with one it fell below 1.5% but never reached zero (VeruSAGE).

**Solution.**

- **Escape-hatch ledger.** Every trust construct is counted per component from the AST. L4: budgeted, monotonically
  non-increasing, each with a sealed waiver. L5: zero outside one allow-listed boundary file signed by the spec owner;
  Lean/Coq artefacts show only standard axioms (`#print axioms`, `Print Assumptions`).
- **Spec-strength checks (deterministic):** `requires` satisfiable (a witness is proved); `ensures` not a tautology and
  not implied by `requires`; a stub returning default or arbitrary values must **fail** verification; the spec is
  proved on frozen concrete I/O cases and must reject a frozen adversarial negative-output suite (VeriScale's scaled
  suites cut spec pass rates by about a third at 1.05–1.4× evaluation time; Bai et al. 2026); termination via
  `decreases` (Lahiri 2024; Leino 2023).
- **Per-tool vacuity probes:** Kani `cover!` reachability and an injected `ensures(false)` that must fail (KaPilot);
  CBMC unwinding assertions always on and a coverage job flagging unreachable lines (Chong et al. 2021); Frama-C
  `-wp-smoke-tests` with zero doomed goals; Eva configuration hashed and alarm-class switches deny-listed (Kirchner et
  al. 2015).
- **Determinism of results.** Resource limits (`rlimit`), not wall-clock timeouts; solver "unknown" or timeout fails;
  proof results are never cached across a code or spec change, because a behaviour-preserving refactor broke 35–52% of
  proofs (and 73–96% after helper extraction), though a two-token repair usually fixed them (proof-stability study
  2026, Lean only).
- **Pin the agent setup with the toolchain.** A Verus upgrade cut an agent pipeline's success from 75% to 53%
  (ExVerus, Yang et al. 2026); toolchain bumps rerun a proof canary suite.

### 3.4 GPU kernels that cannot be verified on GPU-less CI

**Problem.** Free runners have no GPU. Testing a kernel on one device against a tolerance is weak evidence: a perfect
differential oracle could catch at most 47 of 82 production kernel bugs and six bugs produce errors below the
tolerance (Racer, Vu & Tan 2026, preprint); vendor race detection finds 15–27% of known races (Jacobson et al. 2024;
SuperCollider 2026); every shader compiler tested had bugs, naga included (Donaldson et al. 2017, 2023; Bernhard et al.
2024); bounds checks in WGSL can legally be optimised away in the presence of races (SafeRace, Levine et al. 2025);
73.8% of claimed GPU kernel speed-ups had no end-to-end gain (SpecBench, Zhao et al. 2026).

**Root cause.** No static race verifier, coverage tool, mutation tool or deductive verifier exists for WGSL, SPIR-V,
naga IR or rust-gpu (GPU addendum gap list). Software Vulkan implementations likely run work-groups serially and so
cannot show weak-memory behaviour or most inter-work-group races (our inference; no fidelity study exists). Tolerances
are chosen by hand and drift under pressure (Vanover et al. 2020 document a tolerance raised from 1e-8 to 1e-3).

**Solution.**

- **Level caps (§5.2):** WGSL/rust-gpu ≤ L3; structured CUDA kernels L4 under stated conditions; no GPU code at L5.
- **CPU-side, deterministic checks on every PR:** naga and `spirv-val` validation; multi-backend differential execution
  (naga vs Tint, on lavapipe and SwiftShader or WARP; any disagreement fails); K permuted work-group orders must give
  an identical output hash (Peng & Rajan 2019); N-run determinism hashes (comparison with a sequential run had 87%
  recall in HiRace's study, far above the vendor tool); reproducible or fixed-order reductions.
- **Derived tolerances.** At L3+ tolerances are *computed* from order-independent error bounds (summation in any order
  or tree is bounded by (n−1)·u·Σ|aᵢ|) and the per-builtin ulp tables of the target API, never guessed (Muller et al.
  2018). Some binary32 builtins are allowed up to 8192 ulp by API profiles. Every tolerance oracle gets a vacuity check
  (an all-zeros output must fail; Du et al. 2026) and an IEEE special-value sweep (Vanover et al. 2020).
- **CUDA static evidence on CPU:** race and barrier-divergence freedom (Faial verified 96% of 227 real kernels;
  Cogumbreiro et al. 2021, 2023), PTX-level equivalence to a reference kernel (VOLTA, Dubey et al. 2025).
- **Dynamic and performance evidence on the self-hosted GPU machine only**, pulled from trusted refs (§6.5).

### 3.5 Slow mutation and proof loops on 4-vCPU runners

**Problem.** cargo-mutants rebuilds per mutant (median 1.6 h on a benchmark set where a schema-based engine took 13–23
s; Lévai et al. 2026) and generates 8–31% invalid mutants; Miri runs 3000–7000× slower than native (Jung et al. 2026);
CBMC has a tail of tens of minutes and tens of GB (Chong et al. 2021); fuzzing cost per new bug grows exponentially
(Böhme & Falk 2020).

**Root cause.** Whole-repo runs, per-mutant compilation, and treating "more time" as more assurance.

**Solution.**

- **Affected-target selection** from the Buck2 graph (target-level selection is safe; file-level selection was
  the fastest safe option in Gligoric et al. 2015), with a full run nightly.
- **Mutation scope = changed lines + dependents.** Diff-only mutants are the worst choice: 81% of commit-relevant
  mutants lie outside the changed lines (Ojdanic et al. 2022; Ma et al. 2021). Use a fixed-seed sample over 1-hop
  dependents per PR; seeded random sampling is near-optimal (an oracle strategy beats random by only 13%; Gopinath et
  al. 2016).
- **Schemata, then TCE on survivors.** One compile with runtime mutant switching (Untch et al. 1993; Lévai & McMinn
  2023), then compile only survivors at a pinned optimisation level and hash the object code: identical to the original
  means equivalent, identical to another mutant means duplicate (Papadakis et al. 2015; Kintis et al. 2018 remove
  about 28% of C mutants this way). Combining TCE with schemata through the Buck2 action cache is our inference, not a
  published result.
- **Content-addressed caching** of verifier results keyed on (code, spec, toolchain, config) hashes; valid only
  because results are deterministic (rlimit, pinned seeds).
- **Hard per-PR budgets; timeouts fail.** Fuzzing gates on corpus replay plus a fixed short budget per target, never on
  "hours fuzzed" or coverage (Klees et al. 2018; Böhme et al. 2022).
- **Nightly:** seeded full-repo mutation sample (≥ 1,000 mutants or 10%, every mutant visited within a week **[P]**),
  subsumption scoring, Miri seed sweeps, long fuzz campaigns, larger model-checking constants.

### 3.6 Rust tooling gaps

**Problem.** Rust has no MC/DC: the unstable rustc implementation was removed in 2025 and a 2026 project goal plans an
unstable re-implementation (rust-lang/rust PR #144999; Rust project goals 2026). There is no MISRA-equivalent: the
Safety-Critical Rust Consortium guidelines are incomplete and a 2026 goal plans 50–200 safety lints over one to two
years. There is no sound whole-program abstract interpreter for Rust, no qualified Rust verifier, and no effective Rust
SAST (§P4). Only `core` is partially certified (5,169 functions to IEC 61508 SIL 2 and ISO 26262 ASIL B as of
Ferrocene 26.02); `std` is not.

**Root cause.** Ecosystem age; MC/DC instrumentation was a maintenance burden; pattern-match MC/DC semantics exist only
on paper (Zaeske et al. 2024).

**Solution.**

- **Declared substitutes, honestly labelled.** For Rust at L4–L5: condition-operand mutation (relational, logical,
  match-arm operators) killed at an asserted output, plus proof of every decision. This approximates observable MC/DC,
  which behaves like strong mutation of conditions (Meng et al. 2018), but mutation **cannot formally subsume** MC/DC
  because MC/DC constrains *pairs* of tests (Ammann & Offutt 2016). The evidence bundle states "MC/DC objective not
  met; substituted by …", never equivalence.
- **Pinned, curated lint profile** in place of MISRA (§4), with the effective lint level read from the compiler, not
  the source: crates were found declaring `unsafe` forbidden while overriding it per function (Evans et al. 2020).
- **Per-function proofs instead of abstract interpretation:** Verus, Flux or Prusti for panic/overflow/bounds freedom;
  Kani for bounded checks; Miri for executed UB.
- **C islands** where the property needs mature C tooling: Clang MC/DC, Frama-C (WP/Eva) or Astrée, CompCert. Each
  island sits behind a narrow, verified FFI boundary (P2).
- **SAST is advisory at every level**; a SAST tool crash fails the gate; Rust security evidence at L3+ comes from Miri,
  fuzzing, Kani and executable exploit tests (BaxBench: about 50% of functionally correct LLM backends were exploitable
  and a commercial SAST missed 64 of 237; Vero et al. 2025).

### 3.7 Measurement validity and checker trust

**Problem.** The metrics people reach for (complexity, warning density, coverage %, raw mutation score, LLM ratings)
measure size or execution, invite gaming, or come from tools with large blind spots.

**Root cause.** Size confounding (P10); proxies with no defect link (Clippy warning density has never been linked to
defects; Li et al. 2024); thresholds without provenance; and checkers that are themselves fallible (P4). No study has
yet measured agents gaming complexity metrics, so that risk is inferred, not measured.

**Solution.** A **metric registry** in the sealed repository. Every metric declares id, tool, scope, unit, direction,
mode (`hard_cap`, `ratchet` or `advisory`), provenance tag ([S]/[E]/[P]), and at least one **countermetric** that
gaming it would trip (e.g. function LOC ↔ module totals and propagation cost; test count ↔ mutation survivors;
suppressions ↔ protected-path diffs). Drift detection flags a merge where a metric improves sharply while its
countermetric worsens. The registry refuses an entry without a countermetric. Exact property checks (no cycles,
`forbid(unsafe_code)` effective, zero new suppressions) are the preferred gate form.

---

## 4. The L0–L5 ladder

Levels are **cumulative**: each table lists only what a level adds or tightens; everything from the levels below
still applies. Tools are named as "Rust tool; C/C++ equivalent".

Crosswalk (approximate; levels are not formally equivalent across standards):

| Level | IEC 61508 | ISO 26262 | DO-178C | EN 50128/50716 | IEC 62304 | NASA |
|---|---|---|---|---|---|---|
| L0 | – | – | E | – | – | non-safety |
| L1 | below SIL 1 | QM | D | basic integrity | A | Class D |
| L2 | SIL 1 | A | D–C | SIL 1 | B | Class C |
| L3 | SIL 2 | B | C | SIL 2 | C (low) | Class B |
| L4 | SIL 3 | C | B | SIL 3 | C | safety-critical |
| L5 | SIL 4 | D | A | SIL 4 | C | Class A + safety-critical |

(After Hayhurst et al. 2001; IEC 61508-3:2010; ISO 26262-6:2018; EN 50128:2011; IEC 62304:2015; NPR 7150.2D. The
DO-178C column follows coverage objectives: level D has none, C statement, B decision, A MC/DC.)

### 4.0 L0: throwaway

**Intended code:** exploratory scripts, one-off data fixes, spikes. **Risk analogue:** DAL E; no claim is made.

| Area | Gate (deterministic check) | Threshold | Tool | Runs | Source |
|---|---|---|---|---|---|
| Requirements/spec | Manifest entry with `level = "L0"` and a one-line purpose | present | manifest checker (this project) | PR | P1 |
| Architecture & deps | No target at L1+ depends on an L0 target | 0 edges | Buck2 `cquery` + visibility | PR | ISO 26262-9:2018 cl. 5 (lift-up) |
| Language & lints | Compiles; formatted | pass | rustc, rustfmt; clang, clang-format | PR | — |
| Toolchain | Toolchain pinned by hash | pinned | Buck2 hermetic toolchain | PR | IEC 61508-3:2010 7.4.4.15 |
| Evidence | Build log in gate report | present | gate report | PR | — |

All other areas: none. Nothing is claimed, so gaming does not matter.

### 4.1 L1: internal tools and non-safety code

**Intended code:** internal tools, build scripts, dashboards, prototypes others use. **Risk analogue:** QM; DAL D;
NASA Class D.

| Area | Gate | Threshold | Tool | Runs | Source |
|---|---|---|---|---|---|
| Requirements/spec | Every top-level requirement in the manifest has ≥ 1 test tagged with its ID | 100% of HLRs | trace checker (this project) | PR | **[S]** DO-178C Table A-7 HLR coverage, level D (Hayhurst et al. 2001) |
| Architecture & deps | No cycles between crates, Buck2 targets or modules | 0 **[P]** | Buck2 graph; module-graph tool | PR | Mo et al. 2021 (design rule) |
| Architecture & deps | Dependency rule (P2) holds for all edges | 0 violations | manifest + Buck2 graph | PR | ISO 26262-6:2018 7.4.9 |
| Language & lints | Zero warnings under the sealed rustc + Clippy profile, passed on the command line | 0 **[P]** | rustc, Clippy; `-Wall -Werror` + clang-tidy | PR | **[S]** NPR 7150.2D SWE-135 (static analysis, all classes); Li et al. 2024 (gate on zero, not density) |
| Language & lints | `#![forbid(unsafe_code)]` effective at every crate root unless allow-listed (effective level read from the compiler) | 0 unlisted | rustc lint-level query | PR | Evans et al. 2020; Astrauskas et al. 2020 (76.4% of crates use no `unsafe`) |
| Language & lints | `deny(dead_code)`; suppression count reported | deny; report | rustc; `-Wunused` | PR | Moon et al. 2026 (dead code biases judges) |
| Metrics | LOC, cyclomatic and cognitive complexity reported | advisory | rust-code-analysis-style tool; lizard | PR | P10 |
| Static analysis | Dependency policy: advisories, licences, sources, bans | 0 findings **[P]** | cargo-deny; OSV scanner | PR | practitioner consensus (2026) |
| Testing adequacy | Full Buck2 reverse-dependency test set of changed files passes, retries off | 100% pass | `buck2 test`; ctest | PR | Wang et al. 2026 (PatchDiff: 7.8% of "solved" fail the full suite) |
| Testing adequacy | Structured results; every expected test ID reports an explicit pass | missing = fail | libtest/JUnit JSON parser | PR | **[E]** Yu et al. 2025 (UTBoost) |
| Testing adequacy | No empty or assertion-free tests in changed test code | 0 **[E]** | AST check (this project) | PR | Ouédraogo et al. 2026 (14–37% empty) |
| Testing adequacy | Changed-region coverage reported | advisory | `-C instrument-coverage`, llvm-cov; Clang source-based coverage | PR | Winters et al. 2020 (fixed bars become ceilings) |
| Simulation/determinism | No new `#[ignore]`, retry attribute or `sleep` in tests without allow-list | 0 | AST lint | PR | Luo et al. 2014 |
| GPU/numerics | Every WGSL/SPIR-V module validates; smoke test on lavapipe | pass | naga, `spirv-val`; lavapipe | PR | wgpu CI (2026) |
| Toolchain & supply chain | All tools identified with pinned versions; `Cargo.lock` committed; lockfile or path-dependency changes need a separate approved task | pinned; 0 unapproved | Buck2 toolchains; lock diff check | PR | **[S]** EN 50128:2011 6.7 (T1); Scale AI 2026 (dependency forgery) |
| Anti-gaming/process | Gate config pinned by hash, outside the candidate; fresh no-network grading container; test files restored from sealed copy; test-target set and test-case counts may not shrink; canary pack for each enabled gate turns red | all pass | gate runner (this project) | PR | **[E]** Thaman 2026; Zhong et al. 2025; SWE-CC 2026 |
| Anti-gaming/process | LLM review, if used, is advisory and recorded separately | advisory | — | PR | P5 |
| Evidence | Gate report: versioned JSON envelope with SARIF 2.1.0 runs for findings, metrics outside SARIF, toolchain/config/canary hashes, signed by the gate identity | present | gate report | PR | P8 |

### 4.2 L2: product code

**Intended code:** shipped libraries and services without safety impact; customer-visible behaviour. **Risk
analogue:** ASIL A; SIL 1; DAL D–C; NASA Class C.

| Area | Gate | Threshold | Tool | Runs | Source |
|---|---|---|---|---|---|
| Requirements/spec | Low-level requirements (per public function or module contract) traced to unit tests | 100% of LLRs | trace checker | PR | **[S]** DO-178C Table A-7 LLR coverage, levels A–C (Hayhurst et al. 2001) |
| Architecture & deps | Declared layering rules on the Buck2 target graph and crate module graph | 0 violations | Buck2 graph; layering checker | PR | **[E]** Xiao et al. 2016; Zhou et al. 2009 |
| Architecture & deps | Propagation cost (reachability over the resolved graph, trait-object edges counted) | ratchet **[P]** | graph tool (this project) | PR | MacCormack et al. 2006 |
| Language & lints | Curated Clippy restriction set in non-test code: `unwrap_used`, `expect_used`, `panic`, `indexing_slicing`, `arithmetic_side_effects`, `as_conversions`, `cast_*`, `float_cmp`, `todo`, `unimplemented`, `dbg_macro`, `mem_forget`, `undocumented_unsafe_blocks`, `multiple_unsafe_ops_per_block` | 0 **[P]** | Clippy (never the whole `restriction` group); MISRA/CERT subset via clang-tidy or Cppcheck | PR | Rust project goals 2026; ISO 26262-6:2018 Table 1 1g (+ at A) |
| Language & lints | New suppressions (`allow`, `expect`, `NOLINT`) need a justification token; aggregate ratchet | ratchet | AST parser (attributes, not comments) | PR | metric-validity addendum (exact conformance count) |
| Language & lints | C/C++ coding-standard checks block only after their precision is measured on our code | measured before blocking **[P]** | Cppcheck / clang-tidy | NL | Hatton 2004, 2007 (≈1:42 real:false for MISRA C:2004 required rules) |
| Metrics (hygiene) | Function LOC ≤ p90 of a pinned benchmark (start 74 until re-derived); cognitive complexity no increase on changed functions; cyclomatic ≤ 25; distribution-shape check | caps/ratchets **[P]** | metrics tool | PR | Alves et al. 2010; Oliveira et al. 2014; P10 |
| Static analysis | Miri on every test reaching `unsafe`, Tree Borrows, pinned seed set; `cfg(miri)` ignores and `-Zmiri-disable-*` denied | 0 UB | Miri; ASan + UBSan | PR | **[E]** Jung et al. 2026 |
| Testing adequacy | Changed-region coverage | ≥ 80% **[P]** (see D4) | llvm-cov; Clang/gcov | PR | ISO 26262-6:2018 Table 9 (statement ++ at A; rationale needed below 100%) |
| Testing adequacy | Pseudo-tested functions on changed code (body replaced by default, all tests still pass) | 0 **[E]** | cargo-mutants (body replacement) / Buck2 mutation driver; Mull or source-level mutator | PR | Maton et al. 2024 |
| Testing adequacy | Diff-scoped mutation survivors listed; raw score reported, never gated | report | Buck2 mutation driver | PR | **[E]** Papadakis et al. 2018 |
| Testing adequacy | Hidden held-out tests with inputs generated from gate-owned seeds | 100% pass | proptest under sealed seeds; RapidCheck | PR | **[E]** Gabor et al. 2025; Zhong et al. 2025 |
| Testing adequacy | ≥ 1 round-trip property per parser/serialiser; property suites report rejection rate | present; rejection rate capped **[P]** | proptest, bolero | PR | Goldstein et al. 2024 |
| Testing adequacy | Executable exploit / negative tests for I/O-facing code | present | test harness | PR | **[E]** Vero et al. 2025 |
| Robustness | Flake gate: changed tests pass 5× in shuffled order and varied thread counts | 5/5 **[P]** | nextest-style runner / `buck2 test` | PR | Luo et al. 2014; Winters et al. 2020 (value collapses near 1% flakiness) |
| Resources & timing | Instruction-count regression on declared perf-sensitive targets | ≤ +2% **[P]** | gungraun (Callgrind); Cachegrind | PR | practitioner consensus (2026) |
| Resources & timing | Any performance claim backed by an end-to-end benchmark, not a micro-benchmark | required | bench harness | PR | **[E]** SpecBench (73.8% of kernel speed-ups had no end-to-end gain) |
| GPU/numerics | `spirv-val`; Vulkan validation layers clean on lavapipe (`LVP_POISON_MEMORY` on) | 0 errors | SPIRV-Tools, VVL | PR | wgpu CI (2026) |
| GPU/numerics | Differential vs CPU reference within a tolerance *stated in the spec*; N-run determinism hash identical; all-zeros output must fail | pass | harness (this project) | PR | Du et al. 2026; Jacobson et al. 2024 |
| GPU/numerics | IEEE special-value sweep (NaN, ±inf, −0, subnormals): hangs and NaN-in → finite-out fail | 0 | harness | PR | **[E]** Vanover et al. 2020 |
| GPU/numerics | ≥ 1 metamorphic relation per numerical kernel | ≥ 1 **[P]** | harness | PR | Kanewala & Chen 2019 (90% of mutants killed) |
| Toolchain & supply chain | Each checker classified (T2) with a known-limitations list | documented | qualification record (this project) | REL | **[S]** EN 50128:2011 6.7 |
| Anti-gaming/process | Holdout suite; decoy files in sandbox; protected-path changes get full canaries + solo merge; flagged diff patterns (new `PartialEq`/`Eq`/`Hash`/`Deref` impls, new `static mut`/global atomics, literal tables keyed on test values) | pass; flags route to holdouts | gate runner | PR | **[E]** Zhong et al. 2025; SpecBench; Roth et al. 2026 |
| Anti-gaming/process | LLM judge (optional): comments stripped, identifiers renamed, both orders, may add findings but never clear one | advisory | — | PR | Moon et al. 2026; Judge Reliability Harness 2026 |
| Evidence | + coverage report, survivor list, LLR traceability matrix | present | gate report | PR | — |

### 4.3 L3: core logic, HPC kernels, security boundaries

**Intended code:** core algorithms and data structures, numerical kernels, parsers of untrusted input, concurrency
primitives, anything other components trust. **Risk analogue:** ASIL B; SIL 2; DAL C; NASA Class B.

| Area | Gate | Threshold | Tool | Runs | Source |
|---|---|---|---|---|---|
| Requirements/spec | Bidirectional traceability: no public item without a requirement, no requirement without a test | 0 untraced | trace checker | PR | **[S]** NASA-STD-8739.8B (no untraced functionality) |
| Requirements/spec | Specs and tests authored in a separate agent context without write access to the code under test; sealed | enforced | sealed repo + identities | PR | P5; AMLAS (Hawkins et al. 2021) |
| Architecture & deps | Transitive `unsafe` reachability over the locked dependency graph: no new unsafe-reaching dependency unless audited | 0 unaudited | cargo-geiger-style scan + cargo-vet | PR | **[E]** Evans et al. 2020 (38% of crates reach `unsafe` only via deps); Xu et al. 2021 (185/186 memory-safety CVEs need `unsafe`) |
| Architecture & deps | Recursion reported from the call graph | report (gate at L4) | call-graph tool | PR | IEC 61508-3:2010 Table B.1 #6 |
| Language & lints | Zero new suppressions without allow-list entry | 0 | AST parser | PR | P8 |
| Language & lints | No `unsafe impl Send/Sync` over unbounded generics | 0 **[E]** | AST lint | PR | Xu et al. 2021 (30–36 of 186 CVEs); Cui et al. 2024 |
| Language & lints | Every `unsafe` block's `// SAFETY:` names ≥ 1 property from a fixed vocabulary covering that API's required properties | 100% | lint (this project) | PR | Cui et al. 2024 (19-property taxonomy) |
| Language & lints | `overflow-checks = true` in release; new `wrapping_*`/`saturating_*`/`as` flagged | on; flagged | Cargo/Buck2 profile check | PR | Qin et al. 2020; Li et al. 2024 |
| Language & lints | Coding standard (curated profile) zero findings | 0 | Clippy profile; MISRA/CERT subset | PR | **[S]** ISO 26262-6:2018 Table 1 1g (++ at B); IEC 61508-3 Table A.4 #5 (HR SIL 2) |
| Metrics (hygiene) | Fan-in/fan-out on the resolved graph | ratchet **[P]** | graph tool | PR | Arif et al. 2026 (split in/out coupling) |
| Static analysis | Any SAST tool crash fails; findings advisory; "finding disappeared after refactor" flagged | crash = fail | Rust SAST tools; Clang SA | PR | **[E]** Rust SAST evaluation 2026 |
| Static analysis | C/C++: sanitizers on the full test suite | 0 | ASan, UBSan, TSan | PR | — |
| Testing adequacy | 100% statement and branch coverage of changed code, or a deterministic disposition: declared deactivated configuration, or marked defensive code counted in a ratcheted budget; anything else fails | 100% **[S]** | llvm-cov (Rust branch coverage needs nightly **[U]**); Clang/gcov | PR | DO-178C Table A-7 decision coverage, level B (Hayhurst et al. 2001); ISO 26262-6:2018 Table 9 (branch ++ at B) |
| Testing adequacy | Zero unexplained surviving non-equivalent mutants on changed lines **plus** a fixed-seed sample of mutants in 1-hop dependents; TCE on survivors; equivalent-mutant exclusions are sealed records | 0 **[E]** | Buck2 mutation driver (schema engine where available); source-level C/C++ mutator | PR | Petrović et al. 2021, 2022; Ma et al. 2021; Ojdanic et al. 2022; Papadakis et al. 2015 |
| Testing adequacy | ≥ 1 assertion-attributed kill per changed function; pseudo-tested statements on changed code = 0 | ≥ 1; 0 **[E]** | mutation driver | PR | Zhang & Mesbah 2015; Maton et al. 2024 |
| Testing adequacy | Nightly seeded random full-repo mutation sample; distinguishing score reported next to raw | ≥ 1,000 mutants or 10%; weekly full visit **[P]** | mutation driver | NL | Gopinath et al. 2016; Shin et al. 2018 |
| Testing adequacy | Differential testing against a sealed reference or executable spec on generated inputs | 0 divergences | harness | PR | **[E]** Wang et al. 2026 (PatchDiff) |
| Testing adequacy | Model/differential property per data-structure module | present **[P]** | proptest | PR | Goldstein et al. 2024 |
| Formal verification | Kani harness for every `unsafe` block and every arithmetic/indexing kernel: no panic, no UB, no overflow (bounded); unwind bounds and `assume` budget in sealed policy; `cover!` reachability | all SUCCESSFUL | Kani; CBMC (unwinding assertions on) | PR (sharded) | **[E]** Cook et al. 2026; Chong et al. 2021 |
| Robustness/fuzz/concurrency | Fuzz harness for every parser/untrusted-input API: corpus replay + fixed budget per target; crashes deduplicated by root cause | 0 crashes; 60 s/target **[P]** | cargo-fuzz / bolero; libFuzzer, AFL++ | PR; longer NL | Klees et al. 2018; Böhme et al. 2022 |
| Robustness/fuzz/concurrency | Lock-free primitives: exhaustive bounded interleavings; other concurrent code: randomized/PCT schedules | 0 failures | loom; shuttle; (C/C++: GenMC, 2–3 threads) | PR | Kokologiannakis & Vafeiadis 2021 |
| Simulation/determinism | Stateful/distributed components: deterministic simulation; same seed twice → byte-identical trace hash; N seeds per PR; clocks, RNG, spawning and `HashMap` seeding banned in sim-tagged crates | identical; N **[P]** | turmoil + libc overrides, or sans-IO state machines; Clippy `disallowed_methods` | PR; many seeds NL | S2 (2025); Polar Signals (2025) |
| GPU/numerics | Tolerances *derived* from order-independent error bounds and builtin ulp tables; precision reductions (f32→f16/bf16, FMA contraction) need a sound bound | derived | bound calculator (this project); FPTaylor for straight-line code | PR | **[E]** Muller et al. 2018; Isychev & Lohar 2025 (dynamic tuners up to 10⁹× over target) |
| GPU/numerics | Multi-backend differential (naga vs Tint; lavapipe vs SwiftShader/WARP); K permuted work-group orders → identical hash; reproducible or fixed-order reductions | 0 disagreement **[P]** for K | harness | PR | Donaldson et al. 2023; Lidbury et al. 2015; Peng & Rajan 2019 |
| GPU/numerics | ≥ 3 metamorphic relations from distinct categories, each with its own survivor gate | ≥ 3 **[E]** | harness + mutation | PR | Chen et al. 2018 (3–6 diverse MRs reach ≥ 90% of oracle-level detection) |
| GPU/numerics | Indices that depend on values written by other invocations: lint, plus race-freedom argument | 0 unlisted | lint (this project) | PR | Levine et al. 2025 (SafeRace) |
| GPU/numerics | GPU-specific mutation (index ±1, barrier/scope removal, launch config, precision) | survivors listed | GPU mutation harness | GPU | Zhu & Zaidman 2020 |
| Toolchain & supply chain | Every dependency audited or imported-trusted | 0 unaudited | cargo-vet | PR | practitioner consensus (2026) |
| Toolchain & supply chain | Each verification tool validated by its canary pack (seeded-fault suite); compiler flags recorded with every artefact | all canaries red | gate runner | PR | **[S]** ISO 26262-8:2018 cl. 11 method 1c; P4 |
| Anti-gaming/process | Trajectory log checked by deterministic policy checkers; one detected hack disqualifies the session; hidden inputs generated by renaming/permuting visible cases | enforced | gate runner | PR | **[E]** SWE-CC 2026; Roth et al. 2026; RLVR verifier-gaming study 2026 |
| Evidence | + kill matrix, Kani results, fuzz corpus hash, trajectory hash, tolerance derivations | present | gate report | PR | — |

### 4.4 L4: high-integrity components

**Intended code:** components whose failure is hard to detect downstream and expensive: financial ledgers, control
loops, safety monitors, verified kernels used by L4+ parents. **Risk analogue:** ASIL C; SIL 3; DAL B; NASA
safety-critical.

| Area | Gate | Threshold | Tool | Runs | Source |
|---|---|---|---|---|---|
| Requirements/spec | Frozen spec in the sealed store; each hazard-derived constraint maps to a proved contract or a requirements-based test; recorded environment assumptions (a changed deployment context invalidates the level) | 100% mapped | trace checker | PR | Leveson 2011 (via standards addendum); Hatcliff et al. 2014 |
| Requirements/spec | Spec-strength checks: satisfiable `requires`; non-tautological `ensures`; stub implementation fails; proved on frozen concrete I/O; rejects frozen negative-output suite; `decreases` present | all pass **[E]** | Verus / Kani harness generators (this project) | PR | Bai et al. 2026; Lahiri 2024; Leino 2023; Aggarwal et al. 2024 |
| Requirements/spec | Human/external review of the frozen spec | **open decision D1** | — | REL | Leveson 2011; IEC 61508-1 (via Smith & Simpson 2020) |
| Architecture & deps | Boundary evidence for every lower-level dependency: spatial (no shared mutable state, no `unsafe` reachable across the boundary), temporal (bounded execution), communication (validated schema) | 100% | manifest + analyses | PR | **[S]** ISO 26262-6:2018 7.4.9, Annex D |
| Architecture & deps | No recursion; static stack bound from the call graph | 0; bound ≤ budget | call-graph + stack tool | PR | **[S]** IEC 61508-3:2010 Table B.1 #6 (HR SIL 3); ISO 26262-6:2018 Table 6 1j (++ at C); McCormick & Chapin 2015 (RTE proof does not cover stack overflow) |
| Architecture & deps | No allocation after initialisation in declared real-time paths; loops statically bounded | 0 | lint + Kani `loop_decreases` | PR | ISO 26262-6:2018 (dynamic objects ++ from B; applied from L4 here, a documented relaxation **[P]**); Holzmann 2006 |
| Architecture & deps | `unsafe` only in modules with proofs; `panic = "abort"` in release or panic-safety evidence | 0 unproved | manifest + build profile | PR | Xu et al. 2021 (13 unwinding CVEs) |
| Language & lints | No fast-math, FMA contraction or reassociation in proved modules unless the error bound accounts for it | 0 | flag checker | PR | Monniaux 2008; Muller et al. 2018 |
| Metrics (hygiene) | Cyclomatic ≤ 15 per function (hygiene cap); spec size per module tracked as proof-cost indicator | cap **[S]**; advisory | metrics tool | PR | NPR 7150.2D SWE-220; Matichuk et al. 2015 |
| Static analysis | C islands: sound absence of runtime errors with zero alarms, hashed analysis config, alarm-class switches deny-listed, analysis statement coverage with no unexplained dead code | 0 alarms | Frama-C Eva or Astrée | NL | Kirchner et al. 2015; Cousot et al. 2005 |
| Static analysis | Lock-order / double-lock analysis | 0 | lock-graph checker | PR | Qin et al. 2020 (30 of 59 blocking bugs are double locks) |
| Testing adequacy | C/C++ islands: masking MC/DC measured on a canonical (inlined-temporaries) form with an output-level oracle; gap between as-written and canonical MC/DC reported; compiler, version and flags pinned | ≥ 95% **[P]** (standards: R/+ at SIL 3/ASIL C) | Clang `-fcoverage-mcdc`; GCC `-fcondition-coverage` at pinned `-O0` as cross-check | PR | Gay et al. 2016; Rajan et al. 2008; Weiss & Schulz 2026; IEC 61508-3 Table B.2 #7d; ISO 26262-6 Table 9 |
| Testing adequacy | Rust: condition-operand mutation substitute, all killed at an asserted output on changed code + 1-hop dependents; labelled "substitute" | 0 survivors | mutation driver | PR | Meng et al. 2018; Ammann & Offutt 2016 |
| Testing adequacy | Subsuming (dominator) mutation score on the component, from a kill matrix with visible + holdout tests | ≥ 90% **[P]** | mutation driver | NL | Papadakis et al. 2018; Ammann et al. 2014; Kurtz et al. 2016 |
| Testing adequacy | Requirements-based integration tests on the release binary | 100% pass | `buck2 test` on release config | PR | Ross 2016 (unit coverage misses compiler-setting faults) |
| Formal verification | Absence of panic/overflow/out-of-bounds proved on the public API (SPARK-Silver analogue) | proved | Verus, Flux or Prusti; Frama-C WP with `-wp-rte` | PR | **[S]** EN 50128:2011 Table A.5 formal proof (HR SIL 3); Lattuada et al. 2024; Astrauskas et al. 2019 |
| Formal verification | Kani function and loop contracts on public APIs; `stub_verified` composition | all SUCCESSFUL | Kani; CBMC | PR | Cook et al. 2026 |
| Formal verification | Escape-hatch ledger: budgeted, non-increasing, each with sealed waiver; spec-freeze AST diff on proof-only changes; no-cheating verifier mode; `rlimit`; unknown/timeout fails; no proof caching across changes | as stated **[E]** | ledger + AST diff (this project) | PR | VeruSAGE (Yang et al. 2025); Xie et al. 2026; proof-stability study 2026 |
| Formal verification | Concurrent/distributed designs: design model with fixed constants, mandatory liveness, action coverage | pass | TLA+/TLC, Apalache, P | PR (small); NL (large) | Newcombe et al. 2015 |
| Formal verification | Alternative where code is too tuned to prove: proved model + deterministic differential testing of the implementation (gap recorded) | DRT 0 divergences | Lean or Verus model + DRT harness | PR | Disselkoen et al. 2024 (proofs found 4 bugs, DRT/PBT 21) |
| Robustness/fuzz/concurrency | Lock-free code: ordering mutation (weakening any `Ordering` must make a check fail) | 0 survivors | loom; GenMC (SC/TSO/RC11, not ARM/Power) | PR (2–3 threads); NL (4) | Kokologiannakis et al. 2019, 2021 |
| Robustness/fuzz/concurrency | Flake gate N = 20 with sanitizers; Miri many-seeds; continuous offline fuzz campaign | 20/20 **[P]** | Miri, TSan, fuzzers | NL | Luo et al. 2014 |
| Simulation/determinism | Deterministic simulation with fault injection mandatory for stateful components | pass | DST kit | PR; NL | S2 (2025); TigerBeetle VOPR |
| Resources & timing | Timing budget: static instruction/stack proxies on PR; measured timing only on pinned self-hosted hardware with confidence intervals | CI fully inside budget **[P]** | gungraun; bench on GPU machine | PR; GPU | Kalibera & Jones 2013 |
| GPU/numerics | Only **structured CUDA** kernels may be L4 (§5.2): race and barrier-divergence freedom proved; PTX equivalence to a reference on shapes that are not tile multiples, including boundary blocks; preconditions checked for satisfiability against a frozen caller; ITS-aware warp model; compensated or binned reproducible reductions | all proved; unknown = fail | Faial / GPUVerify; VOLTA; ProofWright-style pipeline | PR (CPU); GPU (dynamic) | Cogumbreiro et al. 2023; Dubey et al. 2025; Chatterjee et al. 2025; Collange et al. 2015 |
| GPU/numerics | GPU mutation with justified survivors | ≥ 95% **[P]** | GPU mutation harness | GPU | Zhu & Zaidman 2020 |
| Toolchain & supply chain | Hermetic remote-executed build, built twice, output hashes compared | identical | Buck2 RE | NL | Ferrous Systems 2026 (safety-manual build constraints) |
| Toolchain & supply chain | Every verification tool has a qualification record: pinned version, use cases, known limitations, seeded-fault validation results | complete | qualification records | REL | **[S]** ISO 26262-8:2018 cl. 11 (TCL2/3, method 1c) |
| Toolchain & supply chain | Qualified compiler optional at L4 | open decision D9 | Ferrocene | REL | Ferrous Systems 2026 |
| Anti-gaming/process | Defeater stage: adversarial counterexample search against each claim; an open defeater blocks; agent setup pinned with the toolchain and a proof canary rerun on every bump | 0 open defeaters | gate runner | NL | Bloomfield & Rushby 2020; Yang et al. 2026 (ExVerus) |
| Anti-gaming/process | Oracle authors (spec, tests) from a different model family than the code author; credited as defence in depth only | enforced **[P]** | identities | PR | Nogueira et al. 2026 |
| Evidence | Evidence bundle with a trusted-computing-base manifest (compiler, verifier, solver, trusted specs, external crates, hardware models) and escape-hatch ledger | present | gate report | REL | Lattuada et al. 2024 (trusted specs up to ~1.6K lines) |

### 4.5 L5: proven components

**Intended code:** small, CPU-side components where a failure is unacceptable and unlikely to be caught: the parent
monitor of a decomposition, a core verified kernel, a safety-relevant decision. **Risk analogue:** ASIL D; SIL 4;
DAL A; NASA Class A safety-critical. **Claim:** "verified against the frozen spec", with the residual gaps listed.

| Area | Gate | Threshold | Tool | Runs | Source |
|---|---|---|---|---|---|
| Requirements/spec | Two-sided spec validation: spec matches the code's behaviour *and* rejects illegal states; spec proved equivalent or refinement-related to a frozen reference spec where one exists | pass | Verus/Lean + negative suites | PR | Cheng et al. 2026 (Specula); Bai et al. 2026; Thakur et al. 2025 (≤ 1.9% agent equivalence: do not rely on agents for this) |
| Requirements/spec | Independent spec provenance: spec agent isolated from implementation and proof agents; back-translation checks advisory only | enforced | identities | PR | Amster et al. 2026 (claimcheck, non-deterministic) |
| Requirements/spec | Human/external assessment | **open decision D1** | — | REL | IEC 61508-1 (independent organisation at SIL 4, via Smith & Simpson 2020) |
| Architecture & deps | CPU only; code restricted to the certified `core` subset (leaving it is a lint error) | 0 | Ferrocene `unvalidated` lint | PR | Ferrous Systems 2026 |
| Architecture & deps | `unsafe` forbidden, or proved with an unsafe-capable verifier; bounded Kani proofs alone are insufficient | 0 unproved | RefinedRust (`Print Assumptions` empty), VeriFast, Verus permissions | NL | Gäher et al. 2024; McCormick & Chapin 2015 |
| Architecture & deps | Design proof with safety and liveness, linked to code by trace validation or refinement; every counterexample reproduced in real code | pass | TLA+/P + trace validation; VerusSync-style refinement | NL | Cheng et al. 2026; Newcombe et al. 2015 |
| Testing adequacy | Requirements-based integration tests kept (proof covers source, not object code or HW/SW integration); contracts tested before being proved | 100% pass | `buck2 test` | PR | **[S]** DO-333 (via Moy et al. 2013; Pothon & Ochem n.d.) |
| Testing adequacy | 100% of non-equivalent mutants killed; every equivalence claim discharged by a machine-checked bounded equivalence proof, otherwise it counts as a survivor | 100% **[P]** | mutation driver + Kani/CBMC equivalence harness | NL | Ammann & Offutt 2016 (equivalence undecidable); test-adequacy report |
| Testing adequacy | C islands: MC/DC 100% or sealed per-decision justification; Rust: "MC/DC objective not met; substituted by condition mutation + proof" stated in the bundle | 100% **[S]** | Clang MC/DC | PR | DO-178C Table A-7, level A (Hayhurst et al. 2001); NPR 7150.2D SWE-219 |
| Testing adequacy | Extraneous-code check by coverage-based dead-code detection, declared as a substitute for the DO-333 activity | 0 unexplained | llvm-cov | PR | Moy et al. 2013 |
| Formal verification | Full functional correctness of every non-trivial function | proved | Verus; Aeneas → Lean for pure kernels; Frama-C WP for C islands | PR | **[S]** IEC 61508-3:2010 Table A.9 formal proof (HR SIL 4); Lattuada et al. 2024; Ho & Protzenko 2022 |
| Formal verification | Zero escape hatches outside one signed boundary file; standard axioms only | 0 | ledger | PR | McCormick & Chapin 2015; VeruSAGE |
| Formal verification | Proof robustness: still passes at 0.5× `rlimit` and under multiple solver seeds | pass **[P]** | verifier | NL | formal-verification report |
| Formal verification | DO-333-style structural replacement: complete and disjoint behaviours, data-flow contracts, assumption ledger | present | Verus/Frama-C | PR | Moy et al. 2013 |
| Toolchain & supply chain | Qualified compiler (TCL3 via methods 1b + 1c) or object-code verification for code not traceable to source; compiler-inserted checks traced or object-code coverage measured | required | Ferrocene; binary coverage | REL | **[S]** ISO 26262-8:2018 Tables 4–5; ECSS-E-ST-40C 5.8.3.5e (via AdaCore 2021) |
| Toolchain & supply chain | Translation validation of the LLVM stage as a *reported* gate (validated/violated/timeout/unsupported fractions; any violation fails); differential compilation across two compiler versions or opt levels; binary validation where feasible | 0 violations | Alive2; differential build | REL | Lopes et al. 2021; Klein et al. 2014 (binary validation full at -O1, 79% at -O2) |
| Toolchain & supply chain | C islands: CompCert (no integer↔pointer casts) or binary validation with coverage reported | required | CompCert | REL | Leroy 2009; Klein et al. 2014 |
| GPU/numerics | **No GPU code at L5.** A GPU result may feed L5 only through an L5 CPU-side result checker (certifying algorithm) at the parent | 0 GPU targets | manifest | PR | §5.2 |
| Anti-gaming/process | Any waiver blocks release unless covered by a pre-registered decision | 0 unregistered | gate runner | REL | NPR 7150.2D waivers |
| Evidence | Generated assurance argument (GSN/CAE-style) linking every claim to hashed CI artefacts, with the residual-gap list (MC/DC substitute, trusted base, unverified environment assumptions) | present | evidence generator (this project) | REL | Kelly 2004; Hawkins & Kelly n.d.; Rushby 2015 |

### 4.6 What the ladder does not claim

- No controlled study shows that following a level-*n* technique set lowers defect rates relative to level *n*−1; the
  evidence is observational, expert-based or self-reported (Nair et al. 2014: 72% of 218 safety-evidence studies
  unvalidated in realistic settings; Ferrari & ter Beek 2022: 1.5% of 328 rail formal-methods studies were full case
  studies).
- Ultra-high reliability cannot be shown by testing at all (Butler & Finelli 1993), which is why L5 rests on proof.
- Real certification needs a qualified toolchain and an independent assessor. The tool produces evidence mapped to
  ISO 26262-6 and DO-178C tables, not certificates.

---

## 5. Language and platform matrix

### 5.1 Rust vs C/C++ per capability (late 2026)

| Capability | C/C++ best available | Rust best available | Verdict |
|---|---|---|---|
| MC/DC coverage | Clang `-fcoverage-mcdc` (masking); GCC 14+ `-fcondition-coverage` | none (removed 2025; re-implementation planned as unstable) | **C/C++ ahead.** Rust uses a declared substitute |
| Coding standard / MISRA | MISRA C/C++ checkers (Cppcheck Premium, clang-tidy partial, commercial) | curated Clippy profile; SCRC guidelines incomplete | **C/C++ ahead** on coverage of rules; Rust's type system removes many rule targets |
| Sound whole-program absence of runtime errors | Astrée (0 false alarms at 400 kLOC; Cousot et al. 2005), Frama-C Eva | none; per-function deductive (Prusti, Flux, Verus) or bounded (Kani) | **C/C++ ahead** |
| Sound FP range/round-off | Astrée FP domains; FPTaylor (straight-line) | none | **C/C++ ahead** |
| Bounded model checking | CBMC, ESBMC | Kani (on CBMC) | **Parity** |
| Deductive functional correctness, SMT-automated | Frama-C WP (annotations ≥ code, up to ~6:1 at EAL7; Kosmatov et al. 2024) | Verus (5.1:1 proof:code on systems; 6–262 s per project), Prusti, Creusot | **Rust ahead** on automation and cost |
| Foundational verification of deployed artefacts | seL4, CompCert | RefinedRust, Aeneas, Gillian-Rust (research) | **C ahead** |
| Dynamic UB detection incl. aliasing | ASan/UBSan/MSan/TSan (no aliasing model) | Miri (Tree/Stacked Borrows, provenance, races, weak memory) | **Rust ahead** |
| Weak-memory / lock-free verification | GenMC (RC11/IMM) | loom, shuttle, Miri (partial) | **C/C++ ahead** |
| Verified compiler / translation validation | CompCert; Alive2 on LLVM | none; Alive2 plausibly applies to rustc's LLVM stage (inference) | **C ahead** |
| Qualified compiler | qualified GCC/Clang variants, CompCert | Ferrocene (ASIL D, SIL 3, IEC 62304 C; `core` subset SIL 2/ASIL B); GNAT Pro for Rust | **Parity for CPU targets** |
| Mutation testing | Mull (last release Apr 2025, LLVM 18) | cargo-mutants (active), mutest-rs (research, nightly) | **Rust ahead** |
| Published LLM proof-writing evidence | little (Frama-C/ACSL) | Verus: 81% proof completion with frozen specs | **Rust ahead** (Dafny ahead of both) |
| SAST | mature, noisy | recall ≤ 6% on real CVEs | **neither is evidence**; advisory |

**Policy.** Rust everywhere by default. A C/C++ island is justified when (a) the component is L4–L5 *and* needs MC/DC,
sound whole-program RTE analysis with FP, weak-memory model checking or a verified compiler; or (b) it is a GPU kernel
that needs L4 evidence (CUDA, §5.2); or (c) native performance is clearly better and the island can still meet its
level. Every island sits behind a verified FFI boundary (P2) and inherits the gates of its level with the C/C++ tools
named in §4.

### 5.2 GPU level caps

| GPU stack | Maximum level | Why | Conditions to reach the cap |
|---|---|---|---|
| WGSL / naga / rust-gpu / CubeCL→WGSL | **L3** | No static race verifier, coverage tool, mutation tool or deductive verifier exists for WGSL, SPIR-V, naga IR or rust-gpu; dynamic race detectors are NVIDIA-only; software rasterisers likely cannot show weak-memory races (inference) | All L3 GPU gates: derived tolerances; multi-backend differential (naga vs Tint; lavapipe vs SwiftShader/WARP); permuted work-group orders with identical hashes; reproducible reductions; ≥ 3 MRs; race-dependent-indexing lint; GPU mutation on the self-hosted machine; toolchain versions pinned and the fuzz-qualified version recorded |
| CUDA, structured kernels (barrier-synchronised, affine indexing, no atomics, no data-dependent control flow, fixed shapes) | **L4** | Sound GPU-free evidence now exists for this fragment: Faial (96% of 227 real kernels), GPUVerify (69% automatic), VOLTA PTX equivalence incl. tensor-core GEMM/FlashAttention, ProofWright (74/100 kernels memory-safe and race-free) | Proofs of race and barrier-divergence freedom; PTX equivalence to a frozen reference on non-tile-multiple shapes including boundary blocks; solver unknown/timeout = fail; agent-written preconditions checked for satisfiability against a frozen caller; ITS-aware warp model (pre-Volta lock-step tools certify code that is now UB); "race-free" means scope-aware (HRF) race-free (Nagarajan et al. 2020); compensated/binned reductions; dynamic race detection and GPU mutation on the self-hosted machine as extra evidence |
| CUDA, other (atomics, warp-synchronous, cooperative groups, data-dependent indexing) | **L3** | Outside every verifier's fragment | As for WGSL, plus dynamic race detection on the GPU machine with fixed input sizes and repetition counts (recall depends on input size; SuperCollider 2026) |
| Any GPU code | **not L5** | No qualified GPU compiler or driver; no verified GPU compilation; VOLTA checks one shape and only the first block and reasons over reals; no composed FP-bound equivalence; no target-GPU litmus qualification in the stack | GPU results reach L5 only through an L5 CPU-side result checker |

Notes: rust-cuda and NVIDIA's experimental Rust-to-CUDA compiler are early-stage (Rust-GPU org 2025; Phoronix 2026),
so in practice the L4 CUDA path means CUDA C++ kernels or PTX-level evidence on whatever produced the PTX (inference).
CubeCL's CPU backend gives a GPU-free execution path for the *same* kernel source and is useful for L2–L3 differential
tests. Ferrocene has no GPU targets.

---

## 6. CI architecture on free runners

### 6.1 Two lanes, one source of truth

```
                 sealed gate repo (pinned hash): manifest, lint profiles, operators, seeds,
                 tolerances, baselines, canaries, metric registry
                                   │ read-only
 candidate PR ──► S0 policy (<5 s, no build) ──► S1 build + lints ──► S2 canaries ──► S3 tests + coverage
                                                                                        │
                    ┌───────────────────────────────────────────────────────────────────┤
                    ▼                                                                   ▼
        Lane A: Buck2 (fast, cached)                                  Lane B: Cargo (verification tools)
        build, Clippy, tests, coverage config,                        Kani, Miri, cargo-fuzz/bolero,
        Buck2-native mutation driver, gungraun,                       cargo-deny/vet, Verus,
        shader validation, lavapipe/WARP tests                        cargo-mutants fallback
                    └───────────────► S4 per-level deep checks ◄────────┘
                                             │
                    S5 holdouts (triggered outside the repo's workflows) ──► gate report ──► merge bot
                                             │
            nightly: full mutation sample + kill matrix, Miri seed sweeps, fuzz campaigns, large model checks
            GPU machine (pull model): dynamic races, GPU mutation, real-device differential, timing
```

- **Lane A (Buck2)** carries the fast loop. Coverage is a Buck2 configuration adding `-C instrument-coverage`, with
  `llvm-profdata` merge and `llvm-cov export` as separate cacheable actions; llvm tools must match rustc's LLVM major
  version.
- **Lane B (Cargo)** carries tools that are Cargo-bound today (Kani, Miri, cargo-fuzz, cargo-deny, cargo-vet).
  `Cargo.toml` per crate is the dependency truth; `BUCK` files are generated from it (e.g. reindeer) and CI fails if
  regeneration produces a diff.
- **Stages run cheapest first**; a failure stops later stages. Target budgets **[P]**: L1 ≤ 10 min, L2 ≤ 15 min,
  L3–L4 ≤ 45 min wall clock per PR, achieved by sharding across up to 20 concurrent jobs (the GitHub Free concurrency
  limit **[U]**).
- **The same gate runs locally** for agents, read-only against the sealed config, so the inner loop sees the same
  verdict as CI.

### 6.2 Affected-target selection

`buck2 uquery "rdeps(//..., owner(<changed files>))"` (or `cquery` for configured graphs) chooses tests, mutants, Kani
harnesses and Miri tests per PR. Target-level selection over declared inputs is safe only if inputs are complete, so
L3+ checks prefer remote-executed (sandboxed) actions: Buck2 does not sandbox local-only actions, while remote
execution is hermetic by design (Buck2 docs). A full run on the default branch nightly backstops selection; ML-based
test selection is used only for *ordering*, never for pruning (Machalica et al. 2019; Memon et al. 2017).

### 6.3 Mutation scoped to changed lines and dependents

1. `cargo mutants --list --json --in-diff` (JSON diffs since 26.2.0) enumerates candidate mutants on changed lines;
   a fixed-seed sample of mutants in 1-hop dependents is added (Ojdanic et al. 2022).
2. The Buck2-native driver applies each diff to a scratch overlay and runs only the affected `buck2 test` targets;
   parallelism comes from Buck2 or remote execution. Nothing public does this today; it is a planned component.
3. Survivors are compiled individually at a pinned optimisation level and hashed for TCE.
4. Arid-line filtering uses only sound, type-based rules from the sealed config, and the suppressed count is reported
   (Petrović et al. 2022 reduced a median 820 mutants per change to 7 this way).
5. Where a schema engine is available (mutest-rs design: one compile, runtime switch), it replaces steps 1–2 for
   whole-crate nightly runs; it pins a nightly rustc, which is handled as a separate hermetic toolchain.

### 6.4 Performance gates on noisy runners

Wall-clock noise on shared VMs is several percent or more, useless for regression gates. PR gates use **instruction
counts and cache-simulation estimates** (gungraun, formerly iai-callgrind; Cachegrind), which are deterministic and
free. Valgrind may not support every AVX-512 instruction the kernels use **[U]**. Wall-clock and GPU timing run only on
pinned self-hosted hardware, with repetitions dimensioned per platform and a fail condition of "the 95% confidence
interval of the time ratio lies entirely above 1 + δ" (Kalibera & Jones 2013). CodSpeed's free OSS tier (600
bare-metal minutes per month) is an optional alternative for CPU wall-time trends.

### 6.5 GPU testing without GPUs, and the self-hosted GPU machine

| Tier | Where | What |
|---|---|---|
| G0 validation | PR | naga validation, `spirv-val`, `spirv-opt` round trip, Vulkan validation layers |
| G1 software execution | PR | Linux: pinned Mesa lavapipe build (not the distro package); Windows: WARP + lavapipe; macOS hosted Metal is a paravirtual device lacking simdgroup support, so smoke tests only (wgpu CI 2026) |
| G2 CPU-side static | PR | CUDA: Faial/GPUVerify race and divergence proofs, VOLTA PTX equivalence (PTX is produced without a GPU) |
| G3 real GPU | GPU | dynamic race detection with fixed input sizes, FP-exception detection, GPU mutation, real-device differential, timing, memory-model litmus qualification of the target device (MC Mutants approach; Levine et al. 2023) |

**Pull model for the self-hosted GPU machine.** GitHub advises against self-hosted runners on public repositories
because fork PRs can run arbitrary code on them. So the GPU machine never registers as a runner. It polls for commits
on the default branch or merge-queue refs that have already passed the free-runner gates, runs the GPU suite in a
throwaway container with an egress allow-list, uploads its evidence, and posts a commit status using a token scoped
only to writing statuses. It never runs fork code, never holds the loosening key and accepts no inbound jobs. For
levels that require G3 evidence, the merge bot waits for that status before the result counts.

### 6.6 Caching options for a public repository

Buck2's persistent action cache lives in a Remote Execution API backend; `buck-out` alone is not a reliable
cross-run cache.

| Option | Cost | Notes |
|---|---|---|
| Self-hosted NativeLink, cache-only | free software (Apache-2.0) + a machine | full control; best fit for strict trust boundaries |
| BuildBuddy Personal tier | free (up to 80 Linux RE cores, 100 GB transfer; as of Oct 2026) | zero-ops fallback; remote execution runs on third-party infrastructure |
| Ephemeral cache sidecar persisted via `actions/cache` | free | ~10 GB per-repo cap **[U]**; slowest |

Rules for all options: default-branch and same-repo jobs get read-write access; **fork PRs get read-only access or
none**, so untrusted code cannot poison the cache. Cache keys include toolchain and gate-config hashes. Disk on free
runners (14 GB) is the binding constraint for LLVM, Mesa and Kani toolchains plus outputs; a remote cache and removal
of preinstalled SDKs are needed.

The project's Buck2 fork ([Mivr/buck2](https://github.com/Mivr/buck2), branch `rigor/main`; see its `FORK.md`) adds
three things on top of upstream:

- **One daemon shared across compatible git worktrees.** Many agent worktrees building the same graph share one
  `buckd` and its DICE state, instead of each paying daemon start-up and a separate graph.
- **Action rewinding.** When remote execution reports inputs missing from the CAS, as happens when small free cache
  tiers evict entries, the producing actions are re-run instead of the build failing.
- **Clean-stale fixes.** `clean-stale` never deletes outputs of local actions that are still running.

### 6.7 Fork-PR and credential security

- Never combine `pull_request_target` with a checkout of the PR head.
- No secrets in PR workflows; holdout runs are triggered by an app outside the candidate's workflows.
- Agent tokens: contents and pull-request write only; no administration, no workflow editing.
- The merge bot is the only identity allowed to update the default branch.
- Gate config is read by hash from the sealed repository; gate flags are passed on the command line.
- The loosening key is hardware-backed and held by the owner (open decision D2).

---

## 7. Open decisions for the owner

### D1. Where humans are still required at L4–L5

**Evidence.**

- Agents complete about 81% of proofs against frozen expert specs (VeruSAGE), but proving an agent-written spec
  equivalent to a reference succeeds at most 1.9% of the time (CLEVER); end-to-end spec + code + proof sits at 1–5%
  (VERINA; VeriContest); only 42.3% of verifier-accepted agent specs pass validation (VerusEval).
- Most software-related accidents trace to requirement flaws, and expert review finds most safety-specification
  errors (Leveson 2011). Requirement elicitation and validation is the least mature area (Hatcliff et al. 2014). Failed
  IEC 61508-3 assessments scored lowest on requirements, not coding (Lloyd & Reeve 2009).
- IEC 61508-1 requires assessment by an independent department at SIL 3 and an independent organisation at SIL 4
  (via Smith & Simpson 2020); LLM-authored, unreviewed code plausibly counts as a "novel design", which raises the
  required degree at SIL 2–3.
- An agent pipeline cannot fill an independent-assessor role (P5).

**Options.**

- **(a)** Human or external review of the **frozen spec only** (hazard-derived constraints, requirements, environment
  assumptions) at L4–L5. Code, tests and proofs stay agent-only and are checked deterministically. The review is a
  signed record in the sealed store, invalidated by any spec change.
- **(b)** Agent-only L4–L5, with every claim limited to "verified against spec"; no statement about safety or fitness
  for purpose; the bundle lists spec validation as an unmet independence objective.

**Recommendation: (a)**, because the residual risk at L4–L5 sits in the spec, which is exactly where agents are
weakest and where standards and accident data put the human. It keeps humans off the code and costs one review per
spec change. **Status: open.**

### D2. Root of trust for loosening

Without a human-held, hardware-backed signature, some agent can always loosen a gate (P8). Options: owner hardware key
(recommended); multi-party signature; none (accept that the ladder is advisory). **Open.**

### D3. Mutation thresholds at L4–L5

L4 subsuming score ≥ 90% and L5 100% of non-equivalent mutants with proved equivalences are **[P]**. The evidence
supports "near saturation" (Papadakis et al. 2018; Chekam et al. 2017) but no study validates a number. **Open.**

### D4. Coverage floors at L1–L2

L1 report-only and L2 ≥ 80% changed-region coverage are **[P]**. Standard-aligned alternative: 100% with deterministic
dispositions from L2 (ISO 26262-6 statement ++ at ASIL A). Trade-off: higher floors push agents toward
coverage-directed tests (Gay et al. 2015), mitigated by the assertion-kill and pseudo-test gates. **Open.**

### D5. MC/DC at L4 for C islands

≥ 95% **[P]** vs 100% (NASA SWE-219 requires 100% for safety-critical; IEC 61508 and ISO 26262 only recommend it at
SIL 3/ASIL C). **Open.**

### D6. Rust at L5 without MC/DC

Accept the declared substitute (condition mutation + proof, labelled as not equivalent), or require that L5 components
needing a structural-coverage objective be written as C islands (CompCert + Frama-C + Clang MC/DC) until rustc MC/DC
returns. **Open**; recommendation: accept the substitute for components with full functional proofs, require C islands
only where an external certification target demands MC/DC.

### D7. Hygiene thresholds and their benchmark corpus

Which pinned corpus re-derives LOC/complexity percentiles (top crates plus own code at a frozen commit, or own code
only)? Starting values (function LOC 74, cyclomatic 25 at L2, 15 at L4) are convention until re-derived. **Open.**

### D8. Runtime budgets and repetition counts

Flake repetitions (5 at L2, 20 at L4), fuzz budget (60 s per target per PR), work-group permutations *K*, DST seeds per
PR, instruction-count tolerance (+2%), per-PR wall-clock budgets. All **[P]**. **Open.**

### D9. Qualified compiler at L4

Ferrocene (€240 per user per year; CI use reported free) is required at L5. At L4 it is optional; requiring it adds a
dependency and a lag behind upstream Rust. **Open.**

### D10. GPU policy

Accept the caps in §5.2, including an L4 path that depends on NVIDIA/CUDA tooling? Or cap all GPU code at L3 and
require CPU-side result checkers for anything higher? **Open**; recommendation: accept the caps, with the CUDA L4 path
used only where performance makes a CPU checker impractical.

### D11. Model-family diversity for oracle authors

Require a different model family for spec/test authors at L4+ (as in §4.4), from L3, or never? Evidence says diversity
helps but only partially (Nogueira et al. 2026). **Open.**

### D12. Level-assignment authority and scales

Who assigns S, E and C, and are the scales in P1 right for the owner's domains (HPC, services, tools)? Should
LLM-authored code be treated as a "novel design" that raises the level by one step at L2–L3 (the IEC 61508-1 uplift)?
**Open.**

### D13. Holdout authorship and rotation

Separate red-team agent pool with write access only to the holdout store (proposed), rotation 20% per quarter, rate
limit per PR. **Open.**

### D14. Cache provider

Self-hosted NativeLink (control) vs BuildBuddy free tier (zero ops). **Open.**

---

## 8. Implementation roadmap

Efforts are rough, for one engineer working with agents; they will move.

| Phase | Scope | What the tool does at the end | Effort |
|---|---|---|---|
| **1. Foundation (L0–L1)** | Manifest schema (components, levels, hazard profiles); dependency-rule checker on the Buck2 graph; sealed gate repository pinned by hash; gate report (JSON envelope + SARIF); ratchet engine with monotonicity proof and signed loosening; canary runner; fail-closed test-result parser; test-integrity checks; curated lint profiles; free-runner workflows; local gate command | Assigns levels, enforces dependency rules, runs L0–L1 gates with canaries, refuses unsigned loosening | 3–5 weeks |
| **2. Test strength (L2–L3 CPU)** | Buck2 coverage configuration and diff coverage; Buck2-native mutation driver with dependent sampling, TCE and sealed equivalence records; pseudo-tested checks; hidden holdout runner; flake gate; Cargo lane (Miri, Kani harness policy, fuzz replay, cargo-deny/vet); gungraun perf gate; metric registry with countermetrics; DST kit and determinism meta-test; trajectory policy checker | Full L2–L3 for CPU code on free runners | 5–8 weeks |
| **3. GPU, numerics and infrastructure** | Hermetic toolchains (pinned Mesa lavapipe, WARP, SPIRV-Tools, validation layers, LLVM tools); naga-vs-Tint differential harness; work-group permutation and determinism hashes; tolerance-derivation helper; special-value sweeps; GPU mutation harness; pull-model GPU agent; cache deployment; fork-PR hardening | L2–L3 for GPU code; GPU evidence from the self-hosted machine | 6–8 weeks |
| **4. Formal and L4–L5** | Escape-hatch ledger; spec freeze; spec-strength checks and negative-output suites; Verus/Kani-contract/TLA+ lanes; C-island profile (canonicalised Clang MC/DC, Frama-C, CompCert); CUDA L4 pipeline (Faial/VOLTA); qualification records; evidence bundle and assurance-argument generator; Alive2 and differential-compilation gates | L4 end to end; L5 for small CPU components | 8–12 weeks, then ongoing |

Ordering rationale: anti-gaming infrastructure comes first because without it every later gate measures how well
agents game it.

---

## 9. Expected impact and cost

### 9.1 Expected impact (with evidence strength)

| Mechanism | Expected effect | Evidence |
|---|---|---|
| Sandbox hardening and sealed oracles | Large drop in exploits | −87.7% with four controls (Thaman 2026, preprint, single benchmark) |
| Hidden held-out tests with fresh inputs | Catches hard-coding and heuristic overfitting | EvilGenie caught all non-heuristic hacks (Gabor et al. 2025); SpecBench: 97% visible vs 0% hidden on one lookup-table "compiler" |
| Mutation-based strengthening instead of coverage | Rejects a meaningful share of plausible-but-wrong patches | 19.78% of passing patches rejected (SWE-ABS, Yu et al. 2026); 70% of 1,502 high-priority bugs had a coupled mutant (Petrović et al. 2021) |
| Survivors as findings, scoped to the diff | Actionable feedback | 89% of filtered mutants judged useful at Google (Petrović et al. 2022; self-reported) |
| Deterministic cheat checking on proofs | Cheating drops but never to zero | 14% → < 1.5% (VeruSAGE) |
| Verifier in the generation loop | Converges where single-pass fails | 15/15 vs 0/30 (FORGE, preprint, small sample) |
| Negative-output spec suites | About a third of accepted specs exposed | VeriScale (Bai et al. 2026) |
| Diverse methods rather than one | Different methods find different bugs | Cedar: proofs 4 bugs, differential/property testing 21 (Disselkoen et al. 2024); seL4: verification 144 defects vs testing 16 (Klein et al. 2009) |

Honest limits: most gaming numbers come from 2025–26 preprints with fast-moving model versions; most adequacy evidence
is Java- or C-based, with no Rust fault-coupling study; GPU evidence is concentrated in a few research groups and on
NVIDIA; no study validates the ladder as a whole.

### 9.2 Cost

| Item | Cost | Needed for |
|---|---|---|
| GitHub-hosted runners (public repo) | free, unlimited standard minutes | all levels |
| Rust/C verification tools (Kani, Verus, Miri, cargo-mutants, proptest, bolero, loom, shuttle, gungraun, naga, SPIRV-Tools, Mesa, Frama-C open-source plug-ins, CBMC) | free / open source | L1–L5 |
| Remote cache | free (NativeLink self-hosted or BuildBuddy Personal); machine cost if self-hosted | all levels, for speed |
| Self-hosted GPU machine | hardware and power already owned; no service cost | L3+ GPU evidence |
| Ferrocene qualified toolchain | €25/user/month or €240/user/year; CI use reported free (Ferrous Systems 2024–26) | L5 (optional L4) |
| CodSpeed (optional) | free for OSS incl. 600 bare-metal min/month; then $0.032/min | optional perf trends |
| GitHub GPU larger runners (alternative to self-hosted) | ~$0.052/min, Team/Enterprise orgs only **[U]** | not planned |
| Antithesis (optional) | undisclosed; OSS programme exists | optional L4–L5 system fault injection |
| Astrée, Cppcheck Premium, CompCert commercial licence | quote only (CompCert free for non-commercial use) | only if C islands need them |
| External assessor | engagement-based | only for real certification, or under D1 option (a) if external |
