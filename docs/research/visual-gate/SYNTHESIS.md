# Deterministic visual/UI gate for the C lockers kiosk: synthesis

**Question:** what does the literature support as a *deterministic, generic* gate that would mechanically have caught the 11 escaped UI defects (F1-F11 in `_BRIEF.md`)? The pipeline is pure reactor -> scene graph -> lvrender/LVGL -> XRGB8888 software renderer -> X11, plus an a11y tree over TCP. The Rust app is the oracle. Agents write all code and nobody reviews it.
**Compiled:** 2026-11-10.
**Corpus:** 99 new source summaries in `Articles/` (82 full-text PDFs, ~136 MB; 17 standards, practitioner docs or abstract-only), plus about 90 reused summaries from `lui-conformance`, `lui-usability-metrics`, `lui-agent-loops`, `lui-gpu-rendering`, `lui-perf-measurement`, `event-reactor-testing`, `event-reactor-formal` and `rigor-levels-test-adequacy`.
**Sub-topic reports** (full evidence, numbers, citation lists, exhaustion reports):
- `_SUBTOPIC_A_E_layout_text.md`
- `_SUBTOPIC_B_F_oracles_determinism.md`
- `_SUBTOPIC_C_exploration_temporal.md`
- `_SUBTOPIC_D_interaction.md`

**Evidence tags:**
- `(Author Year)`: a paper summarised in this corpus.
- `[standard]`: WCAG, WebDriver.
- `[practitioner]`: tool docs, vendor or source code.
- `[our inference]`: a design conclusion not directly tested by any source.

Effect sizes are as reported by the original authors.

**Caveat on evidence base.** Almost all of it is web DOM or Android; nothing is on LVGL or embedded kiosks. Most papers are evaluated by the tool's own authors on seeded or self-labelled faults (bias analysis in each sub-report, §1).

---

## 0. Verdict

1. **These were missing oracles, not missing states.** Every escaped defect occurred in a state the existing tests could reach. What was missing were generic checks over (time, event, scene) traces and over scene geometry. The literature has three such check families with high precision, and none needs a human:
   - **Relational geometry checks** on the UI tree: X-PERT precision went from 18% to 76% when it switched from pixels/absolute positions to relations (Roy Choudhary 2013). GWALI: P 91% / R 100% (Alameer 2016). dVermin: P/R 0.97 against 0.39/0.50 for a CNN (Su 2022). VFDetector: about 97-99% precision (Ryou 2018).
   - **Per-event reference-differential oracles:** Memon 2003; Xie & Memon 2007; DiffDroid at about 86% precision (Fazzini 2017).
   - **Generic HMI/temporal property templates:** Harrison 2019; Degani 2002; Bolton 2013; Konrad 2005; Quickstrom found faults in 20 of 43 implementations (O'Connor 2022).
2. **The pure reactor removes the field's two hardest problems.**
   - Model inference: ripping and crawling with state-similarity guesses has a best universal-threshold F1 of 0.24 (Yandrapally 2020).
   - Flakiness: async waits cause 45.1% of UI flaky tests (Romano 2021).
   - With a pure reactor, bounded exhaustive BFS over (abstract state × event catalogue) under a virtual clock is exact, cheap and replayable. Rushby 2002 found an MD-88 mode surprise this way in 0.24 s. **[our inference, by analogy]:** no paper evaluates exhaustive exploration of Elm/Redux-style reducers (Gap G1).
3. **Compare at every pipeline stage, not just at the end.** R2Z2 localised the faulty stage in 18 of 19 cases by comparing stage by stage (Song 2022).
   - F7 ("style ignored") and probably F5 and F6 can be lvrender or LVGL defects. In those cases the scene graph is correct, so a scene-level oracle passes.
   - The gate therefore needs four observation levels: reactor view state, scene graph, the **post-lvrender LVGL object tree** (resolved font, alignment, coords, label lines, flags) and the framebuffer.
   - The a11y tree should be produced from the LVGL object tree, not from the scene graph, so that it is an independent observation **[our inference]**.
4. **Pixels are the last layer, not the first.**
   - Tolerant image oracles give 16-42% false positives: Alégroth 2018 had 16% of 17,567 CI runs; GraphicsFuzz had 41.7%.
   - Learned screenshot detectors plateau at P/R ≈ 0.8 with 10-20% FPR: OwlEyes, Nighthawk, Textout, Minuku. That is unusable without a human.
   - Exact goldens of the C build on the pinned software renderer are cheap and deterministic. They are worth keeping as a "scene correct but never painted" catch-all, but only under strict golden governance (§6.4).
5. **Some things stay hard to check mechanically:**
   - whether an overlap or clip is intended;
   - perceptual adequacy (is the feedback, selected state or emphasis *noticeable*, not just present);
   - defects shared with the Rust oracle;
   - real input-to-photon latency;
   - specs the implementing agent gets to interpret.

   Each needs declared intent in the scene graph, owner-sealed thresholds, or a separate (non-gate) measurement (§3).

---

## 1. What the gate must observe

The trace record per explored step (sub-report C §3.4, extended) **[our inference]**:

```
(t_virtual, event, view_state_projection,
 scene_graph,                 # role, rect, z/layer, style id, msgid+args, intents, enabled/inert, input policy, declared importance
 lv_tree,                     # post-lvrender: resolved font id/size, align, coords, label long-mode + line count, clickable/hidden flags, opacity
 a11y_tree (from lv_tree),
 hit(p) for probe points,     # via the production LVGL indev/hit-test path
 framebuffer region hashes,   # per node rect + full frame (optional per tier)
 oracle_projection)           # same projection emitted by the Rust app on the same event trace
```

**Normalised projection shared with the Rust oracle.** This is the comparison basis for every differential check (DiffDroid-style node matching (Fazzini 2017); relational layout (Roy Choudhary 2013; Walsh 2017)):
- screen id;
- dialog stack;
- node semantic key;
- role;
- msgid + formatted args;
- emphasis/size class;
- alignment class;
- selected/checked;
- visible/hittable;
- relative geometry: containment, order, alignment, size ratios.

Absolute pixels and fonts differ by substrate and are not compared across apps (B §3.3).

---

## 2. Per failure mode: best-evidenced deterministic detection

Each block gives the primary check, its inputs, its FP/FN behaviour and the evidence. "Inputs" use: **RX** = reactor + event catalogue; **SG** = scene graph; **LV** = post-lvrender LVGL tree; **FB** = framebuffer; **A11Y** = a11y tree; **TR** = timed trace; **REF** = Rust oracle; **FONT** = pinned font metrics.

### F1: rejection dialog removed ~60 ms after creation (must stay 8 s)

- **Primary check (property):** P5 *minimum dwell / bounded invariance*. Once dialog d becomes visible, it stays visible for at least T_min of virtual time unless an allowed dismiss event for d occurs. P8 (no flicker) generalises this to any node: it must stay at least T_perc unless a declared successor replaces it.
  - Sources: the minimum-duration and bounded-invariance patterns (Konrad 2005, 80 industrial real-time requirements); bounded past-MTL monitoring (Ho 2014); the transient-state class in Quickstrom's "problem 14" (O'Connor 2022).
  - T_min comes from the spec (8 s), floored by reading rate: D = 1 s + chars/(r·5.6/60), with r = 175 wpm (130 wpm inclusive), floor 3.5 s (Brysbaert 2019 reading rate [our inference for the formula]). Errors should stay until acknowledged (WCAG 2.2.1 [standard]).
- **Primary check (differential):** compare node lifetimes with the Rust trace. Snapshot at every virtual timer deadline, and in 1-frame steps around every transition.
  - Some faults are visible only in a "window of opportunity", so end-of-sequence oracles miss them (Xie & Memon 2007).
- **Inputs:** RX + TR (virtual clock) + SG (+ REF).
- **Exploration requirement:** time must be an explicit event: `fire_next_deadline`, `advance(1 frame)`, `advance(deadline − ε)` (C §3.1).
  - Avoid LVGL's single-jump `lv_test_fast_forward` for lifetime checks; it runs one timer pass [practitioner: LVGL source].
  - Use `lv_test_wait`-style 1 ms stepping, or deadline stepping with a snapshot at each deadline [practitioner].
- **FP/FN:**
  - FPs come from legitimately superseded messages; declare precedence and successor rules.
  - FN: a node created and destroyed inside one handler pass is invisible to deadline snapshots. Log scene *patches* (event-sourced), not periodic snapshots (B §6.2) [our inference].
  - Monitors must be bounded. Unbounded liveness gives no verdict on finite traces: 43 of 97 standard pattern formulas never conclude (Bauer 2011).
- **Strength:** strong (deterministic by construction; patterns well established). The exact UI dwell threshold is inference: no empirical study of toast or banner duration exists (Gap G6).

### F2: no visible feedback while typing the code

- **Primary checks:**
  1. **Echo correspondence:** each accepted digit adds exactly one echo character (masked or clear) and backspace removes exactly one. The echo must be in the view state returned by the *same reducer step* that consumed the key. Sources: Thimbleby & Cairns 2010 (silently ignored keystrokes cause out-by-10 errors); Oladimeji 2011.
  2. **P1 bounded action feedback:** a state-changing tap yields a visible change in the scene projection within T_fb. T_fb ≤ 85 ms (≤ 5 frames at 60 Hz), target ≤ 2 frames.
     - Kaaresoja 2016: visual button feedback guideline 30-85 ms; quality drops at 100-150 ms.
     - Jota 2013: tap latency imperceptible below about 20-40 ms.
     - Dabrowski & Munson 2011: control band 50-200 ms; about 200 ms echo delay raises typing errors.
     - The 0.1/1/10 s limits are expert judgement (Miller 1968), not experiment (Dabrowski 2011) [practitioner: Nielsen 1993].
  3. **P3 state distinguishability:** two reachable states with different entered-code length render differently (Harrison 2019 state-feedback template; Degani 2002 "error states").
  4. **Differential:** the code-display node's text after each key equals Rust's (Memon 2003 per-event oracle; Fazzini 2017).
- **Inputs:** RX + TR + SG/LV (+ REF).
- **FP/FN:**
  - FP: deliberate no-ops (key pressed when the field is full). Declare these, and require "full" feedback.
  - FN: echo that exists but sits away from the user's gaze. Users watched the keys for 271 s and the display for 26 s (Oladimeji 2011); only a placement rule ("echo adjacent to keypad") addresses this [our inference].
  - Per-key confirmation raised correct entries from 69.6% to 82.7% (Hoggan 2008), which confirms the stakes.
- **Strength:** strong.

### F3: banner overlay covers a keypad key; its tap-outside scrim swallows the first tap

- **Primary checks:**
  1. **Hit-test reachability:** in every reachable state, for every enabled, non-inert interactive node n and every probe point p in P(n), `hit(p) ∈ subtree(n)`. P(n) = centre + 3×3 inset grid + a disc of about 7.5 mm radius clipped to n.
     - About 15 mm diameter is needed to capture 95% of uncompensated touches (Holz 2010).
     - Touches are systematically offset, and edge targets fail 31.7% vs 17.6% (Henze 2011, 120 M taps).
     - Same definition as WebDriver "obscured" and Playwright "receives events" [standard][practitioner].
     - VFDetector checks hit-testability over crawled states: FP 2/237 covered-element and 7/254 inconsistent-visibility reports, and 91% of its findings are *not* found by the geometry checker ReDeCheck (Ryou 2018).
  2. **Overlay input-policy rule:** a layer that captures input over an enabled node fails unless the state declares that node inert (declared modal, visibly dimmed). Default overlays capture all touches without propagating, and 0 of 20 users noticed (Fratantonio 2017).
  3. **Effective target area:** the hittable area remaining after occlusion must still meet the size threshold. A 2×48 px uncovered sliver was unclickable (Ryou 2018). WCAG excludes overlapped area [standard].
  4. **First-tap pass-through (behavioural):** with a non-modal overlay shown, a probe tap outside it must have the same effect as tapping `hit(p)` would without the overlay (Belli 2001 complement/"illegal" event pairs; rage-click pattern [practitioner]).
  5. **Geometry:** a non-modal overlay must not intersect an enabled interactive node that the oracle state leaves uncovered. Use relational diff vs REF (Roy Choudhary 2013), with overlap measured on ink masks rather than boxes (Su 2022).
- **Inputs:** SG/LV z-order + input policy, production `hit()`, RX + probe-tap events, REF.
- **FP/FN:**
  - Overlap rules *without* declared layer semantics are noisy: rule-based checks on Android view dumps scored P 0.32 / R 0.36, largely from undeclared floating layers (Liu 2022). Declared input policy per layer removes this class [our inference].
  - FN 1: the gate's hit-test differs from LVGL's real routing (flags, bubbling, hidden scrims). Call the production path headlessly.
  - FN 2: mid-animation interception. Check every animation frame and report how long the interception lasts.
  - FN 3: alpha-0 capturing layers counted as "visible" (Playwright counts opacity:0 as visible [practitioner]).
  - A scrim that swallows taps is invisible in pixels. **Only an input-routing check finds it** (B §6.9).
- **Strength:** strong.

### F4: tapping the EN language toggle does nothing

- **Primary checks:**
  1. **No dead interactive element (P2):** a tap on an enabled interactive node must change the reactor state or the observable scene, unless the (state class, intent) pair is in an **owner-sealed** idempotent list.
     - Mesbah 2012's generic "no dead clickables" invariant: 8 of 10 seeded faults caught with no golden output.
     - Belli 2001: every (state, event) pair needs a declared response.
     - Dead click = no visual change, no navigation [practitioner: Microsoft Clarity].
  2. **Static complement:** every intent id has a non-identity reducer transition in at least one state [our inference; ATF DuplicateClickableBounds rationale [practitioner]].
  3. **Language-switch metamorphic relation at msgid level:** switching language changes the rendered text of every msgid node whose two catalogue strings differ, and changes nothing else. SetDroid's version of this rule had only 19.2% precision, and 91% of its FPs were legitimately untranslated content (Sun 2021). Knowing the msgids and both catalogues removes that FP source **[our inference]**.
  4. **Reversibility:** EN→BG→EN restores the projection (Harrison 2019 reversibility template). Exact round-trip relations are the precise kind of metamorphic oracle: 83.2% precision for SetDroid Rule I and 97% for dVermin, against 19-41% for heuristic relations (Sun 2021; Su 2022; Su 2021 Genie; Wang 2022 ODIN).
- **Inputs:** RX + SG (+ REF for "wrong effect").
- **FP/FN:**
  - FP: re-selecting the already-selected language. Put it on the sealed list, with pressed-state feedback required.
  - FN: a *wrong* visible effect (EN switches to BG) passes P2; it needs REF parity.
  - FN: Mesbah's invariants miss faults that leave the state unchanged (2 of 10).
  - Anti-gaming: an agent can "fix" a dead button by adding it to the idempotent list, so the list must be owner-sealed (C §6.2).
- **Strength:** strong.

### F5: locker number (primary content) renders as two tiny separated glyphs

- **Primary checks:**
  1. **Structural parity:** the semantic value is one text node with the same msgid/args and node count as REF (Fazzini 2017 per-node matching; Si 2024 block matching, where block recall and position best predict human preference).
  2. **Legibility floor in visual angle at the farthest viewing distance**, computed from pinned font metrics (cap/x-height × px pitch). Below the critical print size of about 0.2° x-height, reading slows sharply (Legge 2011). In glance reading, text size dominates (η² 0.55; 3 mm vs 4 mm text cost +27% time), and spacing cannot compensate (Dobres 2018). Minimum font size is a standard deterministic rule (Mahajan 2018, MFix).
  3. **Declared-importance hierarchy:** each node carries an intended importance level (≤ 3 levels; O'Donovan 2014 models emphasis as the correlation of perceived with desired importance). The primary node must have prominence rank 1, with size ratio ≥ k over the next level [our inference for k].
  4. **Glyph-run continuity:** the inter-glyph ink gap inside one numeric run must be ≤ a fraction of the em [our inference]. This catches "two separated glyphs" even when the node count matches.
- **Inputs:** SG + LV + FONT (+ REF) + display spec (ppi, viewing distance).
- **FP/FN:** the floor and rank checks are deterministic. No validated threshold exists for "prominent enough":
  - pixel aesthetic metrics explain only about 36-50% of rating variance (Miniukovich 2014; Oulasvirta 2018);
  - learned importance models (CC 0.875, Fosco 2020) are design-class-specific and not deterministic.
  - Parity with REF only tells you C matches Rust; if Rust is weak, so is C.
- **Strength:** strong for size and structure; medium for hierarchy (Gap G2).

### F6: countdown clipped vertically; format differs from oracle

- **Primary checks:**
  1. **Text-fits-container from font metrics, no OCR:** for every text node and glyph, the ink box (baseline + glyph ofs_y + box_h, line_count × line_height) lies inside the label content box and every clipping ancestor.
     - LVGL and FreeType expose ascent, descent, line height and per-glyph boxes [practitioner].
     - This replaces the OCR step that limited AccessiText's clip detection to P 73%; its geometry-only rules reached R 100% (Alshayban 2022).
     - No published P/R exists for the metric-based version (Gap G4).
  2. **Format parity:** the formatted string equals REF's at the same virtual time, with a pinned initial epoch (Chromium `initialVirtualTime` [practitioner]). Compare formatted values, not pixels (McKeeman 1998 normalisation).
  3. **Bounded recurrence (P14):** while visible, the countdown changes at least every 1 s (Konrad 2005).
- **Inputs:** LV + FONT + TR (+ REF).
- **FP/FN:**
  - FP: intended clipping (marquee/scroll); declare it.
  - FN: glyph shapes that exceed their nominal boxes (diacritics, Cyrillic descenders). Use per-glyph boxes, not line-level ascent.
- **Strength:** strong by construction; empirically unvalidated.

### F7: every text node uses one font size, left-aligned (style ignored)

- **Primary checks:**
  1. **Stage conformance (lvrender):** for every node, the resolved LVGL font, size and alignment equal the style table entry for its style id. This is a stage-wise comparison in R2Z2's sense (Song 2022) **[our inference for the LV level]**. A scene-level oracle cannot see this defect if the scene is right and the mapper drops the style.
  2. **Relational parity vs REF:** centring, alignment and containment relations per node (X-PERT/GWALI relational graphs; InferUI "preserve centering" robustness properties (Bielik 2018)), with ±1 px tolerance. Off-by-one is InferUI's most common violation, and 14% of XFix leftovers were sub-pixel or few-pixel (Mahajan 2017).
  3. **Sanity invariant:** screens with mixed roles show ≥ 2 distinct size levels (O'Donovan 2014: up to 3 levels) [our inference].
- **Inputs:** SG + LV (+ REF).
- **Strength:** strong (relation diff); the levels rule is heuristic.

### F8: no wrapping; Bulgarian sentence truncated mid-sentence

- **Primary check:** a **metamorphic multi-locale render** of every reachable screen in EN, BG and a pseudo-locale. Requirements:
  - the full string is displayed (no clip or dot loss);
  - all ink stays inside the container (F6 check);
  - no new overlaps or protrusions;
  - the EN layout relations are preserved;
  - no font shrink below the style spec.

  Evidence:
  - GWALI uses the base language as the oracle for translated layouts: P 91% / R 100% (Alameer 2016).
  - AccessiText (2× text scale, which matches WCAG 1.4.4): P 88% / R 96% (Alshayban 2022).
  - dVermin: 60% of buggy enlarged screens were silently cropped, which a single frame does not show (Su 2022).
  - Pseudolocalisation: about +40% expansion with delimiters (Microsoft); IBM's table gives 2-3× for strings up to 10 characters and 1.3× for strings over 70 [practitioner].
  - Shrinking text to fit is an anti-pattern: IFix shrank fonts in 21 of 23 subjects, and constraint repair that kept size was rated 19% more readable (Alameer 2019).
- **Exploration requirement:** language is an abstraction-key dimension and an ordinary event. Explorers have a blind spot for system-language changes (Su 2021 Themis).
- **Static check:** sentence-role labels must use wrap long-mode [our inference].
- **Inputs:** SG + LV + FONT under ≥ 3 catalogues (+ REF line count).
- **FP/FN:**
  - FP: intended ellipsis (declare it).
  - FN: if Rust also truncates, parity passes. Hence the *invariant* (full string shown) must hold independently of REF (B §6.1).
  - No P/R exists for pseudolocalisation, and there is no Bulgarian expansion data (Gap G3).
- **Strength:** strong.

### F9: toggle buttons show no selected state

- **Primary checks:**
  1. **Mode visibility as an "if and only if" (P4):** `G(selected(sigma) ↔ selected_visual(S))`, taken from the syringe-pump status-line formula (Bolton 2013) and the visibility template (Harrison 2019).
  2. **No invisible state change:** if the reactor state changes but the scene or pixels do not, fail (Degani 2002 "error/blocking states"; check (c) in D §2).
  3. **Distinctness:** the selected style differs from the unselected style and from siblings, not by colour alone, with a ≥ 3:1 indicator contrast change (WCAG 1.4.1 / 1.4.11 [standard]).
  4. **Exactly one selected** in single-select groups; the a11y tree's selected state equals the visual state.
  5. **Pixel check:** render selection on item i vs item j; the region diff inside each item must be non-empty. Run it at tolerance 0, because a faint tint can fall under Playwright-style 0.2 YIQ defaults [practitioner].
- **Inputs:** RX + SG/LV + A11Y + FB (+ REF).
- **FP/FN:**
  - FN: indicators that are present but not salient. Visual mode feedback alone explained only 4-5% of mode-error variance, against 11-16% for user-held modes (Sellen 1992). There is no evidence for a minimum salient change (Gap G5).
- **Strength:** strong for presence; weak for salience.

### F10: splash missing the version line; F11: staff report missing whole sections

- **Primary checks:**
  1. **Structural content parity vs REF on the same event path:** the multiset of (role, msgid, section key) per screen and subtree must have recall 1.0 against Rust.
     - DiffDroid reports missing nodes first, then compares per-node visuals: 96 inconsistencies at about 86% precision, and 14 of its 16 FPs were trailing whitespace in text bounds (Fazzini 2017).
     - Design2Code-style block recall (Si 2024).
     - Missing static text is a major HMI error class (Duan 2011).
  2. **Screen inventory and reachability (P11):** every screen in the oracle's inventory is reachable in C (Harrison 2019 completeness; GUI ripping of both apps and diffing the inventories (Memon 2003)).
  3. **Required-content invariants (P12):** for example, the splash contains the version node, and the report contains sections S1..Sn, independently of REF (O'Connor 2022 "elements exist" invariants).
  4. No empty, NULL or raw-msgid text; NULL-value text is one of OwlEyes' defect classes (Liu 2020).
- **Inputs:** RX + SG + REF (+ data fixtures).
- **FN:**
  - Sections that appear only with specific data. **Data coverage** matters: the report must be generated with fixtures that populate every section. Specific inputs are a known blind spot (Su 2021) [our inference].
  - Over-normalisation and allowlists hide divergences (B §6.4).
- **Strength:** strong.

### Summary table

| F | Best deterministic technique(s) | Inputs | Evidence strength |
|---|---|---|---|
| F1 | Minimum-dwell / bounded-invariance monitor under virtual time + lifetime differential vs REF at every deadline | RX, TR, SG, REF | strong (patterns); threshold inferred |
| F2 | Echo correspondence + bounded feedback (≤ 85 ms / same reducer step) + per-key REF parity | RX, TR, SG/LV, REF | strong |
| F3 | Production hit-test at probe points in all states + overlay input-policy + first-tap pass-through + effective target size | LV, hit(), RX, REF | strong |
| F4 | No-dead-intent invariant (sealed idempotent list) + msgid-level language relation + round-trip | RX, SG, REF | strong |
| F5 | One-node parity + visual-angle floor + declared-importance rank/ratio + glyph-run continuity | SG, LV, FONT, REF | strong (size); medium (hierarchy) |
| F6 | Font-metric ink-box containment + format parity at pinned virtual time + 1 s cadence | LV, FONT, TR, REF | strong by construction, unvalidated |
| F7 | lvrender stage conformance (style id → resolved font/align) + relational parity ±1 px | SG, LV, REF | strong |
| F8 | EN/BG/pseudo-locale metamorphic render + full-string + containment + relation preservation + no shrink | SG, LV, FONT, REF | strong |
| F9 | Mode-visibility "if and only if" + no-invisible-state-change + distinctness ≥ 3:1 + a11y agreement + exact region diff | RX, SG/LV, A11Y, FB, REF | strong (presence) / weak (salience) |
| F10/F11 | Structural content parity (recall 1.0) + screen inventory + required-content invariants + data fixtures | RX, SG, REF | strong |

---

## 3. What the literature says is hard to catch mechanically

1. **Intent: which overlaps, clips and ellipses are allowed.**
   - Non-observable issues outnumbered true collisions in ReDeCheck (24 non-observable vs 8 true collisions; Walsh 2017).
   - dVermin's errors were scrollable or ellipsized views (Su 2022).
   - Floating layers drove the rule-based FPs in Nighthawk (Liu 2022).
   - Remedy: intent declared in the scene graph (input policy, clip/ellipsis policy, modal flag), or comparison with REF [our inference].
2. **Perceptual adequacy.** "Present" does not mean "noticed" (feedback, selected state, emphasis):
   - Sellen 1992: visual mode indicators are weak.
   - Oladimeji 2011: echo placement vs gaze.
   - Miniukovich 2014 / Oulasvirta 2018: aesthetic metrics explain ≤ 50% of variance.
   - No validated thresholds exist (Gaps G2, G5).
3. **Defects shared with the reference.** Differential oracles are blind to them. McKeeman 1998 found that most discrepancies were bugs in the comparison compilers, and Barr 2015 covers this in the oracle survey. Only independent invariants catch them (full string shown, target size, dwell).
4. **Sub-pixel and few-pixel differences, and cascades.**
   - 14% of XFix leftovers were sub-pixel or few-pixel (Mahajan 2017).
   - Root-cause localisation from pixels: 77-93% (Mahajan 2014/2015), and about 45% top-5 (Mahajan 2016).
   - Relations need about ±1 px tolerance, and fault attribution should go by element relations, not pixels.
5. **Real latency.** Virtual time cannot see renderer stalls, X11 latency or touch-controller latency; commercial devices measured 50-200 ms (Ng 2012). This needs hardware measurement outside the merge gate (Casiez 2015; Schmid 2018).
6. **Gate/runtime divergence.** A re-implemented hit-test or layout in the gate drifts from LVGL. Always call the production code paths headlessly (D §5.1) [our inference].
7. **States outside the explored set.** Bugs deeper than the bound, or that need rare data or environment replies:
   - Themis: 18 of 52 real bugs were found by no tool, and many appeared in only 1 of 5 runs (Su 2021).
   - Bugs hidden by the abstraction key (Baek 2016; Dong 2020 ignores text-box contents, which would hide F2).
8. **Ambiguous specs and vacuous properties.** When the same agent writes both the code and its properties or allowlists, they collude (Xiong 2024 depends on property quality; Fujita 2023: 8.2% of commits rewrite snapshots, usually in the same commit as the code). Remedy is governance (§6.4), not technique.
9. **Semantic correctness of visible text** (right node, wrong message or wrong translation): only msgid+args parity with REF catches it.
10. **Learned detectors' FP floor** of 10-20% FPR, or 83% for a high-recall VLM (Wang 2019; Hu 2025). This is incompatible with zero human review; allowed only as non-gating explanation (Kaynak 2025 LLMShot).

---

## 4. State exploration strategy

Grounding: C §3 (Rushby 2002; Memon 2001; Strecker 2008; Belli 2001; Dong 2020; Choi 2017; Yandrapally 2020; Baek 2016; O'Connor 2022; Bauer 2011; Konrad 2005). Concrete numbers are **[our inference]** unless cited.

1. **Event catalogue (owner-sealed, versioned).**
   - User events: `tap(intent)` for every intent in the current scene, plus **probe taps through the production hit-test**:
     - each interactive node's centre, inset grid and disc;
     - a coarse whole-screen grid;
     - the expected outcome of each probe, computed from the scene;
     - taps that should do nothing, from Belli's complement pairs (Belli 2001).
   - **Macro events with boundary classes** to keep depth low: code entry empty / 1 / max−1 / max / max+1 / valid / invalid / expired. Per-digit events start only from boundary states.
   - **Environment replies:** ok, reject(reason), timeout, error; door sensor.
   - **Language** as an ordinary event.
   - **Virtual-time events:** `fire_next_deadline`, `advance(1 frame)`, `advance(deadline − ε)`.
2. **Abstraction key** = an owner-declared projection of view state, never learned and never a similarity threshold (Yandrapally 2020: universal threshold F1 0.24). Fields:
   - screen;
   - dialog stack;
   - language;
   - every toggle group's selection;
   - code length class;
   - staff mode;
   - timer presence buckets.

   Rule: **every variable read by any property or oracle projection is in the key.** Validate the key by mutation: for each excluded field, seed a bug that depends only on it.
3. **Search.**
   - BFS from reset, which gives shortest counterexamples and a compact regression corpus (Choi 2017's DetReduce: 16.9× smaller suites, with every exception still caught).
   - Run the C reactor and the Rust oracle in **lockstep** and compare projections at every step (Rushby 2002 pattern with REF as the expected model).
   - Depth bound D = 12 macro-level events. Report new abstract states per depth; if the last depth still adds states, raise D or justify not doing so.
4. **Coverage gates.** These are cheap on an exact model; on real GUIs, pair coverage cost 309-1,204 machine-hours (Nguyen 2014).
   - 100% of reachable (abstract state, catalogue event) pairs, probes included. This is Prowell's arc coverage and Memon's event-pair coverage on an exact model.
   - Every dialog × every terminator (Memon 2001, invocation-termination coverage).
   - Every oracle screen reached.
   - Nightly: event triples ending in a time or dialog event. Each +10 points of event-triple coverage multiplied fault-detection odds by 2.14 (Strecker 2008; one small subject).
5. **Beyond the bound (nightly):**
   - swarm or usage-weighted random walks (Groce 2012: +42% distinct crash causes);
   - deeper macro sequences.

   Random Monkey is a strong baseline on devices (Choudhary 2015; Themis: Monkey 22 bugs vs Ape 24), but on a pure reactor the systematic search is nearly free, so random is the supplement [our inference].

**Cost estimate [our inference, measure first]:** about 10²-10⁴ abstract states × 30-80 events gives 10⁴-10⁶ reactor transitions. At 0.1-1 ms per step (reactor + scene build + properties), that is seconds to minutes on one core. Rendering (lvrender + software raster + LV-tree extraction) is needed only once per *new abstract state × locale*. At roughly 5-20 ms per 1080p software frame, 10³-10⁴ states × 3 locales is on the order of 1-10 core-minutes, and it parallelises by frontier partition.

---

## 5. Oracle strategy: reference-differential vs invariants vs metamorphic

**Evidence-based comparison (B §3.1):**
- **Reference differential** has the broadest coverage. It is the only family that catches "missing whole sections" or "missing version line" without anticipating them (F10, F11). Its precision depends on normalising legitimate differences (Fazzini 2017; McKeeman 1998; Mesbah 2012 comparators), and it inherits the reference's bugs. Oracle *strength* matters: checking all widget properties after every event beats widget-only oracles significantly (Memon 2003; Xie 2007, p < 0.05).
- **Invariants and explicit properties** are app-agnostic and independent of REF. They catch shared defects (F8 if Rust also truncated, F3, F1) but only what they encode.
  - Explicit properties: Kea found 92 of 97 historical bugs, vs 13.4% for heuristic oracles, at 3-5 min per property (Xiong 2024).
  - Generic invariants: Mesbah 2012, 8 of 10.
  - HMI templates: Harrison 2019; Bolton 2013.
- **Metamorphic relations** fill screens or conditions where REF has no counterpart or is not trusted. They are precise only when exact:
  - round-trip identity 83.2% (Sun 2021 Rule I);
  - pairing across scales 97% (Su 2022);
  - heuristic relations 19-41% (Sun 2021 Rule II; Su 2021; Wang 2022).

  A pure reactor + virtual time + msgids makes most kiosk relations exact [our inference].

**Recommended layering** (all deterministic; B §3.2, C §4, D §2). Any failure blocks.

| Layer | Check | Catches |
|---|---|---|
| L0 Determinism | Every trace run twice under the virtual clock; view state, scene JSON, LV tree and framebuffer byte-identical; no retries (Su 2021 replay-twice; Parry 2021; Thorve 2018: 13% of flaky "fixes" were skip/delete) | hidden nondeterminism |
| L1 Lockstep REF parity, structure | Per event and per timer deadline: screen inventory, node set by semantic key, msgid+args, dialog stack (Xie 2007; Fazzini 2017) | F1, F2, F4 (wrong effect), F10, F11 |
| L2 REF parity, properties | Per matched node: role, emphasis/size class, alignment class, selected, visible/hittable, relative geometry ±1 px (Roy Choudhary 2013; Walsh 2017) | F5, F6 (format), F7, F9 |
| L3 Stage conformance | Scene → LV tree: resolved font/size/align/flags/rect match the style spec and scene rect; a11y tree from LV tree equals the scene projection (Song 2022 stage-wise) [our inference for LVGL] | F7, F5, mapping bugs |
| L4 REF-independent invariants and properties | P1-P15 temporal/HMI templates (C §4); hit-test reachability, overlay policy, target size, dead intent (D §2); text-fits from font metrics; full string displayed; visual-angle floors; contrast | F1, F2, F3, F4, F6, F8, F9 (plus shared-with-REF defects) |
| L5 Metamorphic | EN/BG/pseudo-locale renders; toggle round-trip; digit+backspace identity; language changes exactly the msgid nodes; sibling keys give the same kind of change | F8, F4, F9, F2 |
| L6 Pixel layer (C build only) | Exact (tolerance 0) per-node-region hashes + full frame on the pinned software renderer, generated by the gate from an L0-L5-passing build | "correct scene, never painted"; regressions |
| L7 Non-gating | LLM/VLM explanation attached to a deterministic failure (Kaynak 2025: about 80% recall, lower precision) | triage only |

**Handling intentional divergence from Rust (B §3.3):**
1. Normalise substrate differences: compare ratios and relations, not absolute pixels or fonts.
2. Exclude by construction nodes without a msgid or marked dynamic.
3. Keep a divergence allowlist as reviewed data. Each entry has a key, a reason class (substrate / intentional / reference bug), a link and an expiry. Entries are never added in a change that touches the C UI code they excuse, and they count against a ratcheting budget.
4. When Rust is wrong, fix Rust (McKeeman 1998) [our inference for the governance].

---

## 6. Keeping it deterministic and cheap on every merge

### 6.1 Determinism recipe (B §4) [our inference built on cited practice]

- **One injected clock.**
  - Lint or link-wrap out direct time reads in UI code.
  - Step LVGL ticks in tests the `lv_tick_inc` + `lv_timer_handler` way; snapshot at every deadline.
  - Pin the initial epoch.
  - Fail on starvation (0-delay work that keeps rescheduling) rather than force-advancing (Chromium `setVirtualTimePolicy` [practitioner]).
- **Real input path.** Inject events through the LVGL indev path. 4 of QUANTUM's 9 FPs came from the harness behaving differently from real input (Zaeem 2014).
- **Pinned substrate.** Hermetic LVGL version, software renderer only, fixed resolution/stride, fonts embedded by hash, fixed AA/hinting, seeded RNG. Platform dependency is 22-34% of flakiness (Thorve 2018; Parry 2021); identical stacks give identical pixels (Mowery 2012).
- **Tolerance 0.**
  - LVGL's `REF_IMG_TOLERANCE` default is 0 [practitioner].
  - Any tolerance requires a measured noise distribution on known-equivalent renders (Lu 2026: noise d ≤ 2 vs defects d ≥ 4). It is set just above the observed maximum and only ever ratchets down.
  - For a pinned software renderer the expected floor is exactly 0, so noise is a determinism bug.
  - Never use SSIM, LPIPS or DISTS as a gate. DISTS is designed to forgive a 5% shift (Ding 2020).
- **Two-tier goldens.** Layout/metric goldens use a box font so they survive font updates (Flutter Ahem [practitioner]); pixel goldens use the pinned production font.

### 6.2 Tiering and cost

**Per merge (target < 10 min, parallel):**
- L0-L5 over the full bounded BFS (D = 12, 100% state×event pairs including probes).
- Rendering and LV-tree extraction once per new abstract state × {EN, BG, pseudo}.
- L6 region hashes for every abstract state.
- **Diff-scoping** is optional when the full run exceeds budget: changed reactor or scene modules → re-explore from affected screens. The full run nightly is the backstop [our inference].

**Nightly:**
- event triples;
- swarm walks beyond D;
- 2× text scale (WCAG 1.4.4 [standard]);
- mutation of the gate itself: seeded reactor, scene and lvrender mutants that each property must kill, so vacuous properties are found (C §4 notes; B §6.5);
- key-validation mutants.

**Release / hardware (not a merge gate):**
- real input-to-photon latency on the kiosk (Casiez 2015; Schmid 2018);
- touch offset at mounting height;
- these cover the virtual-time blind spot.

### 6.3 Evidence-based thresholds (owner-sealed constants)

| Quantity | Gate value | Evidence |
|---|---|---|
| Keypad key | ≥ 20 mm square | plateau 20 = 25 mm, standing kiosk (Colle 2004); 19.05 mm for low-dexterity older adults (Jin 2007) |
| Other kiosk buttons/toggles | ≥ 19 mm recommended; ≥ 16.5 mm floor | Jin 2007 |
| Any touch target | ≥ 14 mm and ≥ 44 CSS px / 48 dp | Leitao 2012 (97% accuracy); WCAG 24 px has "no specific data" [standard]; Apple/Android [practitioner] |
| Hit-area overlap between intents | none | WCAG overlap exclusion [standard]; ATF [practitioner] |
| Edge-to-edge spacing | ≥ 1 mm hard; ≥ 3 mm recommended | 1 vs 3 mm made no difference (Colle 2004; Sesto 2012); 0-10.5 mm made no difference (Leitao 2012); preference 6.35 mm (Jin 2007) |
| Hit-test probe region | centre + 3×3 inset + 7.5 mm disc | Holz 2010; Henze 2011 [exact grid our inference] |
| Visual feedback / echo | ≤ 85 ms (≤ 5 frames @ 60 Hz), target ≤ 2; logically the same reducer step | Kaaresoja 2016; Jota 2013; Deber 2015 (JND 69 ms); Dabrowski 2011 |
| Busy indicator | by 1 s; progress + cancel beyond 10 s | Nielsen 1993 [practitioner] (originally expert judgement, Miller 1968) |
| Transient message dwell | max(spec, 1 s + chars/(r·5.6/60), 3.5 s), r = 175 wpm (130 wpm inclusive); errors until acknowledged | Brysbaert 2019 (mean 238 wpm, 175-300 range); WCAG 2.2.1 [standard]; platform toast defaults 1.5-3.5 s [practitioner] |
| Text legibility | x-height ≥ 0.2° at max viewing distance (body); primary content several times larger [our inference] | Legge 2011; Dobres 2018 |
| Text / UI-state contrast | 4.5:1 text, 3:1 large text and state indicators; not colour-only | WCAG 1.4.3 / 1.4.11 / 1.4.1 [standard]; APCA unvalidated [practitioner] |
| Text expansion for pseudo-locale | about +40% overall; 2-3× for ≤ 10 chars; 1.3× for > 70 chars | Microsoft; W3C/IBM table [practitioner] |
| Relational geometry tolerance vs REF | ±1 px | Bielik 2018; Mahajan 2017 [our inference for value] |
| Pixel tolerance (C build) | 0 | LVGL default [practitioner]; Lu 2026 calibration method |

px = mm × ppi / 25.4. For example, 20 mm is about 111 px on a 15.6″ 1080p panel.

### 6.4 Governance against agents gaming the gate (B §4.8, C §6, D §5)

**Who owns what:**
- **Sealed by the owner, not editable by implementing agents:** event catalogue, abstraction key, property instances (as named Dwyer/Konrad pattern instances, not raw LTL), thresholds, idempotent-intent list, divergence allowlist.
- **The spec linter rejects unbounded liveness** (Bauer 2011).

**Goldens:**
- Build the gate with `LV_TEST_SCREENSHOT_CREATE_REFERENCE_IMAGE 0`. The LVGL template default (1) writes a missing reference and **passes**, so deleting a PNG hides any visual failure [practitioner: LVGL source].
- No record or verify-and-record modes in CI.
- C goldens are generated by the gate only after L0-L5 pass. Golden and allowlist edits never go in the same commit as UI code; Fujita 2023 shows co-change is normal practice, so self-approval is invisible otherwise.
- Count deletions, skips and tolerance increases against a budget (Thorve 2018).

**Retries and LLM judges:**
- No retries; irreproducibility is itself a failure.
- An LLM may explain a failure, never approve one.

---

## 7. Evidence gaps (searched; nothing found)

- **G1** No peer-reviewed study of exhaustive or bounded exploration of Elm/Redux-style pure UI reducers. Only practitioner material (Imandra, TLA+ models of Redux); the argument here is by analogy to explicit-state HMI checking (Rushby 2002).
- **G2** No validated deterministic hierarchy/emphasis gate or "prominent enough" threshold.
- **G3** No precision/recall for pseudolocalisation, and no Bulgarian/Cyrillic expansion data.
- **G4** No paper detects clipping or truncation from renderer font metrics; the closest use OCR (AccessiText) or connected components (dVermin), and ellipsis detection appears only in patents.
- **G5** No evidence for minimum salient change (selected state, feedback) on touch kiosks.
- **G6** No empirical study of minimum display time for transient UI messages, and no Bulgarian reading-rate data.
- **G7** No standing-kiosk target-size study with older or disabled users.
- **G8** No measured FP/FN of exact goldens on a pinned software renderer, and no academic evaluation of virtual-time UI testing (only tool docs).
- **G9** No study of golden re-blessing or allowlist gaming by AI agents (indirect evidence only: Fujita 2023; Alégroth 2018; Thorve 2018).
- **G10** No LVGL, embedded or kiosk layout-testing research. Embedded HMI oracle evidence is thin: Duan 2011 is preliminary, Iqbal 2019 has one subject, SCADE/ARINC 661 material is vendor claims, and the ISO 26262 / DO-178C texts were not obtained.
- **G11** No data on how often agent-written UI code produces these defect classes.

**Not obtained** (abstract or secondary only; listed in the sub-reports):
- Masci 2014 (FASE);
- Navarre 2009 (ICO);
- Campos & Harrison 2008;
- Miller 1968;
- Shneiderman 1984;
- Kaaresoja 2014 TAP (covered by the 2016 thesis);
- VTest (Ran 2022);
- Bisson 2020;
- Janus 2025 (abstract, from the earlier corpus).

**Anna's Archive** was unusable in this run: the session had expired (DDoS-Guard 403), and `annas login` is human-only, so it was not attempted. All papers came from research-hub, arXiv and author or open copies.
