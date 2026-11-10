# Sub-topic B + F: GUI oracles that need no human, and deterministic rendering for gating

**Scope:** (B) the GUI test-oracle problem: reference/differential oracles, cross-platform differential, metamorphic relations, invariants, view-hierarchy/a11y-tree oracles, visual regression testing and its flakiness. (F) Determinism of UI rendering for gating: virtual clocks, pinned fonts, headless renderers, golden governance, safety-critical HMI practice.
**Date:** 2026-11-10. **New sources:** 20 academic (PDF + extracted text + summary) + 6 practitioner summaries. **Reused:** ~25 summaries from earlier corpora (Section 8).
**Tags:** each claim is marked with a citation (Author Year, section/table), [practitioner], or [our inference]. "Other agent" marks summaries written by parallel agents in this same folder.

---

## 1. Consensus

### 1.1 Dominant consensus: compare structured UI state, often, against a reference or invariant; pixels come second

Supporting papers:
- Memon 2000 (Sec. 4): oracle = model-derived expected widget-property state, compared after **every** action.
- Memon 2003 (Fig. 5, 7) and Xie 2007 (Tables IV-V, Sec. 4.6):
  - Oracle strength changes fault detection significantly (Friedman and Wilcoxon, all p < 0.05).
  - Widget-only oracles are weakest: about 1,100 vs about 2,000 fault-detecting tests (Memon 2003 Fig. 7).
  - Some faults show only in a short "window of opportunity", so they need per-event checks.
- Mesbah 2012 (Sec. 5, 8): generic DOM and state-machine invariants catch faults without golden output.
  - 8/10 seeded faults detected.
  - 6 industrial failure types found in one week.
- Fazzini 2017 (Sec. III-IV): match nodes in the view hierarchy against a reference device, report structural differences first and per-node visual differences second.
  - 96 cross-platform inconsistencies found, 16 FPs (about 86% precision [our arithmetic]).
- Su 2021, Sun 2021, Wang 2022 and Xiong 2024 all use view-tree/layout comparison as the oracle substrate, not screenshots.
- Iqbal 2019 and Duan 2011: industrial avionics and automotive HMI testing uses model/spec-level expected values per state, with images used only to *extract* values.
- Reused support:
  - lui-conformance/_SUBTOPIC_REPORT.md concluded "geometry is the primary gate; pixels are secondary".
  - Roy Choudhary 2013 (X-PERT) and Song 2022 (R2Z2) localise through DOM/layout stages.
  - Su 2022 dVermin (other agent) pairs elements exactly: 97% precision and recall, vs OwlEyes CNN at 39%/50%.

Bias risks:
1. Most evidence is Android or web. Desktop/embedded HMI evidence is thin: Duan 2011 is preliminary, Iqbal 2019 has a single subject.
2. Seeded faults dominate the controlled experiments (Memon 2003, Xie 2007, Mesbah 2012 TUDU).
3. Tool papers report precision on their own triage and rarely report false-negative rates. DiffDroid's 0-FN check covered only 3,315 sampled nodes.
4. Publication bias favours tools that found bugs.
5. The Ting Su / ECNU group authored 4 of the metamorphic papers (Genie, SetDroid, ODIN, Kea), so their framing ("heuristic oracles are high-FP, properties are precise") is partly self-comparative.

### 1.2 Secondary consensus A: pixel oracles are sensitive but noisy; flakiness is mostly timing and platform, and must be engineered out

- Visual oracles rarely miss changes but report many non-defects:
  - Alégroth 2018 (Sec. V-B): 0 false negatives, but **16% of 17,567 executions were FPs** vs about 3% true failures.
  - Reused: GraphicsFuzz 41.7% FP (Donaldson 2017); R2Z2 18.75% FP after triage (Song 2022).
  - Reused: Zhang 2026 found exact pixel diff has 0% no-change accuracy on their benchmark.
- Root causes of flakiness:
  - Async wait / timing: 45.1% of UI flaky tests (reused Romano 2021, Table IV), including animation timing in 26 of them. Parry 2021 lists asynchronicity and concurrency as the leading causes.
  - Platform dependency: 34% of sampled flaky bug reports (Parry 2021). Dependency (device, OS, library) is 22% in Thorve 2018.
  - Synchronisation and fuzzy-match instability are the top Visual GUI Testing problems (Alégroth 2015, Sec. 4.1-4.2).
- Practitioner determinism toolkit:
  - Virtual clock: Chromium `setVirtualTimePolicy`, LVGL `lv_test_wait`, Flutter `pump`, Roborazzi paused Compose clock [practitioner].
  - Pinned or box fonts: Flutter Ahem default [practitioner].
  - Exact comparison by default: LVGL `REF_IMG_TOLERANCE 0`; Roborazzi verify fails on any difference [practitioner].
  - Tolerances calibrated from a measured noise floor: reused Lu 2026 (d <= 2 noise vs d >= 4 defects); Chromium `determine_gold_inexact_parameters.py`.

Bias risk: the practitioner evidence is documentation, not measured FP/FN.

### 1.3 Secondary consensus B: metamorphic oracles are precise when the relation is exact, and imprecise when heuristic

| Relation | Precision | Source |
|---|---|---|
| Exact round-trip ("change setting + restore = same GUI") | **83.2%** (124/149) | Sun 2021 Rule I |
| Exact pairing across scales | **97%** | Su 2022 dVermin, other agent |
| Detour returns to same state | 39/47 confirmed with calibrated threshold | Lu 2026, reused |
| Rotate twice | 22 bugs vs 9 distinct FPs (about 71%) | Zaeem 2014 |
| Heuristic independence | **40.9%** | Su 2021 Genie |
| Statistical majority | about **33%** ("approximately 2/3 FPs") | Wang 2022 ODIN |
| "Expected difference" (language changes all text) | **19.2%** | Sun 2021 Rule II |

- Explicit properties beat heuristics: Kea found 92/97 historical bugs (94.8%) vs 13.4% for prior heuristic oracles, at 3-5 min per property (Xiong 2024).

### 1.4 Minority / contested position: learned and VLM oracles

- LLMShot (Kaynak 2025): above 80% recall, lower precision; Gemma3 at temperature 0.1.
- Learned rendering monitor for telltales (Bürkle 2024).
- Reused: Zhang 2026 captioning; other agent's Hu 2025 (VLM about 71% detection at 10% FPR).
- DiffDroid's final visual verdict is itself a trained decision tree (Fazzini 2017, Sec. IV).

All of these are **non-deterministic** or training-dependent and suitable only for triage or explanations [our inference].

---

## 2. Oracle-technique catalogue

Determinism: D = deterministic, M = mixed, L = learned.

| # | Technique (exemplar) | Detects | Inputs | Det. | FP / FN evidence | Cost |
|---|---|---|---|---|---|---|
| 1 | **Reference differential on full structured state after each event** (Memon 2003; Xie 2007) | Any property divergence: missing/extra widgets, wrong text, wrong state; transient faults if checked per event | Widget-property tree of reference and SUT (here: scene graph or a11y tree of Rust vs C) | D | Significant detection gains over widget-only (p < 0.05); richer oracle found faults earlier in 33-62% of cases (Xie 2007 Sec. 4.5). No FP data (seeded faults). | Xie: about 1 month per app for 60k runs in 2006; cheap for a serialised scene graph [our inference] |
| 2 | **Cross-platform structural + per-node visual differential** (Fazzini 2017) | Structural (node missing) and visual (node rendered differently) inconsistencies | View hierarchy + screenshot per window, reference vs test | M (tree matching D; visual classifier L) | 96 inconsistencies, 16 FPs (14 trailing-whitespace text bounds, 2 imperceptible colour); 0 FN in 3,315 sampled nodes | CW-SSIM dominates compute; parallel |
| 3 | **Spec/model-derived expected state** (Memon 2000; Iqbal 2019 OCL; Duan 2011 statecharts) | Divergence from specified values per state | Formal model or constraints + extracted state | D (OCR extraction in Iqbal is M) | Iqbal: 18 faults, OCR "not always 100%"; Memon 2000: cost only | Model authoring |
| 4 | **Generic invariants** (Mesbah 2012) | Dead clickables, broken back-navigation, invalid DOM, error text, accessibility | DOM/scene per state, transitions, network traffic | D | 8/10 seeded faults; misses faults that leave the DOM unchanged | < 6 min crawl+test; < 30 min setup |
| 5 | **Explicit properties (precondition, interaction, postcondition)** (Xiong 2024; Quickstrom LTL, other agent) | Any functional property expressible over UI states | GUI state predicates | D oracle (exploration random) | 94.8% historical recall; no FP problem reported | 3-5 min per property |
| 6 | **Metamorphic identity / round-trip** (Zaeem 2014; Sun 2021 Rule I; Lu 2026) | State loss/corruption after change-and-restore | Two runs' layouts/screens | D if time-aligned | 71% (Zaeem; FPs from time-varying state and harness differences); 83.2% (SetDroid I) | 2x execution |
| 7 | **Metamorphic expected difference** (Sun 2021 Rule II) | Setting/action that does not take effect (e.g., language) | Two runs + classifier of expected change | M (langid) | 19.2%; 91% of FPs from legitimately untranslated content | 2x execution |
| 8 | **Independence metamorphic** (Su 2021 Genie) | Interactions that wrongly affect unrelated content | Seed vs mutant GUI-tree deltas | M (heuristic independence) | 40.9% TP | 16 emulators, large mutant sets |
| 9 | **Deviant-behaviour majority differential** (Wang 2022 ODIN) | Outlier responses to the same event from the same abstract state | Many traces, clustered layouts | M (statistical) | About 1/3 TP | 36 h per app |
| 10 | **Whole-frame exact golden** (LVGL test module; Paparazzi/Roborazzi verify) | Any pixel change | Framebuffer vs PNG | D (if renderer pinned) | High sensitivity; FPs from intentional change and baseline drift (16% of executions, Alégroth 2018) | Cheap |
| 11 | **Whole-frame fuzzy/perceptual** (Playwright YIQ 0.2; SSIM, FLIP, pHash: reused) | Pixel changes above threshold | Two images | M | Threshold hides subtle changes; GraphicsFuzz 41.7% FP with histogram metric (reused) | Cheap |
| 12 | **Region hash / CRC per node rect** (automotive telltale practice, Bürkle 2024 Sec. II) | Any change in a region | Framebuffer ROI + expected check value | D | Breaks under alpha blending, scaling, overlays on variable backgrounds | Very cheap (runtime-capable) |
| 13 | **Assistive-technology replay** (reused Salehnamadi 2021/2022/2023) | Elements unreachable or unlabeled via a11y | A11y tree + AT-driven replay | D | Fewer warnings than rule scanners (reused) | Re-run of functional tests |
| 14 | **Run-twice determinism oracle** (Genie replay-twice filter, Su 2021 Sec. 4.3; Roborazzi byte-identical UI JSON [practitioner]) | Hidden nondeterminism (time, randomness, races) | Same trace twice | D | Genie uses it as a noise filter; for a pure reactor any difference is a bug [our inference] | 2x |
| 15 | **Learned triage** (LLMShot; Bürkle; DiffDroid classifier; reused Zhang 2026; other agent Hu 2025) | Classifies or explains diffs | Images (+ text) | L | About 80% recall with lower precision (LLMShot); not adversarially robust | GPU or VLM calls |

---

## 3. Reference-differential vs invariant vs metamorphic: comparison and combination

### 3.1 Evidence-based comparison

- **Reference differential** has the broadest coverage:
  - It catches anything observable that differs, including "missing whole sections" (F11) and "missing version line" (F10). An invariant would have to anticipate these.
  - Its precision depends almost entirely on **normalising legitimate differences**:
    - DiffDroid's remaining FPs were text-bound padding (Fazzini 2017).
    - McKeeman had to suppress floating-point rounding and could not compare unspecified diagnostic text (McKeeman 1998).
    - Mesbah needed "oracle comparators" for volatile DOM (Mesbah 2012, Sec. 8.4).
  - It also inherits **reference bugs**: "most bugs were in the comparison compilers" (McKeeman 1998).
- **Invariants** are app-agnostic, need no reference and are deterministic:
  - Dead clickables, back consistency, DOM validity (Mesbah 2012); occlusion and content-mask tests (dVermin, other agent).
  - But they only catch what they encode, and they miss faults that leave the state unchanged (2/10 in Mesbah 2012).
- **Metamorphic relations** fill the "no reference for this screen" gap:
  - Precision ranges from 97% to 19%, depending on whether the relation is exact (Section 1.3).
  - With a pure reactor and virtual time, most kiosk relations can be made exact: round-trip identity, sibling uniformity, language switch at msgid level [our inference].
- **Explicit properties** give precise oracles at small authoring cost (Xiong 2024). The risk for AI-written code is vacuous properties written by the same agent [our inference].

### 3.2 Recommended combination (layered, all deterministic) [our inference, built on the cited techniques]

| Layer | Check | Basis |
|---|---|---|
| L0 Determinism | Run each trace twice under the virtual clock; scene-graph JSON and framebuffer must be byte-identical | Su 2021 replay-twice; Parry 2021 |
| L1 Reference parity, structure | After every input event **and** at every timer deadline, compare the normalised C scene graph with the Rust scene graph | Xie 2007 per-event L3; Fazzini 2017 structural CPI; Memon 2000 complete-properties |
| L2 Reference parity, properties | Per matched node, compare role, resolved text (msgid + formatted args), style id / emphasis class, selected/checked state, visibility, z-order and relative geometry (ratios, containment, alignment), not absolute pixels | DiffDroid per-node comparison; reused Walsh/ReDeCheck relational layout |
| L3 Generic invariants on the C scene + a11y tree | No dead intents; no interactive node occluded by a non-modal node; text glyph box inside container (no clipping); text not truncated without ellipsis policy; scrim swallows taps only when modal is declared; lifetime of transient nodes >= declared minimum | Mesbah 2012; dVermin; Salehnamadi a11y replay |
| L4 Metamorphic relations | Toggle twice = identity; language switch changes exactly the msgid nodes; digit then delete = identity; sibling keys respond with the same delta kind | Sun 2021 Rule I/II; Zaeem 2014; Wang 2022 adapted |
| L5 Pixel goldens of the C kiosk | Exact (tolerance 0), per-node-region hashes plus a whole frame. Generated by the gate from an L1-L4-passing build, never hand-edited | LVGL [practitioner]; Bürkle 2024 regions; Alégroth 2018 / Fujita 2023 governance |
| L6 Optional | VLM/LLM explanation attached to a deterministic failure; never a pass/fail input | Kaynak 2025; Zhang 2026 |

### 3.3 Handling intentional divergence from the Rust reference

Evidence-backed mechanisms:
1. **Normalise before compare.** Strip fields that legitimately differ by substrate: absolute pixel coordinates (compare ratios and relations), font family, anti-aliasing. Fazzini 2017 excused dynamically sized lists; McKeeman 1998 suppressed rounding; Mesbah 2012 used comparators.
2. **Ignore lists for non-localisable or dynamic content.** Sun 2021 says Rule II FPs "can be easily resolved" by ignoring specified elements. In the kiosk, nodes without msgid, or with role=dynamic, are excluded by construction [our inference].
3. **Divergence allowlist as reviewed data** [our inference]:
   - Each entry has a key (screen, node semantic key, field), a reason class (substrate / intentional product difference / reference bug), a link to the spec or Rust issue, and an expiry.
   - Entries can only be added in a change that does **not** touch the C UI code they excuse (separation of duties for agents), and they count against a ratcheting budget.
   - Rationale: Alégroth 2018 and Fujita 2023 show baseline updates are routine (8.2% of commits) and mostly correct, so an agent's self-approval cannot be told apart from maintenance.
4. **Reference-bug verdict.** A divergence where Rust is wrong is fixed in Rust (or recorded as a temporary entry with a Rust issue), so the oracle improves rather than being silently forked (McKeeman 1998) [our inference].

---

## 4. Determinism recipe for a software-rendered LVGL / XRGB8888 pipeline

[our inference unless cited]

1. **Single injected clock.**
   - All time reads go through one tick source; ban direct `clock_gettime`/`time()` in UI code by lint or link-time wrapping.
   - Tests advance it LVGL-style: `lv_tick_inc` + `lv_timer_handler` per ms, as `lv_test_wait` does [practitioner: LVGL source].
   - Use "advance to next timer deadline" for speed (Chromium "advance" policy [practitioner]), but take a **scene snapshot at each deadline** so transient states are observable (Xie 2007 window of opportunity).
   - Avoid single-jump `lv_test_fast_forward` for lifetime checks: it runs one handler pass [practitioner: LVGL source].
   - Pin an initial virtual epoch so date/countdown text is reproducible (Chromium `initialVirtualTime` [practitioner]).
2. **Starvation guard.** Fail (do not silently advance) if the reactor keeps scheduling 0-delay work. This mirrors Chromium's `maxVirtualTimeTaskStarvationCount` [practitioner]; failing is the gate-appropriate variant.
3. **Single-threaded, ordered input.**
   - Inject events through the same indev path real input uses: Zaeem 2014 found 4 of 9 FPs were harness-vs-real-input differences.
   - Events are timestamped in virtual time.
4. **Pinned rendering substrate.**
   - Hermetic build with exact LVGL version, software renderer only (no GPU or EGL path in the gate), fixed resolution and stride, XRGB8888 (alpha ignored, as in the LVGL compare [practitioner]).
   - Fixed font files embedded by hash, fixed rasteriser settings (no subpixel AA, fixed hinting), seeded RNG.
   - Platform dependency is a leading flake cause (Parry 2021: 34%; Thorve 2018: 22%).
5. **Two-tier font goldens.**
   - Layout/metrics goldens use a box font or font metrics only, so they survive font updates (Flutter Ahem [practitioner]).
   - Pixel goldens use the pinned production font.
6. **Tolerance calibration.**
   - Default 0 (LVGL default [practitioner]).
   - Before allowing any tolerance, measure a noise distribution on known-equivalent renders (run-twice, rebuilt binaries, different hosts) as reused Lu 2026 and Chromium's Gold parameter search do. Set the threshold just above the observed maximum and only ever ratchet it down (reused lui-conformance report Sec. M5).
   - For a pinned software renderer the expected noise floor is exactly 0. Any nonzero noise is a determinism bug to fix, not to tolerate.
   - Never use Playwright-style 0.2 YIQ defaults: subtle selected-state tints (F9) can fall under them [practitioner].
7. **Per-region verification.**
   - Store one hash per scene-node rect plus the full frame (automotive region-CRC practice, Bürkle 2024 Sec. II).
   - This localises failures, the reported weakness of whole-frame diffs (other agent's Mahajan 2014: 77% localisation accuracy).
   - Bürkle shows region CRCs break under blending over variable content. Keep scrims over static content, or hash pre-composition layers.
8. **Golden governance against AI agents re-blessing.**
   - (a) Build with `LV_TEST_SCREENSHOT_CREATE_REFERENCE_IMAGE 0`. The LVGL template default (1) writes a missing reference and **passes** [practitioner: lv_conf_template.h, lv_test_screenshot_compare.c].
   - (b) No record or verifyAndRecord modes in CI (Roborazzi `verifyAndRecord` [practitioner]).
   - (c) C-kiosk pixel goldens are produced by the gate only after L0-L4 pass against the Rust reference. The primary oracle (Rust scene) is not editable by the C-side agent.
   - (d) Golden or allowlist changes may not be in the same commit as UI code changes; Fujita 2023 shows co-change is the norm.
   - (e) Golden deletions, test skips and tolerance increases are counted and budgeted. Thorve 2018: 13% of flaky fixes were skip/delete.
   - (f) LLM triage (LLMShot) may annotate but never approve.
9. **No retries.** Retry-on-fail (practised in Alégroth 2015) converts real intermittent defects into passes. With L0 in place, any irreproducibility is itself a failure.

---

## 5. Mapping F1-F11 to the best deterministic oracle from this sub-topic

| F | Defect | Best deterministic oracle (this sub-topic) | Backup | Evidence |
|---|---|---|---|---|
| F1 | Rejection dialog removed after about 60 ms (should stay 8 s) | Per-tick reference parity: snapshot the scene at each virtual timer deadline and 1 ms steps around transitions; compare node lifetimes with the Rust trace | Lifetime invariant (transient node visible >= declared duration); property "P: dialog shown at t, Q: present at t+7.9 s" | Xie 2007 window of opportunity; LVGL `lv_test_wait`; Chromium virtual time; Kea properties |
| F2 | No visible feedback while typing a code | Reference differential after each keypress: code-display node text/glyph count must match Rust | Metamorphic "digit then delete = identity" plus "digit changes display" (no-effect check) | Memon 2003; Zaeem 2014; Mesbah 2012 (event must change state) |
| F3 | Banner covers a keypad key; scrim swallows first tap | Invariant: no interactive node occluded by a non-modal node; tap-target hit-test = topmost intent node; scrim allowed only for declared modal | Reference parity of z-order/rect relations; region hash of the key rect | Mesbah-style invariants; dVermin content-mask overlap (other agent); Bürkle regions |
| F4 | Language toggle does nothing | Dead-intent invariant (firing an intent must change reactor state or scene) | SetDroid Rule II at msgid level (all localisable nodes switch catalogue, nothing else changes); Rule I round-trip | Mesbah 2012 Sec. 5.3.1; Sun 2021 |
| F5 | Locker number as two tiny separated glyphs | L2 property parity: emphasis/style class and relative size ratio vs Rust (number height / screen height; single text run) | Per-node region golden | Fazzini 2017 per-node comparison; Iqbal 2019 constraints |
| F6 | Countdown clipped vertically; format differs | Text-fits-container invariant (glyph box within node rect, using pinned font metrics) + formatted-string parity with Rust at the same virtual time | Region golden | Chromium initial virtual time [practitioner]; McKeeman normalisation (compare formatted value, not pixels) |
| F7 | All text one size, left-aligned | L2 parity of style id / font-size class and alignment relations per node vs Rust | Box-font layout golden | Memon 2000 property levels; Flutter Ahem [practitioner] |
| F8 | No wrapping; Bulgarian truncated mid-sentence | Language-switch metamorphic run (bg locale) + text-fits/no-truncation invariant; parity of line count vs Rust | Pixel golden per locale | Sun 2021 (language-change defects); reused Alameer 2016 i18n |
| F9 | Toggle buttons show no selected state | Parity of selected/checked property per node + round-trip relation (toggle twice = identity, once != identity) | Exact region golden (a tint below lax thresholds is still caught at tolerance 0) | Xie 2007 (stateful widgets need state checks); Sun 2021 Rule I; Playwright threshold caveat |
| F10 | Splash missing version line | Structural parity: every Rust node with a semantic key must exist in C (DiffDroid structural CPI) | Static-content check from spec | Fazzini 2017; Duan 2011 (missing static text a major HMI error class) |
| F11 | Staff report missing whole sections | Structural parity of the subtree (section keys and counts) vs Rust, on screens reached by the same event trace | Property "report contains sections S1..Sn" | Fazzini 2017; Memon 2003 (all-window state) |

---

## 6. Hard-to-catch list (what even a good deterministic oracle stack may miss)

1. **Reference and implementation wrong in the same way.** Differential oracles are blind to shared misconceptions, e.g. both apps truncate Bulgarian. Only invariants and spec properties catch these (McKeeman 1998; reused Barr 2015) [our inference].
2. **Transients shorter than the sampling granularity.** A node shown and removed within one timer handler pass is invisible to per-deadline snapshots. It needs event-sourced scene-diff logging, not periodic snapshots (Xie 2007) [our inference].
3. **Faults that leave the observed state unchanged.** Mesbah 2012 missed 2/10 when errors were swallowed. In the kiosk: a scene node that is correct but never painted because of an lvrender mapping bug. Pixel/region goldens are needed as the second layer [our inference].
4. **Over-normalisation.** Every allowlist or normaliser entry is a blind spot, e.g. ignoring absolute geometry hides F5 unless ratios are compared. Precision gains from normalisation (DiffDroid) trade against recall [our inference].
5. **Vacuous properties or invariants written by the same agent** (Xiong 2024's dependence on property quality) [our inference].
6. **Perceptual hierarchy problems that are "parity-correct" but bad**, e.g. Rust also has weak emphasis. These belong to usability metrics (other sub-topic), not oracles.
7. **Blending over variable content.** Region CRCs break and tolerances creep back in (Bürkle 2024).
8. **Font/rasteriser updates.** Legitimate global pixel churn invites mass re-blessing. Two-tier goldens (box-font layout + pinned-font pixels) contain it (Flutter [practitioner]) [our inference].
9. **Hit-testing vs paint divergence.** A scrim invisible in pixels but swallowing taps (F3) is detectable only by an input-routing check, not by visual comparison [our inference].
10. **Order- and history-dependent bugs** reachable only through long event sequences. Oracle strength helps (Xie 2007), but exploration depth limits (reused Quickstrom: about half of 43 implementations failed sequence-dependent checks).

---

## 7. Gaps

- No peer-reviewed study measures **golden-image FP/FN rates for a pinned software renderer** (noise floor of exact goldens). Also noted as missing by the lui-conformance report. Our expectation of a zero noise floor is [our inference].
- No academic study of **golden re-blessing by AI agents** or of governance controls for it. Evidence is indirect (Fujita 2023 churn, Alégroth 2018 FP share, Thorve 2018 skip/delete).
- **Embedded/automotive HMI test-oracle evidence is thin and old.**
  - Duan 2011 is preliminary; Iqbal 2019 has one subject; SCADE/ARINC 661 material is vendor claims [practitioner]; ISO 26262 / DO-178C texts were not obtained.
  - No independent study of image-comparison tolerance in certified HMI testing was found.
- **Divergence-allowlist management** in differential testing has no empirical study. McKeeman 1998 and DiffDroid treat it ad hoc.
- **Virtual-time UI testing** has no academic evaluation found; only tool docs (Chromium, LVGL, Flutter, Roborazzi).
- **LVGL-specific testing practice:** only source code was available (docs URL 404).
- Paywalled/unreached items: VTest (Ran 2022, ICSE-SEIP; host unreachable); Squish image-verification docs (404); ISO 26262 Part 6 / DO-178C text.

---

## 8. Sources

### 8.1 Reused corpus (read, not re-downloaded)

Base: `~/Downloads/research_papers/`

- `lui-conformance/summaries/Barr 2015 - Oracle problem.md`
- `lui-conformance/summaries/Romano 2021 - UI flaky tests.md`: 235 UI flaky tests; async wait 45.1%; animation timing 26.
- `lui-conformance/summaries/Song 2022 - R2Z2.md`: pHash 4,096 bits, Hamming 140; 81.25% TP after WPT oracle.
- `lui-conformance/summaries/Song 2023 - Metamong.md`: 28/28 recall; 0 FP among 21 reports.
- `lui-conformance/summaries/Donaldson 2017 - GraphicsFuzz.md`: 41.7% FP of the histogram image metric.
- `lui-conformance/summaries/Donaldson 2020 - Randomized testing production.md`
- `lui-conformance/summaries/Lu 2026 - Dataviz equivalent mutations.md`: noise d <= 2 vs defects d >= 4; 47 bugs.
- `lui-conformance/summaries/Kulkarni 2026 - Semantics 2D rasterization.md`
- `lui-conformance/summaries/Andersson 2020 - FLIP.md`
- `lui-conformance/summaries/Wang 2004 - SSIM.md`
- `lui-conformance/summaries/Zhang 2018 - LPIPS.md`
- `lui-conformance/summaries/Ding 2020 - DISTS.md`
- `lui-conformance/summaries/Mowery 2012 - Pixel Perfect canvas fingerprinting.md`
- `lui-conformance/summaries/Laor 2022 - DrawnApart.md` (both: rendering differs across hardware)
- `lui-conformance/summaries/Zhang 2026 - Beyond pixel diffs VRT captioning.md`
- `lui-conformance/summaries/Watanabe 2026 - VRT failures discussion.md`
- `lui-conformance/summaries/Roy Choudhary 2013 - X-PERT.md`
- `lui-conformance/summaries/Walsh 2017 - ReDeCheck.md`
- `lui-conformance/summaries/Alameer 2016 - i18n presentation failures.md`
- `lui-conformance/_SUBTOPIC_REPORT.md`: WPT fuzzy reftests; Skia Gold Sobel/fuzzy matching; Chromium inexact-parameter search.
- `event-reactor-testing/summaries/barr2015_oracle_survey.md`
- `event-reactor-testing/summaries/chen2018_metamorphic_review.md`
- `event-reactor-testing/summaries/elbaum2009_carving.md`
- `event-reactor-testing/summaries/choudhary2015_android_tools.md`
- `event-reactor-testing/summaries/wang2018_android_industrial.md`
- `event-reactor-testing/summaries/alshahwan2018_sapienz_fb.md`
- `event-reactor-testing/summaries/tigerbeetle_vopr_docs.md`
- `event-reactor-testing/summaries/antithesis_docs.md`
- `event-reactor-testing/summaries/ocallahan2017_rr.md`
- `event-reactor-testing/summaries/su2017_stoat.md`
- `rigor-levels-test-adequacy/summaries/luo2014-flaky.md`
- `rigor-levels-test-adequacy/summaries/memon2017-google-ct.md`
- `rigor-levels-test-adequacy/summaries/chen2018-metamorphic-review.md`
- `lui-usability-metrics/summaries/Salehnamadi 2021 - Latte assistive-service driven testing.md`
- `lui-usability-metrics/summaries/Salehnamadi 2022 - Groundhog accessibility crawler.md`
- `lui-usability-metrics/summaries/Salehnamadi 2023 - A11y Puppetry AT record and replay.md`
- `lui-usability-metrics/summaries/Moran 2018 - Automated reporting of GUI design violations.md`: GVT 98% precision / 96% recall on injected violations.
- `lui-usability-metrics/summaries/Memon 2007 - Event-flow model of GUIs for testing.md`

Other agents' summaries in this folder (cited, not modified): `visual-gate-2026-11/Articles/`
- `Su 2022 - dVermin scaling issues`
- `Mahajan 2014 - Image comparison presentation failures NIER`
- `Hu 2025 - Minuku display issues`
- `Yandrapally 2020 - Near-duplicate state detection`
- `Baek 2016 - Multi-level GUI comparison criteria`
- `OConnor 2022 - Quickstrom LTL acceptance testing`
- `Mesbah 2012 - Crawljax UI state crawling`

### 8.2 New sources (this agent)

All in `visual-gate-2026-11/Articles/<name>/Summary.md`; PDFs and extracted texts are under the same names.

1. McKeeman, W.M. (1998). Differential Testing for Software. *Digital Technical Journal* 10(1):100-107. No DOI.
2. Memon, A.M., Pollack, M.E., Soffa, M.L. (2000). Automated Test Oracles for GUIs. FSE-8. `10.1145/357474.355050`
3. Memon, A.M., Banerjee, I., Nagarajan, A. (2003). What Test Oracle Should I Use for Effective GUI Testing? ASE 2003. `10.1109/ASE.2003.1240304`
4. Xie, Q., Memon, A.M. (2007). Designing and Comparing Automated Test Oracles for GUI-Based Software Applications. *TOSEM* 16(1). `10.1145/1189748.1189752`
5. Duan, L., Hussmann, H., Niederkorn, D., Höfer, A. (2011). Model-Based Testing for the Menu Behavior of Automotive Infotainment System HMIs. EICS 2011. DOI not verified.
6. Mesbah, A., van Deursen, A., Roest, D. (2012). Invariant-Based Automatic Testing of Modern Web Applications. *IEEE TSE* 38(1):35-53. `10.1109/TSE.2011.28`
7. Zaeem, R.N., Prasad, M.R., Khurshid, S. (2014). Automated Generation of Oracles for Testing User-Interaction Features of Mobile Apps. ICST 2014. `10.1109/ICST.2014.31`
8. Alégroth, E., Feldt, R., Ryrholm, L. (2015). Visual GUI Testing in Practice: Challenges, Problems and Limitations. *EMSE* 20(3). `10.1007/s10664-013-9293-5`
9. Fazzini, M., Orso, A. (2017). Automated Cross-Platform Inconsistency Detection for Mobile Apps. ASE 2017. `10.1109/ASE.2017.8115644`
10. Alégroth, E., Karlsson, A., Radway, A. (2018). Continuous Integration and Visual GUI Testing: Benefits and Drawbacks in Industrial Practice. ICST 2018. `10.1109/ICST.2018.00026`
11. Thorve, S., Sreshtha, C., Meng, N. (2018). An Empirical Study of Flaky Tests in Android Apps. ICSME 2018. `10.1109/ICSME.2018.00062`
12. Iqbal, M.Z., Sartaj, H., Khan, M.U., Ul Haq, F., Qaisar, I. (2019). A Model-based Testing Approach for Cockpit Display Systems of Avionics. MODELS 2019. `10.1109/MODELS.2019.00-14`
13. Su, T. et al. (2021). Fully Automated Functional Fuzzing of Android Apps for Detecting Non-crashing Logic Bugs. *PACMPL* 5(OOPSLA). `10.1145/3485533`
14. Sun, J. et al. (2021). Understanding and Finding System Setting-Related Defects in Android Apps. ISSTA 2021. `10.1145/3460319.3464806`
15. Parry, O., Kapfhammer, G.M., Hilton, M., McMinn, P. (2021). A Survey of Flaky Tests. *TOSEM* 31(1). `10.1145/3476105`
16. Wang, J. et al. (2022). Detecting Non-crashing Functional Bugs in Android Apps via Deep-State Differential Analysis. ESEC/FSE 2022. `10.1145/3540250.3549170`
17. Fujita, S., Kashiwa, Y., Lin, B., Iida, H. (2023). An Empirical Study on the Use of Snapshot Testing. ICSME 2023. `10.1109/ICSME58846.2023.00041`
18. Xiong, Y. et al. (2024). General and Practical Property-based Testing for Android Apps. ASE 2024. `10.1145/3691620.3694986`
19. Bürkle, C., Oboril, F., Scholl, K.-U. (2024). Learning-Based Error Detection System for Advanced Vehicle Instrument Cluster Rendering. arXiv:2409.02647.
20. Kaynak, E.B., Lami, M., Moslemi, S., Koyuncu, A. (2025). LLMShot: Reducing Snapshot Testing Maintenance via LLMs. arXiv:2507.10062.

Practitioner sources:

21. LVGL (2026). Test module: `lv_test_screenshot_compare`, `lv_test_wait`, `lv_test_fast_forward` (source code, github.com/lvgl/lvgl).
22. Paparazzi and Roborazzi (2026). Android JVM screenshot testing docs.
23. Flutter (2026). `matchesGoldenFile`, `pumpAndSettle` API docs.
24. Playwright (2026). `toHaveScreenshot` API docs.
25. Chromium / DevTools (2026). `Emulation.setVirtualTimePolicy` (Emulation.pdl), `--virtual-time-budget`.
26. Ansys/Synopsys (2026). SCADE Display / SCADE Test / ARINC 661 product claims (vendor).

---

## 9. Exhaustion report

**Searches run:**
- research-hub `search_papers`: about 18 queries, including GUI test oracles (FSE 2000, ASE 2003, TOSEM 2007), invariant web testing, QUANTUM, Genie, ODIN, SetDroid, DiffDroid, visual GUI testing, snapshot testing, flaky-test survey, McKeeman, Duan, Iqbal, Bürkle, VTest.
- WebSearch: about 16 queries, covering open copies, instrument-cluster HMI testing, ARINC 661/SCADE, automotive MBT, Kea, VTest, golden flakiness, Chromium virtual time, Squish, ISO 26262 telltale CRC/MISR.
- WebFetch: about 10 pages (Paparazzi, Roborazzi, Flutter, Playwright, Ansys, NAIST, DevTools). LVGL docs, Chromium virtual_time.md and Squish docs returned 404.
- GitHub raw/API: LVGL test sources, DevTools `Emulation.pdl`.

**Downloads:**
- research-hub batch: 6 of 14 succeeded.
- Recovered via author/open copies: Genie (arXiv), Mesbah (UBC), DiffDroid (UMN), Memon 2003 (UMIACS), Parry (White Rose), ODIN (author site), Thorve (VT), Duan (LMU), Kea (author site), Iqbal (ORBilu), LLMShot and Bürkle (arXiv), Fujita (author GitHub), McKeeman (Wayback copy of HP Labs DTJ archive).
- Anna's Archive: session expired (DDoS-Guard 403); skipped per instructions.

**Failures:**
- VTest (Ran 2022): host unreachable and DOI download failed. Dropped; it is an exploration paper, so low relevance to oracles.
- Squish docs: 404. Squish tolerant/threshold modes are cited only from search snippets and are not summarised.
- ISO 26262 / DO-178C / ARINC 661 standard texts: not available. Only vendor claims were used.

**Totals:** 20 new academic PDFs, about 26 MB; 6 practitioner summaries.

**Saturation:**
- GUI oracle classics (Memon line) and Android metamorphic/differential oracles (Su group) are well covered.
- Embedded/automotive HMI oracle evidence is not saturated (few, weak sources).
- No academic study was found for golden governance or virtual-time UI testing.
