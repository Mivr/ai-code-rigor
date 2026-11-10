# Sub-topic D -- Interaction-level checks for a touch kiosk

**Scope:** hit-testing correctness, occlusion of interactive elements (scrims/overlays swallowing taps), touch-target size/spacing and their evidence base, dead/no-effect elements, feedback presence and latency, selected-state/mode feedback, minimum display duration of transient messages.
**Primary failure modes:** F1, F2, F3, F4, F9 (+ touch-target checks generally).
**Compiled:** 2026-11-10 by sub-topic agent D. 17 new full-text sources + 8 standard/practitioner/abstract-only entries; 13 reused corpus summaries.
**Tagging:** every claim carries a citation (Author Year), or [standard], [practitioner], [our inference].

---

## 1. Consensus

### 1.1 Dominant consensus

**D-1. Touch target size has a strong, monotonic effect on speed and errors up to a plateau, and the plateau depends on posture and population -- for a standing public kiosk it is about 20 mm.**
- Kiosk, standing, numeric keypad: plateau at 20 mm, no difference 20 vs 25 mm; recommend >= 20 mm (Colle 2004, Table/Sec. 5; n=20 young adults).
- Older adults (53-84): reaction time plateaus at 16.5 mm (high dexterity) vs 19.05 mm (low dexterity); recommend >= 16.5 mm adjacent, 19 mm for better performance (Jin 2007, Sec. 4.1; n=40).
- Older adults (65-95) on smartphones: >97% accuracy needs >= 14 mm taps, >= 17.5 mm swipes (Leitao 2012; n=40).
- Handheld thumb: 9.2 mm discrete, 9.6 mm serial (Parhi 2006; n=20); 7 vs 10 mm n.s. in time (Park 2010; n=30).
- In the wild, error rate climbs steeply below 15 mm and exceeds 40% below 8 mm (Henze 2011; 120 M taps).
- Uncompensated touch needs ~15 mm diameter to capture 95% of touches (Holz 2010, Fig. 7).
- Standards/guidelines are smaller and admit weak evidence: WCAG 24 / 44 CSS px with "no specific data" for 24 px [standard] (W3C 2023 target size); Apple 44 pt, Android 48 dp [practitioner]; ANSI/HFES 9.5 mm, EIA 19.05 mm, ISO = finger-joint breadth (cited in Sesto 2012; Jin 2007).

**D-2. Edge-to-edge spacing between ~0-1 mm and ~10 mm has no measurable effect on accuracy for touch screens with a point hit-test; very large gaps slow users; older/low-dexterity users *prefer* 3-13 mm.**
- 1 vs 3 mm: n.s. for time and errors (Colle 2004) and for force/dwell (Sesto 2012).
- 0-10.5 mm: n.s. for older adults (Leitao 2012).
- 0/3.17/6.35 mm equivalent; 19.05 mm gaps slower; preferences 6.35 mm (44%) (Jin 2007).
- Guidance still asks for spacing (Apple ~12/24 pt; WCAG circle rule) [practitioner][standard] -- partly to prevent overlap/adjacency errors and partly preference.

**D-3. Every user action needs immediate, perceivable feedback; for a touch button the visual response should arrive within roughly 100 ms (control-task band 50-200 ms).**
- Visual feedback latency guideline 30-85 ms; quality drops at 100-150 ms (Kaaresoja 2016, Table 4-2; n=24).
- Tap land-on latency not discerned below ~20 ms; 85% cannot tell 40 ms from 1 ms (Jota 2013, reused); tapping JND 69 ms direct (Deber 2015, reused).
- Control tasks need 50-200 ms; echo delays ~200 ms raise typing errors (Williges 1982 via Dabrowski 2011).
- 0.1 s "instantaneous" limit [practitioner] (Nielsen 1993; Miller 1968 secondary).
- Per-key confirmation on touchscreens raised correct entries from 69.6% to 82.7% (Hoggan 2008; tactile).
- Slips are detected only if feedback exposes the discrepancy (Norman 1981).

**D-4. The element that visibly appears under the finger is not necessarily the element that receives the event; this must be checked by hit-testing the actual stacking/input order, and users cannot detect violations.**
- Default overlays capture all touches and do not propagate; 0/20 users noticed (Fratantonio 2017).
- Industry automation defines clickability as "topmost hit target at the action point" (WebDriver "obscured"; Playwright "Receives Events") [standard][practitioner].
- Deterministic visibility-fault detection over UI states (overlays/scrims covering buttons) has ~97-99% precision; geometry-only layout checkers miss it (Ryou 2018, C agent).
- Opacity-toggle differential verification of overlaps (Althomali 2019, reused).

### 1.2 Secondary consensus

**S-1. Visible mode/selected state reduces mode errors but visual indicators alone are weak; user-maintained (held) states are much stronger.** Visual feedback explained 4.1-4.8% of variance vs 11-15.6% for a held pedal (Sellen 1992); Monk 1986 (via Sellen) cut mode errors ~1/3 with mode-dependent sound. Formal work treats invisible mode as "error states" (Degani 2002; Rushby 2002, both B/C agents).

**S-2. Echo is necessary but not sufficient: with serial keypads users watch the keys (271 s) far more than the display (26 s) and miss echoed errors** (Oladimeji 2011). Number-entry UIs that silently ignore keystrokes cause out-by-10 errors (Thimbleby 2010, B agent).

**S-3. Transient information must survive long enough to be read; reading-rate evidence gives ~238 wpm average, 175 wpm low end of normal adults, slower for older and L2 readers** (Brysbaert 2019). Standards treat auto-disappearing information as a time limit unless available elsewhere [standard] (W3C 2023 timing adjustable). Platform toast/snackbar defaults (1.5-3.5 s) are explicitly meant to be lengthened by an accessibility timeout [practitioner] (Apple Google 2025).

**S-4. Touches are systematically offset from target centres (posture, user, grip), so single-point checks under-represent real touches** (Holz 2010; Henze 2011).

### 1.3 Bias risks
- **Samples:** young, able-bodied, often male or company-employee samples (Colle 2004: mean 19.5 y; Parhi 2006: 17/20 male; Kaaresoja 2016: Nokia staff). Older/disabled evidence is from small studies (Jin 2007, Leitao 2012, Sesto 2012).
- **Device era:** 2004-2012 resistive/capacitive hardware and PDAs; modern capacitive panels may be more accurate, which would bias size thresholds upward (conservative).
- **Standards' thresholds are policy compromises** (WCAG 24 px explicitly not data-derived; WCAG 2.2.1's 20 s/10x from "clinical experience") [standard].
- **Response-time folklore:** the 0.1/1/10 s limits trace to Miller 1968 expert judgement (Dabrowski 2011); empirical anchors are Kaaresoja 2016, Jota 2013, Deber 2015.
- **Reading-rate data** are for continuous prose in English; UI messages are short and must first be noticed, and no Bulgarian data exist (Brysbaert 2019).
- **Publication bias** towards significant size effects; null spacing results are consistent across studies, which argues against it for spacing.

---

## 2. Catalogue of deterministic interaction checks

All checks below assume these gate inputs [our inference]:
- **R:** the set of reachable view states, from enumerating the deterministic reactor (events -> view state), including timer-driven states (simulated clock).
- **G(s):** the scene graph for state s: nodes with role, rect, z/layer, style id, text msgid+args, intents, enabled/inert flags, and **input policy** per layer (capture / pass-through / capture-and-dismiss).
- **hit(s, p):** the *production* hit-test function (the one lvrender/LVGL input uses), not a reimplementation.
- **T:** an event trace with a logical clock (frames/steps) and per-frame scene snapshots; optional software-rendered pixels for visual-diff checks.
- **ppi:** the physical pixel density of the kiosk panel (px = mm x ppi / 25.4).

### (a) Reachability / occlusion of interactive nodes via hit-testing (F3, F4)
- **Rule:** for every s in R and every node n with an intent, enabled, not inert, effectively visible: for every sample point p in P(n), hit(s, p) is n or a descendant of n.
- **P(n):** centre + an inset grid (e.g. 3x3 at 20/50/80% of width/height) **plus** a disc of radius ~7.5 mm around the centre clipped to n (Holz 2010: ~15 mm diameter needed for 95% capture; Henze 2011: systematic offsets) [our inference on the exact grid].
- **Effective target area variant:** area(n) minus area where hit(s,p) != n must still meet the size threshold (WCAG overlap-exclusion rule [standard]; ATF hit-rect with clipping [practitioner]).
- **Overlay policy rule:** if a layer L captures input over n, then either n is marked inert in s (declared modal, with visible dimming), or the check fails. A non-modal banner with a capture-and-dismiss scrim over enabled keys is exactly F3 (Fratantonio 2017 semantics) [our inference].
- **Stability precondition:** n's rect is identical for >= 2 consecutive frames before testing (Playwright "Stable") [practitioner]; test only settled states and, separately, every animation frame for "transient interception" warnings.
- **Determinism:** deterministic (pure function of G(s) and hit()).
- **Known FPs:** intentionally modal dialogs not marked inert (fix: make inert-ness explicit); decorative non-interactive overlays with pass-through that are misdeclared; nodes partly clipped by scroll containers by design.
- **Known FNs:** if hit() used by the gate differs from the runtime input path (use the same code); states not in R (timers, races, external backend replies); points outside P(n); invisible-but-clickable nodes passing as "visible" when alpha is 0 (Playwright counts opacity:0 as visible [practitioner]) -- add a separate "clickable implies alpha >= threshold and contrast" rule.
- **Citations:** Ryou 2018 (C agent; ~97-99% precision on real pages), Fratantonio 2017, W3C Playwright 2025, Althomali 2019 (reused), Holz 2010, Henze 2011, W3C 2023 target size.

### (b) Minimum size and spacing (touch targets)
- **Size rule:** effective hit rect (from (a)) contains an axis-aligned square of side S_role (WCAG square-fit procedure [standard]), where S_role is physical mm converted with ppi.
  - keypad key (serial entry): **S >= 20 mm** (Colle 2004; Jin 2007 low dexterity 19.05 mm).
  - other primary buttons/toggles on a public kiosk: **S >= 19 mm** recommended, **>= 16.5 mm** hard floor for adjacent buttons (Jin 2007), **>= 14 mm** absolute floor (Leitao 2012).
  - never below **44 CSS px / 44 pt / 48 dp** equivalents [standard][practitioner].
- **Spacing rule:** hit rects of distinct intents must not overlap (hard; WCAG overlap exclusion [standard]; ATF duplicate bounds [practitioner]); edge-to-edge gap **>= 1 mm** hard (Colle 2004), **>= 3 mm** recommended (Jin 2007 band 3.17-12.7 mm; older-adult preference 6.35 mm); for any undersized target apply the WCAG circle test scaled to S_role [standard].
- **Edge rule:** targets within ~10 mm of the screen edge get no reduction (unlike ATF's 32 dp edge allowance) because edges have higher error (Henze 2011: 31.7% vs 17.6%; WCAG 2.5.5 advice [standard]) [our inference].
- **Determinism:** deterministic.
- **FPs:** inline text links (none expected on kiosk); deliberately dense staff screens (declare an explicit exemption list, reviewed).
- **FNs:** physical size depends on correct ppi config; parallax/viewing angle at mounting height not modelled.
- **Citations:** Colle 2004, Jin 2007, Leitao 2012, Parhi 2006, Park 2010, Henze 2011, Holz 2010, Sesto 2012, W3C 2023 target size, Google 2025 ATF, Apple Google 2025, Bi 2013 FFitts (reused, for predicted time if a soft metric is wanted).

### (c) No-effect (dead element) detection (F4, F9, F3)
- **Rule:** for every s in R and every enabled interactive n, synthesise a tap at a point where (a) holds; let s' = reactor(s, tap). Require **model change** (s' != s, excluding clocks) **and visible change** (G(s') differs from G(s) in style/text/structure, or pixels differ) within N frames -- unless n declares `noop_ok` with a required feedback (e.g. pressed flash).
- **Invisible-state-change variant:** if s' != s but the rendered scene (or pixels) is unchanged, fail as "state change not observable" -- catches F9-type defects where the toggle's model changes but nothing shows (Degani 2002 "error state"; Harrison 2019 "action/state feedback" template, B/C agents; Norman 1981) [our inference].
- **Static complement:** every intent id on a node must have a non-identity reducer transition in at least one state (reactor coverage), and no two clickable nodes share bounds (ATF DuplicateClickableBounds rationale: clickable containers that do not process clicks) [practitioner].
- **First-tap variant:** if the first tap only removes an overlay and a second tap is needed to reach n, report "tap swallowed" (F3; rage-click pattern [practitioner] Microsoft 2025).
- **Determinism:** deterministic given the deterministic reactor; pixel-diff variant deterministic with the software renderer.
- **FPs:** legitimately idempotent taps (re-selecting the selected language) -- require declared `noop_ok` + pressed-state feedback; actions whose effect is off-screen (sound, external locker unlock) -- declare the external effect as an observable trace event.
- **FNs:** effect occurs only after an async backend reply not modelled; "wrong effect" (tapping EN switches to BG) needs oracle comparison with the Rust app (B agent).
- **Citations:** Microsoft 2025 Clarity (dead click = no visual change, no navigation), Google 2025 ATF, Alshayban 2020, Su 2021 Genie (B agent), Salehnamadi 2022 Groundhog (reused; differential touch vs AT action), Norman 1981, Degani 2002, Harrison 2019.

### (d) Feedback within N frames (F2, F9, F4)
- **Rule:** for every accepted press event e at frame f_e on node n, the first frame f_fb at which n shows a pressed/selected style *or* the associated echo/target field changes satisfies f_fb - f_e <= N.
  - **N_visual = 85 ms -> <= 5 frames at 60 Hz** (Kaaresoja 2016 visual guideline 30-85 ms; quality drop 100-150 ms).
  - Preferred: **<= 2 frames (~33 ms)**, matching the point of subjective simultaneity 32 ms (Kaaresoja 2016) and imperceptible tap latency < 20-40 ms (Jota 2013, reused).
  - Logical-latency form: the echo must be present in the view state returned by the **same reducer step** that consumed the key event (0 logical frames), and lvrender must present it in the next frame [our inference] -- deterministic and immune to wall-clock noise.
  - If an operation takes > 1 s (e.g. code verification), a busy indicator must appear within 1 s [practitioner] (Nielsen 1993); > 10 s needs progress + cancel [practitioner].
- **Echo correspondence (F2):** for each accepted digit, the echo field's length increases by exactly 1 (masked or clear); Backspace decreases by 1; no digit accepted without echo (Thimbleby 2010, B agent; Oladimeji 2011; Colle 2004 and Sesto 2012 protocols both echo digits). Echo field placed adjacent to the keypad and sized for reading [our inference from Oladimeji 2011's gaze data].
- **Determinism:** deterministic in frames; real input-to-photon latency needs hardware measurement (Casiez 2015, Schmid 2018, reused) -- out of gate scope.
- **FPs:** debounced repeated taps (declare debounce); keys that intentionally do nothing when the field is full (declare and require a "full" feedback).
- **FNs:** frame-count proxy ignores X11/compositor/touch-controller latency (typically 50-200 ms on commercial devices: Ng 2012, reused).
- **Citations:** Kaaresoja 2016, Dabrowski 2011, Hoggan 2008, Jota 2013, Deber 2015, Ng 2012 (reused), Nielsen 1993, Miller 1968 (secondary), Oladimeji 2011, Colle 2004, Sesto 2012.

### (e) Minimum dwell for transient messages (F1, F8)
- **Rule:** for every message node m (role = message/banner/dialog with msgid) created at frame f_c, it is not removed before f_c + D(m) except by (i) an explicit user dismiss event on m, or (ii) replacement by a node conveying the same msgid; and its lifetime is >= the spec value if the spec states one (oracle says 8 s for the rejection dialog).
- **D(m) from reading rate:** D = T_notice + chars(m, locale) / (r x c), with r = **175 wpm** (low end of normal adult silent reading; Brysbaert 2019) or **~130 wpm** for an older/L2-inclusive kiosk [our inference: ~0.75 x 175], c = 5.6 characters/word (4.6 letters + space; Brysbaert 2019 word-length correction), T_notice = 1 s [our inference]. Floor at 3.5 s (longest platform toast default [practitioner]) and, for error/rejection messages, prefer "until acknowledged" (WCAG 2.2.1 [standard]).
  - Example: 45-character English rejection text -> 45 / (175 x 5.6 / 60 = 16.3 cps) = 2.8 s + 1 s = **3.8 s**; at 130 wpm **4.7 s**; a Bulgarian translation ~20-30% longer -> ~5-6 s [our inference]. All far above the F1 escape (60 ms).
- **Determinism:** deterministic (lifetimes measured in the logical clock of the trace).
- **FPs:** messages superseded by a more important state (declare precedence rules).
- **FNs:** message present but occluded (combine with (a)-style visibility of the message itself) or clipped/truncated (F6/F8, C agent); "read" is not "understood".
- **Citations:** Brysbaert 2019, W3C 2023 timing adjustable, Apple Google 2025 (Snackbar 1.5/2.75 s, Toast 2/3.5 s, getRecommendedTimeoutMillis), Legge 2011 (A/E agent, reading vs print size).

### (f) Selected-state visibility (F9)
- **Rule 1 (distinctness):** in every toggle/segmented group, for every state s, the selected item's style id differs from its own unselected style and from all unselected siblings; differences must involve more than colour alone (WCAG 1.4.1 Use of Color [standard]) and the state indicator must reach >= 3:1 contrast against adjacent colours (WCAG 1.4.11 Non-text Contrast [standard], not re-fetched in this run).
- **Rule 2 (persistence):** the indicator is proactive and persistent for the whole time the mode holds (Sellen 1992's classification: sustained, proactive), not a transient flash.
- **Rule 3 (exactly one / consistent):** single-select groups have exactly one selected item in every reachable state, and the a11y tree's selected/checked state equals the visual state (differential between visual and accessibility layers: Salehnamadi 2022 Groundhog, reused) [our inference].
- **Rule 4 (pixel-level):** render the group with selection on item i vs item j; the pixel difference inside each item's rect must be non-empty and above a minimum changed-area fraction [our inference; no evidence for the fraction].
- **Determinism:** deterministic.
- **FPs:** groups intentionally allowing zero selection (declare).
- **FNs:** distinct but not salient indicators (thin underline) pass Rule 1 -- salience thresholds lack evidence (gap).
- **Citations:** Sellen 1992, Norman 1981, Sesto 2012 (button colour change as standard feedback), Rushby 2002 / Degani 2002 (B/C agents), Salehnamadi 2022 (reused).

### (g) Supporting checks
- **Clickable implies visible and labelled:** alpha above threshold, not clipped below S_role, has a non-empty label in the a11y tree (Chen 2021, reused; Alshayban 2020).
- **Visible-looking implies clickable or non-interactive role:** nodes styled as buttons must have an intent (prevents "looks tappable, isn't" -- dead affordance) [our inference].
- **Long-dwell tolerance:** activation must not depend on touch durations shorter than ~2x the non-disabled dwell; gross-motor users dwell 2.3x longer (Sesto 2012) -- avoid long-press semantics on primary keys [our inference].

---

## 3. Evidence-based threshold table

| Quantity | Threshold for gate | Evidence | Strength |
|---|---|---|---|
| Keypad key size (standing kiosk, serial code entry) | **>= 20 mm square** (hard) | Colle 2004 (plateau at 20 mm; n=20); Jin 2007 (19.05 mm low dexterity) | Medium (lab, young sample; corroborated by older-adult data) |
| Other buttons/toggles, public kiosk | **>= 19 mm** recommended; **>= 16.5 mm** hard floor | Jin 2007 (n=40, 53-84 y) | Medium |
| Absolute floor for any touch target | **>= 14 mm** and >= 44 CSS px / 48 dp | Leitao 2012 (97% accuracy criterion); W3C 2023 [standard]; Apple/Android [practitioner] | Medium / policy |
| Handheld reference (not kiosk) | 9.2-9.6 mm | Parhi 2006; Park 2010 | Medium (lower bound only) |
| Size needed for 95% capture without offset compensation | ~15 mm diameter | Holz 2010 | Medium (n=12) |
| Error rate in the wild | steep rise < 15 mm; > 40% < 8 mm | Henze 2011 (120 M taps) | Strong for relative trend |
| Hit-test sampling region | centre + inset 3x3 grid + 7.5 mm-radius disc (clipped) | Holz 2010; Henze 2011 | [our inference] on exact grid |
| Hit-rect overlap between distinct intents | **0 (none)** | W3C 2023 overlap exclusion [standard]; Google ATF [practitioner] | Policy + mechanism |
| Edge-to-edge spacing | **>= 1 mm** hard; **>= 3 mm** recommended (3.17-12.7 mm band) | Colle 2004, Sesto 2012 (1 vs 3 mm n.s.); Jin 2007 (prefs 6.35 mm); Leitao 2012 (0-10.5 mm n.s.); Apple 12/24 pt [practitioner] | Strong for "small is fine"; preference only for larger |
| Pressed-state / echo visual latency | **<= 85 ms (<= 5 frames @60 Hz)**; target **<= 2 frames**; logical: same reducer step | Kaaresoja 2016 (30-85 ms; quality drop 100-150 ms); Jota 2013; Deber 2015 (tap JND 69 ms) | Medium-strong |
| Control-task response band | 50-200 ms | Dabrowski 2011 review (Williges echo ~200 ms raises errors) | Medium (old studies) |
| Busy indicator for slow operations | appear by **1 s**; progress + cancel beyond **10 s** | Nielsen 1993 [practitioner]; Seow 2008 via Dabrowski 2011 | Practitioner |
| Transient message minimum dwell | **D = 1 s + chars / (r x 5.6 / 60)** with r = 175 wpm (130 wpm inclusive); floor **3.5 s**; error messages: until acknowledged or >= spec (8 s) | Brysbaert 2019 (238 wpm mean, 175-300 range); W3C 2.2.1 [standard]; platform defaults 1.5-3.5 s [practitioner] | Medium (reading-rate strong; transfer to UI messages is inference) |
| WCAG time-limit extension | warn and allow >= 20 s to extend; adjust >= 10x | W3C 2023 timing adjustable [standard] | Policy ("clinical experience") |
| Touch dwell variability | gross-motor dwell 2.3x non-disabled | Sesto 2012 | Medium |
| Selected-state indicator contrast | >= 3:1, not colour-only | WCAG 1.4.11 / 1.4.1 [standard] | Policy |
| Physical conversion | px = mm x ppi / 25.4 (e.g. 20 mm = 111 px on 15.6" 1080p @141 ppi; 81 px on 21.5" 1080p @102 ppi) | arithmetic [our inference] | -- |

---

## 4. Mapping to failure modes F1-F11

| F | Defect | Checks from this sub-topic | Would it catch the escape? | Key evidence |
|---|---|---|---|---|
| F1 | rejection dialog removed ~60 ms after creation | (e) minimum dwell / lifetime + "removed only by user dismiss or same-msg replacement" | **Yes** (60 ms << any D >= 3.5 s; spec 8 s) | Brysbaert 2019; W3C 2.2.1 [standard]; Apple Google 2025 [practitioner] |
| F2 | no visible echo while typing code | (d) feedback within N frames; echo correspondence (+1 per digit); (c) visible-change on key tap | **Yes** (no echo -> no change in echo field; pressed state alone may pass (c) but fails echo correspondence) | Kaaresoja 2016; Dabrowski 2011; Hoggan 2008; Oladimeji 2011; Colle 2004; Sesto 2012 |
| F3 | banner overlay + tap-outside scrim covers a key and swallows first tap | (a) hit-test at sample points; overlay input-policy rule (capture over non-inert enabled nodes fails); effective-area size; (c) first-tap-swallowed variant | **Yes** | Fratantonio 2017; Ryou 2018 (C); W3C/Playwright; Holz 2010; Henze 2011; W3C 2023 overlap rule |
| F4 | EN language toggle does nothing | (c) model+visible change on tap; static intent-handled coverage | **Yes** | Microsoft 2025 [practitioner]; Google ATF [practitioner]; Su 2021 Genie (B); Norman 1981 |
| F5 | locker number tiny, split glyphs | not served (hierarchy/emphasis: A/E agent); (g) "visible-looking implies role" irrelevant | No | -- |
| F6 | countdown clipped, format differs | partially: (g) clipping of interactive labels; (e) message node must be fully visible | Partial (only if the countdown is a message/interactive node) | C agent |
| F7 | all text one size, left-aligned | not served (A/E) | No | -- |
| F8 | no wrapping, Bulgarian truncated | partial: (e) dwell scales with localized char count; truncation check belongs to C | Partial | Brysbaert 2019 (word-length/language effects) |
| F9 | toggles show no selected state | (f) distinctness/persistence/exactly-one/a11y-consistency; (c) invisible-state-change variant | **Yes** | Sellen 1992; Norman 1981; Degani 2002 (B); WCAG 1.4.1/1.4.11 [standard] |
| F10 | splash missing version line | not served (content parity: B/C) | No | -- |
| F11 | staff report missing sections | not served (content parity) | No | -- |

---

## 5. Hard-to-catch list

1. **Gate hit-test vs runtime input routing divergence.** If the gate re-implements hit-testing (e.g. from scene z-order) while LVGL routes by its own object tree/flags (clickable, hidden, event bubbling, scrim objects), the check can pass while the device fails. Mitigation: call the production hit-test path in headless mode [our inference].
2. **Timer- and backend-driven states** (banner appearing after a timeout, async verification reply) outside the enumerated R; F1-style removals often come from such events. Requires the simulated clock and modelled backend replies in R [our inference].
3. **Mid-animation interception** (a sliding banner briefly covers keys). Stable-state checks miss it; per-frame checks may over-report. Report as warning with duration [our inference].
4. **Invisible-but-capturing layers** (alpha 0 scrims) and **visible-but-inert affordances**; "visible" definitions (Playwright counts opacity:0 as visible [practitioner]) hide them.
5. **Salience of feedback and selected state.** A diff exists but is too small/low-contrast to notice; visual indicators are weak even when present (Sellen 1992). No evidence-based minimum change area.
6. **Feedback placement vs gaze.** Echo exists but is far from where users look (Oladimeji 2011: 271 s on keys vs 26 s on display).
7. **Real input-to-photon latency** (touch controller + X11 + panel) is not visible in logical frames; commercial devices measured 50-200 ms (Ng 2012, reused). Needs hardware measurement (Casiez 2015; Schmid 2018, reused).
8. **Physical touch offset/parallax at the kiosk's mounting height and angle** -- sample regions use generic offsets (Holz 2010; Henze 2011), not kiosk-specific ones.
9. **Wrong-but-visible effect** (tapping EN switches to the wrong language): (c) passes; needs oracle comparison (B agent).
10. **Reading vs understanding:** dwell time ensures time to read, not comprehension; short UI messages after gaze shifts have no direct reading-rate data.
11. **Long-dwell / tremor users** whose touches register as long-press or double tap (Sesto 2012) -- depends on input configuration, not scene geometry.

---

## 6. Evidence gaps

- **No standing-kiosk study with older or disabled users**; kiosk data are young adults (Colle 2004); older-adult data are seated or handheld (Jin 2007; Leitao 2012); disabled data are seated (Sesto 2012).
- **No empirical study of minimum display duration for transient UI error messages** (toasts/banners); thresholds must be derived from reading-rate meta-analysis plus inference (Brysbaert 2019) and standards with "clinical experience" provenance [standard]. No Bulgarian reading-rate data.
- **Spacing:** evidence shows small gaps are harmless for accuracy; recommended larger gaps rest on preference (Jin 2007) and guidelines; the WCAG 24 px circle has no published data [standard].
- **Selected-state salience:** no quantitative evidence for the minimum visual difference (area/contrast) that makes a selected state noticeable on touch kiosks; mode-error evidence is from 1990s text editors (Sellen 1992).
- **Touch feedback latency for visual-only feedback on large kiosk panels and older users** is unstudied; Kaaresoja 2016 is handheld with engineer participants.
- **Deterministic dead-element detection** in research is mostly fuzzing/metamorphic (Genie, B agent) or AT-differential (Groundhog, reused) on Android; no paper formalises "every interactive node has an observable effect" as a static+dynamic gate -- our (c) is synthesis [our inference].
- **Not obtained:** Miller 1968 (secondary only), Kaaresoja et al. 2014 TAP paper (covered by thesis), Shneiderman 1984 (ACM CSUR), Park et al. 2008 MobileHCI (covered by Park 2010), Kobayashi 2011, Doherty & Thadani 1982 (grey literature; its 400 ms productivity claim not used), Card, Moran & Newell 1983 (book), Monk 1986 (cited via Sellen 1992).

---

## 7. Sources

### 7.1 Reused corpus (read summaries; not re-downloaded)
- `(local corpus) 2013 - FFitts law finger touch.md`
- `.../lui-usability-metrics/summaries/Chen 2021 - Accessible or not Android app accessibility.md`
- `.../lui-usability-metrics/summaries/Salehnamadi 2021 - Latte assistive-service driven testing.md`
- `.../lui-usability-metrics/summaries/Salehnamadi 2022 - Groundhog accessibility crawler.md`
- `.../lui-usability-metrics/summaries/Salehnamadi 2023 - A11y Puppetry AT record and replay.md`
- `.../lui-usability-metrics/summaries/Mateus 2021 - Mapping of accessibility problems automated vs inspection vs users.md`
- `.../lui-usability-metrics/summaries/Huq 2026 - Automated interventions vs actual screen reader user experience.md`
- `.../lui-usability-metrics/summaries/Zhong 2026 - TaskAudit agentic functiona11ity errors.md`
- `.../lui-usability-metrics/summaries/Moran 2018 - Automated reporting of GUI design violations.md`
- `.../lui-usability-metrics/summaries/Yang 2021 - Visual design smells vs Material guidelines.md`
- `(local corpus) 2013 - Touch latency effects.md`
- `.../lui-gpu-rendering/summaries/Ng 2012 - Low-latency direct touch.md`; `.../Ng 2014 - Stylus latency perception.md`; `.../Deber 2015 - Latency improvement perception.md`; `.../Casiez 2015 - Measuring end-to-end latency.md`
- `(local corpus) 2018 - Low-cost end-to-end latency measurement.md`; `.../Liu 2025 - GUIWatcher.md`
- `(local corpus) 2019 - Visual verification layout failures.md`
- Other agents' Articles cited (not modified): `visual-gate-2026-11/Articles/{Ryou 2018 - Visibility faults HTML5, Su 2021 - Genie functional fuzzing, Thimbleby 2010 - Reducing number entry errors, Rushby 2002 - Model checking mode confusion, Degani 2002 - Formal verification of human-automation interaction, Harrison 2019 - UI verification templates, Zaeem 2014 - QUANTUM interaction oracles, Legge 2011 - Print size reading}/Summary.md`

### 7.2 New sources (this agent) -- folder `visual-gate-2026-11/Articles/<name>/Summary.md`
Full text obtained (PDF in `PDFs/`, text in `Extracted Texts/`):
1. Colle, H.A., Hiszem, K.J. (2004). Standing at a kiosk: effects of key size and spacing on touch screen numeric keypad performance and user preference. Ergonomics 47(13):1406-1423. DOI 10.1080/00140130410001724228
2. Parhi, P., Karlson, A.K., Bederson, B.B. (2006). Target size study for one-handed thumb use on small touchscreen devices. MobileHCI '06:203-210. DOI 10.1145/1152215.1152260
3. Sesto, M.E., Irwin, C.B., Chen, K.B., Chourasia, A.O., Wiegmann, D.A. (2012). Effect of touch screen button size and spacing on touch characteristics of users with and without disabilities. Human Factors 54(3):425-436. DOI 10.1177/0018720811433831
4. Jin, Z.X., Plocher, T., Kiff, L. (2007). Touch screen user interfaces for older adults: button size and spacing. UAHCI 2007, LNCS 4554:933-941. DOI 10.1007/978-3-540-73279-2_104
5. Park, Y.S., Han, S.H. (2010). Touch key design for one-handed thumb interaction with a mobile phone: effects of touch key size and touch key location. Int. J. Industrial Ergonomics 40(1):68-76. DOI 10.1016/j.ergon.2009.08.002
6. Leitao, R., Silva, P.A. (2012). Target and spacing sizes for smartphone user interfaces for older adults: design patterns based on an evaluation with users. PLoP 2012. (no DOI located)
7. Henze, N., Rukzio, E., Boll, S. (2011). 100,000,000 taps: analysis and improvement of touch performance in the large. MobileHCI '11:133-142. DOI 10.1145/2037373.2037395
8. Holz, C., Baudisch, P. (2010). The generalized perceived input point model and how to double touch accuracy by extracting fingerprints. CHI '10:581-590. DOI 10.1145/1753326.1753413
9. Alshayban, A., Ahmed, I., Malek, S. (2020). Accessibility issues in Android apps: state of affairs, sentiments, and ways forward. ICSE 2020:1323-1334. DOI 10.1145/3377811.3380392
10. Fratantonio, Y., Qian, C., Chung, S.P., Lee, W. (2017). Cloak and Dagger: from two permissions to complete control of the UI feedback loop. IEEE S&P 2017. DOI 10.1109/SP.2017.39
11. Dabrowski, J., Munson, E.V. (2011). 40 years of searching for the best computer system response time. Interacting with Computers 23(5):555-564. DOI 10.1016/j.intcom.2011.05.008
12. Kaaresoja, T.J. (2016). Latency guidelines for touchscreen virtual button feedback. PhD thesis, University of Glasgow (theses.gla.ac.uk/7075); related: Kaaresoja, Brewster, Lantz (2014) ACM TAP 11(2):9, DOI 10.1145/2611387 (not obtained)
13. Hoggan, E., Brewster, S.A., Johnston, J. (2008). Investigating the effectiveness of tactile feedback for mobile touchscreens. CHI '08:1573-1582. DOI 10.1145/1357054.1357300
14. Oladimeji, P., Thimbleby, H., Cox, A. (2011). Number entry interfaces and their effects on error detection. INTERACT 2011, LNCS 6949:178-185. DOI 10.1007/978-3-642-23768-3_15
15. Sellen, A.J., Kurtenbach, G.P., Buxton, W.A.S. (1992). The prevention of mode errors through sensory feedback. Human-Computer Interaction 7(2):141-164. DOI 10.1207/s15327051hci0702_1
16. Norman, D.A. (1981). Categorization of action slips. Psychological Review 88(1):1-15. DOI 10.1037/0033-295X.88.1.1
17. Brysbaert, M. (2019). How many words do we read per minute? A review and meta-analysis of reading rate. Journal of Memory and Language 109:104047. DOI 10.1016/j.jml.2019.104047

Standard / practitioner / secondary (no PDF):
18. W3C (2023). WCAG 2.2 SC 2.5.8 / 2.5.5 Target Size + Understanding docs [standard] -- `Articles/W3C 2023 - WCAG 2.2 target size`
19. W3C (2023). WCAG 2.2 SC 2.2.1 Timing Adjustable + Understanding [standard] -- `Articles/W3C 2023 - WCAG 2.2 timing adjustable`
20. W3C WebDriver2 Element Click / interactability + Playwright actionability [standard][practitioner] -- `Articles/W3C Playwright 2025 - Hit-test actionability checks`
21. Google Accessibility Test Framework: TouchTargetSizeCheck (48 dp; 32 dp edge/IME), DuplicateClickableBoundsCheck [practitioner] -- `Articles/Google 2025 - ATF touch target and duplicate clickable bounds`
22. Apple HIG accessibility (44x44 pt; 12/24 pt padding), Android 48 dp, Material Snackbar 1500/2750 ms, Toast 2000/3500 ms, AccessibilityManager.getRecommendedTimeoutMillis [practitioner] -- `Articles/Apple Google 2025 - Platform target size and transient message guidance`
23. Nielsen, J. (1993). Response times: the 3 important limits [practitioner] -- `Articles/Nielsen 1993 - Response time limits`
24. Microsoft Clarity semantic metrics: dead clicks, rage clicks [practitioner] -- `Articles/Microsoft 2025 - Clarity dead and rage clicks`
25. Miller, R.B. (1968). Response time in man-computer conversational transactions. AFIPS FJCC 33:267-277. DOI 10.1145/1476589.1476628 -- **secondary/abstract only** -- `Articles/Miller 1968 - Response time conversational`

---

## 8. Exhaustion report

- **Searches (research-hub):** ~19 title/author queries (all brief targets) + 3 exploratory queries (elderly touchscreen, unresponsive widgets, kiosk older adults). Web searches: 9 (open copies of Brysbaert, Alshayban, Kaaresoja, Miller, Oladimeji, Jin; toast duration studies; Android toast constants). Exploratory query for "detecting unresponsive/non-functional widgets" returned nothing relevant beyond the B agent's Genie/Nighthawk; exploratory query for "toast/notification display duration study" returned only design-system guidance (no empirical study found).
- **Downloads:** research-hub batch 19 requested -> 12 succeeded (2 were HTML stubs, removed); 2 more via second batch (Park 2010 ok, Kobayashi 2011 HTML stub removed). Open copies: Brysbaert (gwern mirror), Alshayban (NSF PAR), Oladimeji (IFIP DL), Leitao (Maynooth), Kaaresoja thesis (Glasgow), Jin 2007 (Springer open PDF).
- **Failures:** Miller 1968 (iamue scan timed out twice; ACM paywall), Kaaresoja 2014 TAP (paywall; thesis used), Shneiderman 1984 (paywall), Park et al. 2008 MobileHCI (paywall; journal version used), Kobayashi 2011 (stub), Anna's Archive session expired (DDoS-Guard 403; `annas login` not run per rules). Not attempted: Doherty & Thadani 1982 (grey), Monk 1986, Card et al. 1983 (book).
- **Totals:** 17 new PDFs, **24 MB**; 17 extracted texts; 25 new Summary.md (17 full-text, 7 standard/practitioner, 1 secondary); 13+ reused corpus summaries; 8 other-agent Articles cited.
- **Saturation judgement:** target size/spacing and feedback latency are well saturated for the purpose (multiple converging studies); minimum message duration and selected-state salience are genuinely thin in the literature (gaps above), not a search failure.
