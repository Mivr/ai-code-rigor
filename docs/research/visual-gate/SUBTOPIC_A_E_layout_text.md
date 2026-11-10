# Sub-topic A + E: layout/presentation failure detection and text-rendering correctness (no OCR)

**For:** deterministic, generic visual/UI gate for the C lockers kiosk (scene graph -> lvrender/LVGL -> software renderer; a11y tree over TCP; Rust app as oracle).
**Failure modes mainly served:** F3 (occlusion part), F5, F6, F7, F8, F9 (visual side), F10, F11 (content parity).
**Compiled:** 2026-11-10. **New sources summarised:** 19 papers + 3 practitioner/standard notes (22). **Reused corpus summaries:** 26.
**Tags:** every claim carries a citation `[Author Year]`, `[practitioner]`, `[standard]` or `[our inference]`.

---

## 1. Consensus

### 1.1 Dominant consensus

**Layout and text defects are detected most precisely by comparing *relations between elements' geometry* (containment, direction, alignment, overlap, visibility/hit-testability), taken from the UI tree. The comparison is either against a reference rendering, or for self-consistency under a controlled transformation (viewport width, text scale, locale, RTL). Pixels are used only to confirm visibility or to localise. Pixel-only and learned single-image detectors are less precise and miss defects that do not "look broken" in one frame.**

Supporting evidence:
- **Relational diff against a reference.**
  - [Roy Choudhary 2013, X-PERT]: precision rose from 18% to 76% by moving from absolute positions/pixels to relative layout.
  - [Alameer 2016, GWALI]: P 91% / R 100% for i18n failures. On the same task WebSee scored P 55% and Fighting-Layout-Bugs P 73% / R 55%.
  - [Alameer 2019, CBRepair]: relations from the baseline language used as linear constraints.
  - [Mahajan 2018, MFix]: layout-distortion metric = symmetric difference of relation labels.
  - [Moran 2018, GVT]: P 98% / R 96% on injected design violations.
  - [Bielik 2018, InferUI]: robustness properties.
- **Self-consistency across a transformation.**
  - [Walsh 2015]: 91% of CSS mutants detected, no false positives.
  - [Walsh 2017, ReDeCheck]: 0 FP for collision and protrusion (43 FP only in the small-range class), but many non-observable issues (NOIs).
  - [Walsh 2020]: "subtle" regressions caught 0.89 vs 0.44 at fixed widths.
  - [Su 2022, dVermin]: page-level P/R 0.97/0.97 vs OwlEyes 0.39/0.50 and DiffDroid 0.30/0.11.
  - [Alshayban 2022, AccessiText]: overall P 88% / R 96%; the geometry-only rules have R 100%.
  - [Yadav 2026]: 11/11 RTL bugs, 0 false alarms; mirrored-pixel diff cannot separate buggy from correct.
- **Hit-testability over states.** [Ryou 2018, VFDetector]: ~97-99% precision. Its defects are 91% disjoint from ReDeCheck's.
- **Pixels used only to confirm.** [Althomali 2019, VISER]: opacity-toggle diff inside the DOM-derived area agrees with manual labels in 87.2% of cases. [Su 2022]: alpha masks refine bounding-box overlap.
- **Declarative properties over geometry and style cover real bugs.**
  - [Hallé 2015, Cornipickle]: >90 real bugs in 35 sites were all expressible as properties.
  - [Panchekha 2018, VizAssert]: 2.6% FP, 2.2% timeouts on formal verification of real pages.

**Bias risks of the dominant consensus:**
- Most evaluations are by the tool authors, on seeded or self-labelled faults:
  - [Mahajan 2014/2015] seed faults themselves.
  - [Walsh 2017] labelled by the first author.
  - [Su 2022] labelled by its own annotators, and reused the pilot apps for evaluation.
  - [Alshayban 2022] verified its own reports.
- Three groups dominate:
  - USC (Halfond): WebSee, FieryEye, XFix, IFix, MFix, GWALI, CBRepair.
  - Sheffield (McMinn/Kapfhammer): ReDeCheck, VISER.
  - ISCAS/Monash (Wang/Chen): OwlEyes, Nighthawk, dVermin.
  In practice they share subjects (the 23 GWALI pages; the Alexa/ReDeCheck pages) and baselines.
- The web DOM is heavily over-represented; there is almost no embedded or LVGL evidence (Gap 5.5).
- Precision is usually reported on flagged items; recall is often bounded by the authors' own inspection.
- Learned baselines are sometimes run out of their training domain, e.g., OwlEyes on scaling issues in [Su 2022]. This inflates the deterministic tool's margin.

### 1.2 Secondary consensus

1. **Text-expansion/scaling defects need a metamorphic two-render oracle.** They are often invisible in one frame: a missing view, or silently cropped content (60% of dVermin's buggy screens are "content cropping") [Su 2022, Sec. 3.2]. [Alshayban 2022] showed the same at 2x text, which matches WCAG 1.4.4 [standard]. Industry pseudolocalisation is the same idea: +40% expansion and delimiters [practitioner: Microsoft 2022; Android; W3C/IBM table].
2. **Learned screenshot detectors plateau around P/R 0.8 with 10-20% FPR.** That is unsuitable for an unattended gate.
   - [Liu 2020, OwlEyes]: P 0.85 / R 0.84.
   - [Liu 2022, Nighthawk]: 0.83/0.82 on real data.
   - [Wang 2019, Textout]: AUC 0.956 but FPR 0.196 at TPR 0.947.
   - [Hu 2025, Minuku]: fine-tuned VLM P 0.91 / R 0.71 / FPR 0.10. Un-finetuned GPT-4.1 and Claude-3.7 reach R 0.40/0.39 at FPR 0.14/0.11; Gemini-2.5 reaches R 0.96 at FPR 0.83.
3. **Legibility thresholds belong in visual angle, not pixels or points.** The critical print size is about 0.2 deg x-height, and the fluent range is 0.2-2 deg [Legge 2011]. In glance reading, size dominates (eta^2 = 0.55, 3 mm vs 4 mm costs +27% time), and extra spacing cannot compensate for small size [Dobres 2018]. Minimum font size is a standard mobile-friendly rule [Mahajan 2018, MFix]. Contrast thresholds are 4.5:1 for text, 3:1 for large text and UI state indicators [standard: WCAG 2.2].
4. **Visual hierarchy is "perceived importance vs intended importance".** This is computable once intent is declared [O'Donovan 2014: Pearson correlation of perceived vs desired importance; at most 3 levels]. Learned importance models are good (CC 0.875 on Imp1k) but class-specific [Fosco 2020; Bylinskii 2017; Jiang 2023 UEyes]. Bottom-up size/colour matters less than location conventions on mobile [Leiva 2020].
5. **Fixing layout by shrinking text is a known anti-pattern.**
   - IFix shrank fonts in 21/23 subjects; without shrinking it resolved only 29% of IPFs [Alameer 2019].
   - Users rated relation-preserving repairs more readable: +19% vs IFix (p = 1.4e-7) [Alameer 2019].
   - Enlarging fonts introduced new overflow [Mahajan 2018].
   - Single-objective repair damages layout [Le-Cong 2021].

### 1.3 Disagreements / tensions
- **Exact vs tolerant pixel comparison.**
  - Against real mock-ups, exact comparison flagged >84% of a page, and a perceptual tolerance was needed (Delta 10%, F 27, L 20, C 0) [Mahajan 2015, WebSee].
  - Inside one browser, exact image inequality was sufficient for confirming visibility [Althomali 2019].
  - Resolution: tolerance is needed only across rendering stacks [our inference]. The kiosk C build and the Rust oracle probably render through different stacks, so compare them at node/relation level; within the C build (EN vs BG, before vs after an event), exact pixels are usable.
- **Heuristic rules vs learned detectors.**
  - [Hu 2025] and [Liu 2022] report that rule-based/static analysis does poorly: P 0.32 / R 0.36 on view-hierarchy JSON.
  - The failures they list are missing font metrics, undeclared floating layers, and spurious nulls in the dump [Liu 2022, Sec. 2.3]. These are properties of foreign, incomplete trees, not of geometry checks [our inference].
  - Deterministic tools with complete trees report 0.88-0.99 precision [Su 2022; Ryou 2018; Alameer 2016].
- **Intended overlaps and clips.** Every geometric checker struggles with intent:
  - ReDeCheck's NOIs (overlap invisible due to padding/overflow:hidden) [Walsh 2017].
  - dVermin's scrollable/ellipsized FPs and FNs [Su 2022].
  - Floating toolbars and dialogs [Liu 2022].
  - AccessiText ignores overlaps that already exist at 1x [Alshayban 2022].
  - Consensus answer: compare against a reference, or declare intent [our inference].

---

## 2. Technique catalogue

Inputs: **SG** = scene-graph geometry/style, **FB** = framebuffer, **A11Y** = accessibility tree, **TR** = event trace / state graph, **REF** = reference app/oracle rendering, **FONT** = font metrics.

| # | Technique | Detects | Input | Determinism | Reported numbers (citation) |
|---|---|---|---|---|---|
| T1 | Relational layout graph diff vs reference (direction/alignment/containment/intersection edges; text runs as nodes; matched nodes) | misalignment, overlap, wrap, overflow, moved/missing elements, i18n failures | SG + REF (or same app in other locale) | deterministic | X-PERT P 76% (from 18%) [Roy Choudhary 2013]; GWALI P 91% / R 100%, median 3 elements to inspect, 9.75 s/page [Alameer 2016]; GVT P 98% / R 96% synthetic [Moran 2018] |
| T2 | Self-consistency over a transformation range (viewport sweep + binary search at transitions) | collision, element/viewport protrusion, small-range, wrapping | SG across N renders | deterministic | 91% mutants, 0 FP [Walsh 2015]; 197 TP / 43 FP / many NOIs [Walsh 2017]; 0.89 vs 0.44 on subtle regressions [Walsh 2020] |
| T3 | Metamorphic text-scale / locale / pseudo-locale two-render diff | missing view, new overlap, crop, truncation, unresponsive text | SG + FB at 1x and kx (or EN vs BG/pseudo) | deterministic (mixed when OCR is used) | AccessiText P 88.3% / R 95.8%; overlap P 89% R 100%; cropped P 73% (OCR) [Alshayban 2022]; dVermin page P/R 0.97, view P 0.84 / R 0.91, 20/21 field issues confirmed [Su 2022]; pseudo +40%, IBM table 130-300% [practitioner] |
| T4 | Hit-testability / covered-element check over a crawled state graph, comparing states reached by the same event sequence | interactive element fully covered (incl. translucent overlay or scrim), off-screen, inconsistent visibility | SG (z-order) + TR (+ REF or second config) | deterministic | FP 7/254 inconsistent faults, 2/237 covered errors; 91% of findings disjoint from ReDeCheck [Ryou 2018] |
| T5 | Targeted visibility confirmation by opacity toggling inside an area of concern | whether a geometric overlap/protrusion is actually visible (filters NOIs) | SG + FB | deterministic | 87.2% agreement with manual labels; 0.8 s per failure [Althomali 2019] |
| T6 | Content-mask (alpha) overlap and crop, template-match visibility | overlap/crop of actual ink, not boxes | per-node FB layers + SG | deterministic | part of dVermin above [Su 2022] |
| T7 | Text-fits-container from font metrics (line count x line_height; per-glyph ink box vs content/clip rect) | vertical clip (F6), horizontal overflow, wrap overflow | SG + FONT | deterministic | no published P/R (replaces OCR step whose errors gave AccessiText's 73% precision) [practitioner: FreeType, LVGL; our inference] |
| T8 | Truncation by string comparison (rendered/laid-out text vs source) or end delimiters | ellipsis/clip truncation, hard-coded strings, concatenation | SG text + catalogue | deterministic | AccessiText truncated P 100% / R 85% via OCR [Alshayban 2022]; delimiter method [practitioner: Microsoft, Android] |
| T9 | Declarative FO+LTL properties over element geometry/style per state; formal layout verification | any expressible invariant: overlap, overflow, misalignment, tone-on-tone, dead pop-up buttons, moving elements | SG + TR | deterministic | >90 real bugs all expressible, no P/R [Hallé 2015]; VizAssert 2.6% FP, 2.2% timeouts [Panchekha 2018]; Troika 13-1469x faster [Panchekha 2019] |
| T10 | Reference-image pixel diff + element localisation (R-tree, clustering, displacement test, symptom ranking) | any visible regression vs reference | FB + REF frame + SG boxes | deterministic | detection 100% seeded; localisation 77% exact-diff [Mahajan 2014], 93% PID [Mahajan 2015]; root cause top-5 ~45% [Mahajan 2016] |
| T11 | Screenshot diff + classifier for perceptibility | cross-browser visual differences | FB pair | mixed | R 0.98 / P 0.66 raw; F 0.923 with classifier [Saar 2015] |
| T12 | Learned single-screenshot detectors (CNN, Faster R-CNN, VLM) | occlusion, text overlap, missing image, NULL text, blank, truncation | FB only | learned | OwlEyes P 0.85 / R 0.84 [Liu 2020]; Nighthawk P 0.83 / R 0.82, AP 0.59 [Liu 2022]; Textout AUC 0.956, FPR 0.196 at TPR 0.947 [Wang 2019]; Minuku P 0.91 / R 0.71 / FPR 0.10; frontier VLMs R ~0.4 or FPR 0.83 [Hu 2025] |
| T13 | Minimum legible size in visual angle (x-height/cap-height, viewing distance, pixel pitch) | text too small, key content not dominant | SG + FONT + display spec | deterministic | CPS 0.2 deg x-height [Legge 2011]; size effect eta^2 0.55, +27% time for 3 vs 4 mm [Dobres 2018]; font-size rule [Mahajan 2018] |
| T14 | Contrast ratio (WCAG) for text and for state indicators | low-contrast text, tone-on-tone, invisible selected state | SG colours + composited FB under glyphs | deterministic | 4.5:1 / 3:1 / 7:1; 1.4.11 3:1 for states [standard]; tone-on-tone bugs observed [Hallé 2015]; low contrast among top Android a11y issues [Chen 2021] |
| T15 | Hierarchy check: intended importance (declared) vs perceived prominence (size/contrast/isolation proxy); learned importance maps as advisory | flat hierarchy, wrong emphasis, primary not dominant | SG + FONT (+ FB for learned) | deterministic proxy / learned model | emphasis = Pearson(perceived, desired), at most 3 levels [O'Donovan 2014]; UMSI CC 0.875 [Fosco 2020]; AIM metrics ~R^2 0.4-0.5 vs ratings [Oulasvirta 2018]; [Miniukovich 2014] 36-40% variance |
| T16 | Guideline lint (design-system "don'ts", a11y rules) | target size, truncation rules, overlapping clickables, missing labels | SG/A11Y (+FB) | deterministic | UIS-Hunter P 0.81 / R 0.90; 12.3% of 60,756 UIs violate [Yang 2021]; ~89% of apps have rule violations [Chen 2021] |
| T17 | Layout robustness invariants / constraint hierarchies | order, standard margins, centring, aspect, inside-screen, pixel-perfect | SG (two configs or REF) | deterministic | generalisation 12.6% -> 86.5-92.3% with properties; real off-screen/overlap bugs found [Bielik 2018]; required/strong/weak hierarchy, 0.14 ms re-solve [Badros 2001] |
| T18 | Mirror metamorphic oracle (LTR vs RTL geometry within 2 px) | BiDi inset/margin bugs | SG x2 | deterministic | 11/11 detected, 0 false alarms; pixel mirror diff cannot separate buggy from correct (0.2% vs 0.6% overlap) [Yadav 2026] |
| T19 | Element-level block match vs reference (text blocks matched, then area recall, position, colour) | missing/extra content, misplacement | SG/FB + REF | deterministic | Block-Match 12-93 range vs CLIP 78-90; block recall and position are the strongest predictors of human preference [Si 2024] |

Notes:
- T7 and T8 are the kiosk-specific upgrades of T3: the renderer is known, so the OCR steps (AccessiText) and connected-component ink-area proxies (dVermin) are unnecessary [our inference].
- T4 must treat input-capturing transparent nodes (scrims) as covering. Ryou's covered errors include translucent overlays [Ryou 2018, AdF.ly and Alexa examples].

---

## 3. F-mode mapping (best deterministic technique(s) from this sub-topic)

| F | Defect | Best deterministic technique(s) | Concrete kiosk check [our inference unless cited] | Evidence strength |
|---|---|---|---|---|
| F1 | dialog removed after ~60 ms | out of main scope; T9 temporal operators [Hallé 2015]; state-graph comparability [Ryou 2018] | `next time dialog shown -> present for >= 8 s of virtual time` on the reactor trace; compare state sequence to the Rust oracle trace | weak here (see trace/LTL sub-topics) |
| F2 | no visible feedback while typing | T1/T19 per-state content parity vs oracle; T10 frame diff after each event; T9 "next time keypress then display text changes" | after each keypad event in a trace, the code-display node's rendered text must equal the oracle's (masked or digits as oracle) and differ from the previous state | medium (pattern from [Hallé 2015] "moving/malfunctioning", [Mahajan 2015]) |
| F3 | banner covers a key; scrim swallows the tap | **T4 hit-testability** [Ryou 2018] + T6 content-mask overlap [Su 2022] + T1 new-intersection rule; declared layer intent (lesson of [Liu 2022] static-analysis FPs) | for every state: each interactive node's top-most-hittable area (z-order incl. transparent input-capturing scrims) >= min target (WCAG 2.5.8: 24 CSS px equivalent [standard]); overlays may intersect interactive base nodes only if declared modal; no intersection that the oracle state lacks | strong |
| F4 | language toggle does nothing | T4 (if covered); else behavioural (other sub-topics); T9 "next time click(toggle) then locale/state changes" [Hallé 2015 "malfunctioning buttons"] | click each interactive node in each state; require an observable state or scene change matching the oracle | medium (behavioural part elsewhere) |
| F5 | locker number as two tiny separated glyphs | **T13** visual-angle floor [Legge 2011; Dobres 2018] + **T15** intended-vs-perceived importance [O'Donovan 2014] + T1 structural parity (one node vs two) | primary-role node: rendered cap height >= designed angle at the farthest viewing distance; its prominence rank = 1 on screen with a ratio >= k over the next level; node count/text equals the oracle ("123" as one text node, not split); clear-space isolation [Dobres 2018] | strong for size; medium for hierarchy (no validated threshold) |
| F6 | countdown clipped vertically; format differs | **T7** font-metric ink box inside content/clip rect [practitioner FreeType/LVGL]; T3 cropped rule [Alshayban 2022; Su 2022]; T8/T19 text equality vs oracle | for every text node and every glyph: ink bbox (baseline + ofs_y + box_h) inside the label content box and every clipping ancestor; rendered string == oracle string (format) | strong (deterministic by construction); P/R of the metric version unpublished |
| F7 | every text node one size, left-aligned | T15 hierarchy-level count; T1/T17 alignment/centring relations vs oracle [Roy Choudhary 2013; Alameer 2016; Bielik 2018 "preserve centering"]; style-id fidelity | screens with mixed roles must show >= 2 distinct size levels; per node the resolved font size/alignment equals the style id's spec; centring/alignment relations equal the oracle's; flag near-misses (1-3 px) [O'Donovan 2014] | strong for relation diff; medium for "levels" rule |
| F8 | Bulgarian sentence truncated, no wrap | **T3 metamorphic locale/pseudo-locale** [Alshayban 2022; Su 2022; practitioner pseudoloc] + **T7/T8** + T1 relation preservation [Alameer 2016/2019] | render every screen in EN, BG and a pseudo-locale (IBM-table expansion, delimiters, Cyrillic padding); require full string displayed (no DOT/CLIP loss), all ink inside the container, no new overlaps or protrusions, all EN relations preserved, no font shrink below the style spec [Alameer 2019] | strong |
| F9 | toggles show no selected state | T14 state contrast (WCAG 1.4.11 >= 3:1 for state indication [standard]); T10/T1 per-state style diff vs oracle; T9 temporal | for each toggle: rendering(selected) vs rendering(unselected) must differ in a visual property, with >= 3:1 contrast change or an added indicator; style matches oracle per state | medium-strong |
| F10 | splash missing version line | T1/T19 content parity vs oracle (set of text nodes/msgids per screen); T10 reference-frame diff; OwlEyes/Minuku "blank/missing" classes as taxonomy only | per screen: multiset of (role, msgid) equals the oracle's; no empty/NULL/raw-msgid text [Liu 2020 NULL class] | strong |
| F11 | staff report missing sections | T1/T19 structural/content parity (node matching then recall) [Si 2024; Moran 2018; Alameer 2016 matching]; T3 "missing view" rule [Alshayban 2022] | section-level tree diff vs oracle: every oracle section/node matched (text + role), recall = 1.0; report unmatched as missing | strong |

---

## 4. What the literature says is hard to catch mechanically

1. **Intent: which overlaps, clips and ellipses are allowed.**
   - Non-observable issues (overlap hidden by padding or overflow:hidden) are a large share of ReDeCheck reports: 24 NOI vs 8 TP collisions [Walsh 2017].
   - dVermin's errors are all about intent: runtime-scrollable views and ellipsized-but-cropped text [Su 2022, Fig. 8].
   - Floating toolbars and dialogs break naive overlap rules [Liu 2022].
   - AccessiText suppresses overlaps already present at 1x [Alshayban 2022].
   - Mitigation: declare intent in the scene graph, or compare against the oracle [our inference].
2. **Sub-pixel and few-pixel differences that humans do not perceive.** 14% of XBIs remaining after repair were of this kind [Mahajan 2017, XFix]. Off-by-one pixel violations were the most common robustness violation [Bielik 2018]. 2 px cases were "subjective" in [Althomali 2019]. A relational check needs a small tolerance [our inference].
3. **Cascades / root-cause localisation.** One faulty element moves its neighbours, and deleted elements cannot be blamed by pixel-to-element mapping: localisation is 77% in [Mahajan 2014], 93% in [Mahajan 2015], top-5 root cause ~45% in [Mahajan 2016].
4. **Aesthetic adequacy and hierarchy quality.** Pixel metrics explain ~36-50% of variance in ratings [Miniukovich 2014; Oulasvirta 2018]. Importance is design-class specific [Fosco 2020]. Bottom-up conspicuity predicts mobile attention weakly [Leiva 2020]. Segmentation choices change metric values [Bourguet 2018]. No validated threshold exists for "primary content is prominent enough" (Gap 5.2).
5. **Correctness of content meaning.** Translation quality, wrong-but-plausible text, and correct number formats need a content oracle; no geometric technique catches them. Only the oracle comparison does (T1/T19) [our inference]. Learned single-image detectors cannot detect missing content that leaves no visual artefact [Su 2022, Sec. 6.1].
6. **Non-determinism in the system under test.** It is the only source of VFDetector FPs (rotating ads, live banners) [Ryou 2018]. In a deterministic reactor this disappears, but only if clocks and animations are virtualised [our inference].
7. **Partial occlusion usability.** "Partially covered" is not "usable": a 2x48 px uncovered sliver could not be clicked [Ryou 2018]. A minimum hittable-area threshold must be chosen [our inference; WCAG 2.5.8 gives 24 CSS px [standard]].
8. **Non-linear layout features.** CSS floats could not be modelled as linear constraints [Alameer 2019]. Disjunctive layouts (wrap vs not) are outside Cassowary's linear model [Badros 2001].
9. **Perceptual contrast beyond the WCAG ratio.** The ratio ignores weight and size beyond the large-text cut-off. APCA addresses this but lacks independent validation [practitioner].
10. **Learned detectors' false-positive cost.** FPR is 10-20% even after fine-tuning [Wang 2019; Hu 2025], or 0.83 for a high-recall VLM [Hu 2025]. This is incompatible with zero human review [our inference].

---

## 5. Gaps (searched, nothing suitable found)

5.1 **No peer-reviewed controlled study comparing WCAG 2 contrast ratio vs APCA** for readability or legibility. Only practitioner posts, the APCA author's own claims, and a small Zagreb survey (abstract only, not obtained) turned up [web search 2026-11-10].
5.2 **No deterministic, validated "visual hierarchy / emphasis" gate.** Existing work is learned importance prediction [Fosco 2020; Bylinskii 2017], energy terms tuned for synthesis rather than detection [O'Donovan 2014], or practitioner heuristics ("1.5x size ratio between levels", source unvalidated) [practitioner web search]. A threshold for "primary content must dominate" would be our own design choice.
5.3 **No empirical precision/recall for pseudolocalisation** as a defect detector, and no Bulgarian/Cyrillic-specific expansion statistics. Only general IBM/W3C and Microsoft rules of thumb exist [practitioner].
5.4 **No paper uses renderer font metrics** (ascent/descent/line height/glyph bbox) instead of OCR to detect clipped or truncated text. AccessiText (OCR) and dVermin (connected components) are the closest. Ellipsis/truncation detection by size comparison appears only in patents (US 10185705, US 8755612, US 9529489; not reviewed in full) [web search].
5.5 **No layout-testing research on embedded/LVGL-style GUIs or kiosks.** All evidence is web DOM or Android view trees [our search]. There is no kiosk-specific legibility study for numerals read at a glance from standing distance. Dobres (automotive glance, words) and Legge (continuous reading) are the nearest.
5.6 **No study measuring how often LLM/agent-written UI code produces these defects.** The visual-defect taxonomies come from human-written apps [Liu 2020; Su 2022; Hallé 2015; Watanabe 2026].
5.7 **Nothing found validating a minimum hittable-area threshold for partially occluded targets.** WCAG 2.5.8 is a standard, not an occlusion study.

---

## 6. Sources

### 6.1 Reused local corpus (cited, not re-downloaded)
`~/Downloads/research_papers/lui-conformance/summaries/`:
- `Alameer 2016 - i18n presentation failures.md`
- `Althomali 2019 - Visual verification layout failures.md`
- `Walsh 2015 - Layout faults responsive.md`
- `Walsh 2017 - ReDeCheck.md`
- `Walsh 2020 - Responsive regressions.md`
- `Roy Choudhary 2013 - X-PERT.md`
- `Mahajan 2017 - XFix search-based repair.md`
- `Saar 2015 - Browserbite.md`
- `Panchekha 2018 - VizAssert.md`
- `Panchekha 2019 - Troika modular layout.md`
- `Yadav 2026 - BiDi structural verification.md`
- `Watanabe 2026 - VRT failures discussion.md`
- `Si 2024 - Design2Code.md`
- `Hottelier 2015 - Synthesis layout engines.md`

`~/Downloads/research_papers/lui-agent-loops/summaries/`:
- `Mahajan 2018 - IFix.md`
- `Le-Cong 2021 - Usability and Aesthetics repair.md`

`~/Downloads/research_papers/lui-usability-metrics/summaries/`:
- `Moran 2018 - Automated reporting of GUI design violations.md`
- `Yang 2021 - Visual design smells vs Material guidelines.md`
- `Oulasvirta 2018 - Aalto Interface Metrics AIM.md`
- `Bylinskii 2017 - Learning visual importance for designs.md`
- `Leiva 2020 - Visual saliency in mobile UIs.md`
- `Gupta 2018 - Saliency prediction for mobile UIs.md`
- `Jiang 2023 - UEyes saliency across UI types.md`
- `Miniukovich 2014 - Visual impressions of mobile app interfaces.md`
- `Chen 2021 - Accessible or not Android app accessibility.md`
- `Bourguet 2018 - Metrics-based GUI aesthetics segmentation problem.md`

Other agents' folders in this base dir, cited by name only: `Fazzini 2017 - DiffDroid cross-platform`, `Mesbah 2012 - Invariant-based web testing`, `OConnor 2022 - Quickstrom LTL acceptance testing`.

### 6.2 New sources (this sub-topic; Summary.md in `Articles/<name>/`)

| # | Source | DOI / URL | Full text |
|---|---|---|---|
| 1 | Alshayban & Malek 2022, AccessiText, ESEC/FSE | 10.1145/3540250.3549118 | Yes (UCI author copy) |
| 2 | Su et al. 2022, dVermin "The Metamorphosis", ASE | 10.1145/3551349.3556935 (arXiv 2212.04388) | Yes |
| 3 | Liu et al. 2020, OwlEyes, ASE | 10.1145/3324884.3416547 | Yes |
| 4 | Liu et al. 2022, Nighthawk, IEEE TSE 49(1) | 10.1109/TSE.2022.3150876 | Yes |
| 5 | Mahajan & Halfond 2014, image comparison (NIER), ASE | 10.1145/2642937.2642966 | Yes |
| 6 | Mahajan & Halfond 2015, WebSee, ICST | 10.1109/ICST.2015.7102586 | Yes |
| 7 | Mahajan et al. 2016, visual symptoms (FieryEye), ICST | 10.1109/ICST.2016.35 | Yes |
| 8 | Mahajan et al. 2018, MFix, ICSE | 10.1145/3180155.3180262 | Yes |
| 9 | Alameer, Chiou & Halfond 2019, CBRepair, ICST | 10.1109/ICST.2019.00026 | Yes |
| 10 | Ryou & Ryu 2018, VFDetector, ICST | 10.1109/ICST.2018.00027 | Yes |
| 11 | Wang et al. 2019, Textout, ISSRE | 10.1109/ISSRE.2019.00032 | Yes |
| 12 | Hu et al. 2025, Minuku, ASE Industry Showcase | 10.1109/ASE63991.2025.00270 (from web search; verify) | Yes (mirror) |
| 13 | Bielik, Fischer & Vechev 2018, InferUI, PACMPL OOPSLA | 10.1145/3276526 | Yes (SRI author copy) |
| 14 | Badros, Borning & Stuckey 2001, Cassowary, ACM TOCHI 8(4) | 10.1145/504704.504705 | Yes |
| 15 | Hallé et al. 2015, Cornipickle, ICST | 10.1109/ICST.2015.7102635 | Yes |
| 16 | Legge & Bigelow 2011, print size, J Vision 11(5):8 | 10.1167/11.5.8 | Yes |
| 17 | Dobres et al. 2018, glance legibility crowding, Applied Ergonomics 70 | 10.1016/j.apergo.2018.03.007 | Yes |
| 18 | O'Donovan, Agarwala & Hertzmann 2014, layouts, IEEE TVCG 20(8) | 10.1109/TVCG.2014.48 | Yes |
| 19 | Fosco et al. 2020, UMSI importance, UIST | 10.1145/3379337.3415825 | Yes |
| P1 | W3C WAI 2023, WCAG 2.2 (SC 1.4.3/1.4.4/1.4.6/1.4.10/1.4.11/1.4.12/2.5.8, luminance) [standard] | https://www.w3.org/TR/WCAG22/ | web |
| P2 | Microsoft 2022 Pseudolocalization; Android pseudolocales; W3C "Text size in translation" (IBM table) [practitioner] | URLs in summary | web |
| P3 | FreeType glyph metrics; LVGL font/label docs [practitioner] | URLs in summary | web (LVGL font page 404; used API/label pages via search) |

Folder names: `Alshayban 2022 - AccessiText`, `Su 2022 - dVermin scaling issues`, `Liu 2020 - OwlEyes`, `Liu 2022 - Nighthawk`, `Mahajan 2014 - Image comparison presentation failures NIER`, `Mahajan 2015 - WebSee`, `Mahajan 2016 - Visual symptoms debugging`, `Mahajan 2018 - MFix mobile friendly`, `Alameer 2019 - i18n repair layout constraints`, `Ryou 2018 - Visibility faults HTML5`, `Wang 2019 - Textout`, `Hu 2025 - Minuku display issues`, `Bielik 2018 - InferUI relational layout synthesis`, `Badros 2001 - Cassowary`, `Halle 2015 - Testing web apps through layout constraints`, `Legge 2011 - Print size reading`, `Dobres 2018 - Crowding text size glance legibility`, `ODonovan 2014 - Learning layouts graphic designs`, `Fosco 2020 - Visual importance across design types`, `W3C-WAI 2023 - WCAG 2.2 text resize reflow spacing`, `Practitioner 2022 - Pseudolocalisation and text expansion`, `FreeType-LVGL - Font metrics clipping check`.

PDFs: 19 files, ~57 MB, in `PDFs/` with matching names. Extracted texts are in `Extracted Texts/`.

---

## 7. Exhaustion report

**research-hub `search_papers` queries (13):**
1. WebSee ICST 2015 title
2. MFix title
3. OwlEyes title
4. AccessiText title
5. Ryou visibility faults title
6. visual symptoms debugging title
7. Alameer 2019 title
8. Legge print size title
9. Cassowary title
10. "internationalization localization bugs empirical study mobile apps text truncation" (found Textout)
11. InferUI title
12. "font scaling text size accessibility Android apps layout issues detection"
13. "visual hierarchy typography emphasis computational metric graphic design importance" (books only)

Further queries:
- "color contrast ratio WCAG validation readability perceptual contrast APCA" (nothing academic)
- "GUI display issues empirical study Android text overlap missing image" (noise)
- Dobres glance legibility
- "Predicting visual importance across graphic design types"
- Cornipickle

**WebSearch queries (9):**
1. AccessiText pdf
2. WCAG vs APCA empirical
3. text truncation detection mobile (found dVermin, Minuku, patents)
4. pseudo-localization expansion ratios
5. dVermin arXiv
6. InferUI pdf
7. Minuku
8. O'Donovan layout pdf
9. computational visual hierarchy model (practitioner blogs only)

Plus 1 LVGL metrics search.

**WebFetch:**
- W3C text-size article
- Android pseudolocales
- Microsoft pseudolocalization
- WCAG 2.2 quickref (truncated at 100k chars)
- WCAG 2.2 TR glossary at offset 170k
- WCAG Understanding 2.5.8
- FreeType glyph metrics
- LVGL font page (failed: 301 redirect, then 404)

**Downloads:**
- research-hub batch: 10/11, then 1/2, 2/2, 2/3, 1/3.
- Direct curl from author copies: AccessiText (UCI), dVermin (arXiv), InferUI (ETH SRI; the ETH research-collection link returned HTML), Minuku (mirror).

**Failures and substitutions:**
- **Anna's Archive:** the session had expired (DDoS-Guard 403). `annas login` was not run, as instructed, so Anna's was skipped.
- **research-hub could not fetch:**
  - AccessiText: obtained from the author site instead.
  - InferUI: obtained from the author site instead.
  - Dobres 2016 Ergonomics (glance legibility, age/typeface/polarity): substituted Dobres 2018.
  - Hallé 2016 JLAMP (Cornipickle journal version): used the ICST 2015 version.
  - Bisson 2020 "Detecting Responsive Web Design Bugs with Declarative Specifications": not obtained, not summarised.
- **Not pursued:** Zagreb WCAG/APCA survey (abstract-level only via search).

**Time/size:** ~57 MB of PDFs kept (budget 400 MB). The OwlEyes-online demo PDF was deleted (not summarised).
