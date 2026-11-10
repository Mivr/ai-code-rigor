# Sub-topic C -- Systematic GUI state exploration and temporal properties of UI traces

**For:** deterministic, generic visual/UI gate for the C lockers kiosk (pure reactor -> scene graph -> LVGL -> X11; Rust app as oracle). See `_BRIEF.md`.
**Scope:** model-based GUI testing, GUI ripping, event-flow graphs, state abstraction/equivalence, GUI coverage criteria, bounded exhaustive exploration of a pure reactor, temporal properties of UI traces (bounded response, minimum dwell, liveness), LTL/MTL runtime verification, specification patterns, formal verification of interactive systems / HMI (feedback, mode visibility, consistency, reversibility).
**Corpus:** 26 new sources (all full text; PDFs in `PDFs/`, extracts in `Extracted Texts/`, summaries in `Articles/<Author Year - Short>/Summary.md`) + reused summaries from earlier corpora (Section 8).
**Tags:** every claim carries a citation (Author Year, with table/section where useful), **[practitioner]** or **[our inference]**. Paraphrased throughout; no long quotes.

---

## 0. Verdict at a glance

1. **The kiosk can do what GUI-testing research has been approximating for 25 years: exact, exhaustive, bounded exploration of the UI state machine.** Research tools spend most of their effort reverse-engineering a model (ripping, crawling) and guessing state equivalence (Memon 2003; Mesbah 2012; Baek 2016; Yandrapally 2020). A pure reactor plus an event catalogue makes the model exact and resets free; BFS with an owner-declared abstraction key replaces all of it [our inference, supported by Rushby 2002 who did exactly this with Murphi in 0.24 s].
2. **The escaped defects F1, F2, F4, F9 are oracle gaps, not reachability gaps.** All the relevant states were reachable by the existing tests; what was missing were *generic* properties over traces. The HMI literature already formalised those properties as reusable templates: action/state feedback, visibility (mode iff indicator), consistency, reversibility, completeness (Harrison 2019; Bolton 2013; Degani 2002), and the real-time specification literature gives the timing forms: bounded response, minimum duration, bounded invariance (Konrad 2005; Dwyer 1999).
3. **State properties must be written in bounded (safety) form over virtual time.** Unbounded liveness ("eventually feedback") is not monitorable on finite traces -- 43 of 97 Dwyer-pattern formulas yield a monitor that can never conclude (Bauer 2011, Sec. 2.5). Quickstrom solved the same problem for web UIs with step-bounded LTL and found faults in 20 of 43 TodoMVC implementations (O'Connor 2022, Table 1).
4. **Coverage target:** 100% of reachable (abstract state, enabled event) pairs, including "illegal" pairs (taps that should do nothing; Belli 2001), as the PR floor; event-triple coverage nightly, because event-triple coverage roughly doubles the odds of fault detection per +10 points (Strecker 2008, CWS reduced model).
5. **State abstraction is the crux and the main risk.** Similarity thresholds do not work (best universal-threshold crawl F1 0.24; RTED model F1 decays 0.95 -> 0.45 during a crawl; Yandrapally 2020). Too fine explodes on clocks and dynamic text, too coarse hides bugs (Baek 2016). Declare the key as a projection of reactor view state; never learn it [our inference].

---

## 1. Consensus

### 1.1 Dominant consensus (multi-source, high confidence)

| # | Claim | Supporting sources | Strength |
|---|---|---|---|
| D1 | GUI behaviour is best modelled as a finite event/state graph and tested by event sequences; adequacy should be measured by event-based coverage (event, event-pair, length-n), not code coverage | Memon 2001 (statement coverage ~92% with single events, Sec. 5.2); Memon 2007 [reused]; Yuan 2011 [reused]; Strecker 2008; Belli 2001; Amalfitano 2015 (adjacent edge pairs); Prowell 1999 arc coverage [reused] | Strong, but mostly 2001-2011 desktop/Java evidence |
| D2 | Longer/contextual sequences find faults that pairs miss | Strecker 2008 (odds x2.14 per +10% triple coverage, CWS); Yuan 2011 [reused]; O'Connor 2022 (problems 7-13 need nontrivial steps); Su 2021 (deep scenarios are the top blocker) | Moderate-strong |
| D3 | State abstraction (equivalence of GUI states) determines model quality, coverage and bug finding; no single granularity is best | Mesbah 2012; Baek 2016; Yandrapally 2020; Gu 2019 [reused]; Su 2021 (Sec. 4.3); Dong 2020; Amalfitano 2015 | Strong |
| D4 | The oracle, not input generation, is the weak point of GUI testing; most automated tools only detect crashes | Banerjee 2013; Kong 2019; Nguyen 2014; Su 2021; Barr 2015 [reused]; Choudhary 2015 [reused] | Strong |
| D5 | Generic interactive-system properties (feedback, mode visibility, consistency, reversibility, reachability) can be formalised once and instantiated per device; doing so finds real defects in safety-critical UIs | Harrison 2019 (138 theorems, infusion pump); Bolton 2013 (review: syringe pump status-line, autopilots); Degani 2002 (error/blocking states, autopilot); Rushby 2002 (MD-88 kill-the-capture in 0.24 s); Thimbleby 2010 (silent keystroke handling) | Strong qualitatively; no effect sizes (case studies) |
| D6 | Temporal requirements cluster into a small pattern set; real-time requirements into five bounded patterns | Dwyer 1999 (92% of 555 specs; response + universality + absence = 80%); Konrad 2005 (80 requirements, 9 industrial docs) | Strong for reactive/embedded systems; UI-specific frequency data absent |
| D7 | Runtime monitors must use finite-trace semantics with bounded operators to give definite verdicts; bounded MTL and past operators are monitorable in constant space | Bauer 2011; Ho 2014; Basin 2015; O'Connor 2022 | Strong (theory) |

### 1.2 Secondary / contested positions

- **Random vs systematic exploration.** Monkey was the overall winner on 60+ apps (Choudhary 2015 [reused]) and found 22 of 52 real bugs vs Ape's 24 (Su 2021, Table 4); but systematic/state-based tools beat random on other benchmarks (Amalfitano 2012; Dong 2020: 1.73x unique crashes vs Monkey; Su 2017 Stoat [reused]). The disagreement is largely about *device* testing under time budgets, where reset/replay costs dominate (Choi 2017) -- it does not transfer to a pure reactor where systematic exploration is nearly free [our inference].
- **Learned guidance.** Deep-learning and RL explorers (Humanoid 18, Q-testing 10 bugs) did not beat simpler tools (Su 2021, Table 4); Humanoid's model did not drive coverage (Li 2019 [reused]). Weak evidence for any role of learned exploration in a deterministic gate.
- **Formal verification vs simulation.** HMI formal verification scales worse than simulation (Bolton 2013) and needs a hand-built model; Harrison 2019's proofs took up to 79 min each. For the kiosk this tension dissolves because the *implementation* reactor is the model and its reachable abstract state space is small [our inference].
- **LTL-based acceptance testing of UIs** rests on a single tool evaluation by its authors (O'Connor 2022). Promising, not yet replicated.
- **Statement coverage vs GUI coverage.** Strecker 2008 finds statement-level detectability is also a significant predictor (odds x2.12 per +10%), so code coverage is not useless -- just insufficient alone; consistent with Inozemtseva 2014 [reused] that coverage correlates only weakly with effectiveness once size is controlled.

### 1.3 Bias risks

- **Tool-author evaluation:** GUITAR, Crawljax, AndroidRipper/MobiGUITAR, TimeMachine, Quickstrom, Harrison's templates were evaluated by their authors on chosen subjects. Themis (Su 2021) is the only independent ground-truth benchmark here, and it only measures crashes.
- **Crash-only oracles:** most exploration evidence counts crashes; the kiosk's defects are non-crash. Transfer of "tool X finds more bugs" to visual/functional defects is unproven [our inference].
- **Android/web/desktop-Java skew;** no embedded/LVGL subject anywhere in this corpus.
- **HMI case-study selection bias:** formal HMI papers report the defects found, not the false-alarm rate or effort-adjusted yield; no controlled comparison with conventional testing.
- **Old subjects:** coverage-criteria evidence is 2001-2011 (Memon, Strecker, Yuan); effect sizes come from 2 small Java apps.
- **Same-agent tautology risk (our setting):** if one AI agent writes both the reactor and its properties, templates can be instantiated to match the bug. Mitigation in Section 3/7 [our inference; cf. R5 oracle independence in `event-reactor-prior-art.md`].

---

## 2. Technique catalogue

Inputs legend: **R** = pure reactor + event catalogue; **S** = scene graph per step; **T** = event trace with virtual timestamps; **O** = reference (Rust oracle) app; **A** = accessibility tree.

| Technique | What it detects | Inputs | Determinism | Evidence (numbers) | Cost | Kiosk use |
|---|---|---|---|---|---|---|
| **Event-flow graph + event / event-pair / length-n coverage** (Memon 2001; Memon 2007; Yuan 2011) | Untested event interactions; adequacy measure for sequence suites | R (enabled events per state) | Deterministic | WordPad clone: 362 events -> 21,659 length<=3 sequences; 19.3% infeasible from over-approximate EFG; 30 h unattended (Memon 2001) | Sequences grow ~|E|^n; cheap on a reactor | Coverage report + gate: 100% (state,event) pairs; triples nightly |
| **Invocation-termination coverage** (Memon 2001) | Dialogs that cannot be (or are wrongly) closed | R | Det. | Defined + subsumption proof; no fault data | Tiny | Every dialog x every terminator (timeout, button, tap-outside) |
| **Faulty/illegal event pairs (complement graph)** (Belli 2001) | Responses to events that should be ignored/blocked; recovery defects | R + declared expectations | Det. | Nokia 6610 SMS: 5-12 faults per sub-model (Table 1/2) | Small: |states| x |events| | Taps on non-interactive points, under scrims, on disabled keys -> declared expected outcome (F3, F4) |
| **GUI ripping / crawling with state-flow graph** (Memon 2003; Mesbah 2012; Amalfitano 2012/2015; Nguyen 2014) | Model of reachable screens; crashes; missing windows | Black-box GUI | Mostly det. | Ripping 5-89 s/app, 5-10 min manual fix (Memon 2003); GUITAR pair coverage: 4,468 / 15,562 tests, 309 / 1,204 machine-hours, 7 crashes, 14 false positives (Nguyen 2014); MobiGUITAR 7,711 tests, 10 bugs | Expensive on real toolkits | Replace by exact reactor BFS; keep "rip both apps and diff screen inventories" for F10/F11 |
| **State-equivalence functions** (Mesbah 2012; Baek 2016; Yandrapally 2020; Dong 2020) | Determines dedup, coverage, explosion | S / A / pixels | Det. given threshold | Universal threshold crawl F1 0.24; RTED 0.95 -> 0.45 over crawl; visual PDiff best on unseen apps (Yandrapally 2020); content-level comparison explodes on clocks (Baek 2016) | Cheap per comparison | Use declared projection of reactor view state, exact hash; no thresholds |
| **Snapshot/state-population search** (Dong 2020; Choi 2017) | Deep states missed by sequence mutation | Restorable states | Mixed | 1.15x coverage, 1.73x crashes vs Monkey (68 apps); 137 crashes top-100 apps (Dong 2020); DetReduce 16.9x smaller suites, 14.7x faster, all exceptions kept (Choi 2017) | VM snapshots on Android; free on a reactor | BFS frontier = snapshot pool; emit BFS spanning tree as regression corpus |
| **Explicit-state model checking of UI/automation vs expected model** (Rushby 2002; Degani 2002) | Mode confusion, automation surprise, invisible/automatic transitions, error and blocking states | R + expected (mental/oracle) model | Det. | Kill-the-capture found in 0.24 s (Rushby 2002); autopilot case (Degani 2002) | Very cheap for small models | Lockstep BFS of C reactor vs Rust oracle; invariants on projections |
| **HMI property templates** (Harrison 2019; Bolton 2013) | Missing feedback, invisible mode, inconsistent actions, irreversibility, buried features | R + filter (state projection) + p_filter (scene projection) | Det. | 138 theorems on infusion pump, failures exposed design issues; syringe-pump status-line counterexample (Bolton 2013) | PVS proofs 1 s-79 min; on a finite reactor: enumeration | Core of the property catalogue (Section 4) |
| **LTL acceptance testing on UI traces** (O'Connor 2022) | Sequence-dependent functional bugs, transient states, missing elements | T + S projections | Verdict det.; generation random | 20/43 implementations faulty; 14 problem classes; bound 100 -> ~42 s/passing impl.; accuracy log in bound, time linear | Spec ~300 lines | Template for the trace monitor: state-machine spec + invariants, but with exhaustive generation and virtual time |
| **LTL3 / RV-LTL monitors** (Bauer 2011) | Earliest violation of temporal properties on finite prefixes | T | Det. | 97 pattern formulas: monitors mostly < 100 states; 43/97 never conclusive | Monitor synthesis cheap | Spec linter: reject unbounded liveness |
| **Online MTL / MFOTL monitoring** (Ho 2014; Basin 2015) | Timing violations, parametric over objects | T with timestamps | Det. | Trace-length-independent full MTL under bounded variability (Ho 2014); MonPoly-Fin orders of magnitude faster than regular-relation variant (Basin 2015) | Low for short finite traces | Offline evaluation of bounded MTL over each gate trace, parametric over node ids |
| **Specification patterns** (Dwyer 1999; Konrad 2005) | Mis-specified properties; gives vetted templates | — | Det. | 92% of 555 specs match patterns; 80 industrial RT requirements -> 5 RT patterns | Zero runtime cost | Property DSL keyed by pattern names; agents instantiate, cannot invent raw LTL |
| **Number-entry error blocking analysis** (Thimbleby 2010) | Silently ignored keystrokes, silent resets, ambiguous displays | R + S | Det. | Out-by-10 error probability roughly halved by blocking design (model) | Cheap | Keypad feedback properties (F2) |
| **Statistical usage-model testing / swarm** (Whittaker 1994; Prowell 1999; Groce 2012 [reused]) | Field-weighted failures; feature masking | R + usage model | Det. if seeded | +42% distinct crash causes (Groce 2012) | Cheap | Beyond the exhaustive bound, nightly |
| **Trace validation against a spec** (Cirstea 2024 [reused]) | Implementation/spec divergence | T + spec | Det. | Discrepancies in all case studies | Model-checking cost | Validate recorded C traces against Rust-derived transition relation |

---

## 3. Recommendation: bounded exhaustive exploration of the pure reactor

Grounding: Rushby 2002 (explicit-state search finds mode surprises instantly), Memon 2001/Strecker 2008 (event-based coverage, triples matter), Belli 2001 (illegal pairs), Dong 2020/Choi 2017 (states not sequences are the unit; free resets favour BFS), Baek 2016/Yandrapally 2020/Mesbah 2012 (abstraction), O'Connor 2022 / Bauer 2011 / Konrad 2005 (bounded temporal properties). Everything below beyond those citations is **[our inference]**.

### 3.1 Event catalogue (owner-sealed, versioned)
- **User events derived from the scene:** for every node with an intent in the current scene: `tap(intent)`. Plus **probe taps** at geometric sample points: centre and 4 inset corners of every interactive node's visible rect, and a coarse grid (e.g. 8x6) over the whole screen. Probe taps go through the real hit-testing path (scene/LVGL hit test), not straight to intents, so scrims and overlays that swallow taps (F3) are exercised. Expected outcome of each probe is computed from the scene: topmost interactive node at the point, or "no-op" (Belli 2001 complement-graph idea).
- **Parameterised macro events** to keep depth low: `enter_code(class)` with classes {empty, 1 digit, max-1, max, max+1, valid, invalid, expired}; `staff_login(valid|invalid)`; per-digit events only from boundary states so F2's per-keystroke feedback is still checked (Su 2021 shows deep paths otherwise exceed budgets; ~80k traces for one 8-page bug).
- **Environment events:** server/locker-controller replies {ok, reject(reason), timeout, error}, door sensor changes, language toggle (an ordinary event; Su 2021 found system-language changes are a blind spot of explorers).
- **Time events (virtual clock):** the reactor never reads a clock; time arrives as `tick(dt)`. The explorer offers, in every state with pending deadlines: `fire_next_deadline` (advance exactly to the earliest pending timer), `advance(eps)` (e.g. 1 frame = 16 ms and 100 ms), and `advance(deadline - eps)` (just before expiry). This is what makes minimum-dwell (F1) and bounded-response (F2) properties checkable without flakiness; real-time waiting is what made Quickstrom slow and rare faults flaky (O'Connor 2022, Sec. 4.3).

### 3.2 Abstraction key (deduplication)
- Key = canonical hash of an owner-declared **projection of reactor view state**, not of pixels or scene text: `screen_id`, dialog/overlay stack (ids + kind), `language`, selected state of every toggle group, entered-code *length and validity class* (not digits), staff mode, locker-assignment class, presence (not value) of each pending timer bucketed {none, pending, < 1 frame}, countdown bucket {running, zero} (not seconds).
- Rule: **every variable referenced by any property or oracle projection must be in the key** -- otherwise the property is checked in only one representative of a merged class (MobiGUITAR's id/type-only key would merge EN-selected and BG-selected; Dong 2020 drops text-box values, which would hide F2).
- Keep the **full scene** (and full view state) at each visited step for property checking and oracle comparison; abstraction only decides whether to *expand* a state again. Exclusions from the key (clock text, countdown seconds) must be declared in a reviewed list, analogous to Crawljax's ignore regions (Mesbah 2012), because every exclusion is a potential blind spot.
- Do not use edit-distance/visual-similarity thresholds (Yandrapally 2020; Baek 2016).
- Symmetry: digits and locker numbers are equivalent up to boundary classes; treat them as data classes, as above.

### 3.3 Search
- BFS from the initial state (reset is free; BFS yields shortest counterexamples, which agents can act on and which double as regression fixtures -- Choi 2017's reduction result suggests the BFS spanning tree is a compact regression corpus).
- **Lockstep differential mode:** run the same event on the C reactor and the Rust oracle at every step; compare declared projections of the two scenes (screen inventory, required nodes, texts by msgid, selection states, emphasis classes); a divergence is a failure with its shortest path (Rushby 2002 pattern with the oracle as the "expected model").
- **Bounds:** expand until fixpoint of abstract states or depth D. Start with D = 12 user-level events (macros count as one); report the number of new abstract states found at each depth -- if depth D still adds states, raise D or justify. Evidence for non-trivial depth: AWS found protocol bugs needing 35-step traces (Newcombe 2015 [reused]); Themis' deep bugs span ~8 distinct page transitions (Su 2021). The "small scope hypothesis" (most bugs show up in small scopes; Jackson, Alloy) is [secondary, not downloaded].
- **Coverage gates:** (a) 100% of reachable (abstract state, enabled catalogue event) pairs including probe taps -- equivalent to Prowell's arc coverage and Memon's event-pair coverage on an exact model; (b) every dialog x every terminator (invocation-termination, Memon 2001); (c) nightly: event-triple coverage restricted to triples ending in a time event or dialog event (Strecker 2008); (d) fail the gate if a catalogue event or screen in the oracle's inventory is never reached.
- **Scaling evidence and estimate:** real-GUI pair coverage costs 309-1,204 machine-hours (Nguyen 2014) -- the reason to run at the reactor/scene level. A kiosk of ~10^2-10^4 abstract states x ~30-80 catalogue events (incl. probes) gives 10^4-10^6 transitions; at 0.1-1 ms per step for reactor + scene build + property checks this is seconds to minutes on one core, trivially parallel by frontier partitioning [our inference; measure].
- **State explosion controls if needed:** tighten key (drop data, keep classes), swarm-style per-run event subsets for deeper random walks beyond D (Groce 2012 [reused]), and partial-order reduction for commuting independent events (e.g. two timers) [our inference].
- **Rendering cross-check, sampled:** replay the shortest path to each abstract state through lvrender + software renderer headlessly and check that the accessibility tree equals the scene projection; pixel-level checks belong to other sub-topics.

### 3.4 Trace record per step
`(t_virtual, event, view_state_projection, scene_graph, a11y_tree, oracle_scene_projection)` -- the input to every property in Section 4.

---

## 4. Generic temporal / HMI property catalogue

Notation. A trace is `tau = (t_i, e_i, sigma_i, S_i)`, i = 0..n: virtual time, event that produced step i, reactor view state, scene after the event. Helper predicates over a scene S (computed from the scene graph; other sub-topics define the geometric parts):
`present(n,S)`; `vis(n,S)` (present, non-empty rect intersected with screen and container clip, not fully covered by higher-z opaque node); `hit(p,S)` = topmost node whose hit area contains point p; `enabled(n,S)`; `obs(S)` = owner-declared observable projection (visible texts by msgid+args, visible node set, selection/emphasis classes); `filter_k(sigma)` / `pfilter_k(S)` = paired projections of the same user-relevant quantity k from state and from scene (Harrison 2019).
Formulas are in bounded MTL with past (`G` always, `F_[a,b]` eventually within, `H` historically/past-always, `O` once-in-past); all are **safety** properties, monitorable with definite verdicts on finite traces (Bauer 2011; Ho 2014). Thresholds are owner-set constants; 100 ms / 1 s / 10 s response limits are HCI rules of thumb **[practitioner: Card, Moran & Newell; Nielsen response-time limits]**.

| ID | Property | Formula (sketch) | Source pattern | Serves |
|---|---|---|---|---|
| **P1 Action feedback (bounded response)** | Every tap on an enabled interactive node whose action changes user-relevant state yields a visible change within T_fb (e.g. 100 ms) | `G( e_i = tap(n) and enabled(n,S_{i-1}) and filter(sigma_i) != filter(sigma_{i-1}) -> F_[0,T_fb] (pfilter(S_j) != pfilter(S_{i-1})) )` | Harrison 2019 action feedback (eq. 3); Konrad 2005 bounded response; Dwyer 1999 response | F2, F4 |
| **P2 No dead interactive element (no-effect action)** | A tap on an enabled interactive node must change sigma or obs(S), unless the (state, intent) pair is in the owner's declared idempotent list | `G( e_i = tap(n) and enabled(n,S_{i-1}) -> (sigma_i != sigma_{i-1} or obs(S_i) != obs(S_{i-1}) or (state_class, n) in Idem) )` | Belli 2001 (declared response for every pair); Thimbleby 2010 (no silent keystrokes); Harrison 2019 | F4, F2 |
| **P3 State feedback / distinguishability** | Two reachable states that differ in a user-relevant quantity render differently | `for all reachable sigma, sigma': filter_k(sigma) != filter_k(sigma') -> pfilter_k(S(sigma)) != pfilter_k(S(sigma'))` (check pairwise within each screen class) | Harrison 2019 state feedback (eq. 2); Degani 2002 error states | F2 (digit count), F9 |
| **P4 Mode / selection visibility (iff)** | The displayed mode indicator is true exactly when the mode holds | `G( lang(sigma_i) = m <-> selected_visual(toggle_m, S_i) )` for each m; generally `G(filter_k(sigma_i) = pfilter_k(S_i))` | Bolton 2013 syringe-pump formula; Harrison 2019 visibility (eq. 7) | F9 |
| **P5 Minimum dwell of modal/notification dialogs (bounded invariance)** | A dialog d, once shown, stays visible for at least T_min (8 s for rejection) unless an allowed dismiss event for d occurs | `G( (vis(d,S_i) and not vis(d,S_{i-1})) -> G_[0,T_min) (vis(d,S_j) or O_(t_i, t_j] e in Dismiss(d)) )`; past form: `G( not vis(d,S_j) and O_[0,T_min) appear(d) -> O_(appear, now] e in Dismiss(d) )` | Konrad 2005 minimum duration / bounded invariance; Ho 2014 (past MTL); Quickstrom problem 14 (transient states) | F1 |
| **P6 Maximum duration / auto-dismiss** | A transient dialog/banner disappears by T_max | `G( appear(d) -> F_[0,T_max] not vis(d) )` | Konrad 2005 maximum duration | F1 (dual), F3 (banner overlay persisting) |
| **P7 No invisible automatic transition (blocking-state freedom)** | Every timer/environment event that changes user-relevant state changes obs(S) | `G( e_i in AutoEvents and filter(sigma_i) != filter(sigma_{i-1}) -> obs(S_i) != obs(S_{i-1}) )` | Degani 2002 blocking states; Rushby 2002; Thimbleby 2010 silent reset | F1, F2 |
| **P8 No flicker / minimum perceivable presence** | Any element that appears stays at least T_perc (e.g. 200 ms) unless replaced by a declared successor | `G( appear(n) -> G_[0,T_perc) (vis(n) or O e in Dismiss(n)) )` | Generalises P5; O'Connor 2022 problem 14 [our inference for threshold] | F1 |
| **P9 Tap reaches the visually topmost target (no swallowed tap)** | For every visible interactive node n and probe point p in its visible rect, `hit(p,S) = n`, unless a declared modal scrim is shown; for non-modal overlays, taps outside the overlay reach the underlying node on the *first* tap | `G( for all n in vis_interactive(S_i), p in probe(n): hit(p,S_i) = n or modal_active(S_i) )` and `G( e_i = probe(p) and not modal_active(S_{i-1}) -> effect(e_i) = effect(tap(hit(p,S_{i-1}))) )` | Belli 2001 faulty pairs; [our inference]; geometry from other sub-topics | F3 |
| **P10 Reversibility** | Designated action pairs undo each other on the relevant projection | `for all reachable sigma: guard(sigma) -> filter(b(a(sigma))) = filter(sigma)` (toggle EN/BG; digit/backspace; open/close dialog) | Harrison 2019 reversibility (eq. 6) | F4, F9 |
| **P11 Completeness / reachability** | Home/cancel reachable from every screen within k events; every screen in the oracle inventory reachable | `for all reachable sigma: exists path len <= k to Home`; `Screens(oracle) subset Screens(C)`; `AG(X -> EF Y)` per required pair | Harrison 2019 completeness (eq. 1); Bolton 2013 reachability | F11, F10 |
| **P12 Required-content invariants per screen** | Each screen contains its required nodes (e.g., splash version line, report sections), derived from the oracle's scene for the same state | `G( screen(sigma_i) = x -> Required(x) subset present(S_i) )`; differential: `obs_k(S_C(path)) = obs_k(S_Rust(path))` | O'Connor 2022 (invariants that UI elements exist); Rushby 2002 (expected model) | F10, F11, F7, F5 (if emphasis class is in obs) |
| **P13 Action/state consistency** | Same intent has same effect, position and style class across screens; language toggle changes *all* visible msgids | `G( e = tap(intent) -> delta(filter) in Effect(intent) )`; `G( e_i = setLang(L) -> F_[0,T_fb] for all text nodes: locale(S_j,n) = L )` | Harrison 2019 action/state consistency, universality | F4, F8, F7 |
| **P14 Bounded recurrence of live values** | While a countdown is visible, its text updates at least every 1 s and matches the oracle format | `G( vis(countdown,S_i) -> F_[0,1s] changed(countdown) )` + format equality with oracle | Konrad 2005 bounded recurrence | F6 (cadence/format part) |
| **P15 Precedence / authority** | Locker opens only after a valid code is visibly confirmed | `G( e = open_locker -> O (vis(confirm)) )` | Dwyer 1999 precedence; Leveson authority-limit category via Bolton 2013 | safety, F2 |

Implementation notes [our inference]: evaluate all properties offline over each explored path (traces are short and finite), parametric over node ids as in MFOTL (Basin 2015) so violations name the node; reject unbounded `F`/`G` without intervals in the spec linter (Bauer 2011); store properties as instances of named patterns (Dwyer 1999; Konrad 2005) in an owner-sealed file; each property gets mutation tests (a seeded reactor/scene mutant it must kill) so tautological instantiations are caught.

---

## 5. Mapping F1-F11

| F | Defect | Exploration needed | Primary properties | Detectable deterministically here? | Key sources |
|---|---|---|---|---|---|
| F1 | Rejection dialog removed ~60 ms after creation (should stay 8 s) | Reach reject path; time events `advance(eps)`, `advance(T-eps)` | P5, P8, P7; invocation-termination coverage; oracle differential at t+eps | **Yes** (virtual time) | Konrad 2005; Ho 2014; O'Connor 2022; Memon 2001; Degani 2002 |
| F2 | No visible feedback while typing code | Keypad states at boundary lengths; per-digit events | P1, P3 (digit count from scene), P2, P7 | **Yes** | Harrison 2019; Thimbleby 2010; Degani 2002 |
| F3 | Banner overlay covers key; scrim swallows first tap | Probe taps in all overlay states | P9, P6; Belli complement pairs | **Yes** at scene level (geometry/hit-test from other sub-topics) | Belli 2001 [+ sub-topic A/B geometry] |
| F4 | Language toggle does nothing | Tap every intent in every state | P2, P1, P13, P10 | **Yes** | Belli 2001; Harrison 2019 |
| F5 | Locker number tiny/split glyphs | Reach assignment screen | P12 with emphasis class in obs; differential vs oracle | Partly (needs emphasis/hierarchy metric from other sub-topics) | other sub-topics |
| F6 | Countdown clipped; format differs | Reach countdown states, time events | P14 (cadence + format vs oracle); clipping is geometric | Format/cadence yes; clipping via geometry sub-topic | Konrad 2005 |
| F7 | One font size, left-aligned | Any screen | P12/P13 with style class in obs; oracle differential | Yes if style ids are in obs | Memon 2003 (property-level ripping diff) |
| F8 | No wrapping; BG truncated | Language as a key dimension; all screens x {EN, BG} | P13 + text-fit predicates from other sub-topics | Exploration side yes; fit check elsewhere | Su 2021 (language is a blind spot) |
| F9 | Toggles show no selected state | All toggle states in key | P4, P3, P10 | **Yes** | Bolton 2013; Harrison 2019; Degani 2002 |
| F10 | Splash missing version line | Initial state | P12 (required nodes from oracle) | **Yes** | O'Connor 2022; Memon 2003 |
| F11 | Staff report missing sections | Staff macro events to reach report | P11, P12 differential; screen-inventory coverage | **Yes** | Harrison 2019; Bolton 2013; Memon 2003 |

---

## 6. Hard-to-catch list (what this sub-topic's techniques will still miss)

1. **Perceptual adequacy of feedback.** Templates check that a change is *present in the scene*, not that it is noticeable (size, contrast, location near the finger). Harrison 2019 models perceivability abstractly. Needs geometric/salience metrics (other sub-topics) [our inference].
2. **Legitimately inert actions.** P2 needs an owner-declared idempotence list (tapping the already-selected language). An agent can "fix" a dead button by adding it to the list -- the list must be sealed/owner-reviewed [our inference].
3. **Abstraction-hidden bugs.** Anything depending on a variable left out of the key is checked in one representative only (Dong 2020; Amalfitano 2015). The key must be validated by mutation: seed bugs that depend on each excluded field and check whether the gate notices [our inference].
4. **Real latency vs virtual time.** P1/P5 hold in virtual time; a real renderer stall, LVGL invalidation bug or X11 frame drop is not seen. Needs a sampled real-time replay with frame timestamps (lui-perf corpus) [our inference].
5. **Bugs deeper than D or requiring rare data.** Exhaustion is bounded; Su 2021 shows deep-scenario and specific-input bugs evade tools; macro events mitigate but are owner-designed.
6. **Oracle-shared defects.** If the Rust oracle has the same defect (or the projection omits the attribute), differential checks pass (Barr 2015 [reused]).
7. **Specification ambiguity.** Formalising exposes cases the English spec leaves open (O'Connor 2022: TodoMVC filter state); somebody must decide them -- not the implementing agent.
8. **Semantic wrongness of visible text** (right node, wrong message for the reason) unless msgid+args are in obs and compared with the oracle.
9. **Mode confusion relative to real users' mental models** -- Degani/Rushby need a postulated user model; generic invariants cover only the generic part.
10. **Concurrency outside the reactor** (renderer thread, TCP a11y server serving a stale tree) -- reactor purity does not cover it.
11. **Unbounded liveness** ("the kiosk eventually returns to idle") is not monitorable without a bound (Bauer 2011); must be restated with timeouts.

---

## 7. Evidence gaps

- **No peer-reviewed study of exhaustive/bounded exploration of Elm/Redux-style pure UI reducers.** Only practitioner material (Imandra "Verified React" blog; TLA+ models of Redux) **[practitioner]**; closest academic analogue is explicit-state checking of automation models (Rushby 2002) and the kiosk design inherits its argument by analogy.
- **No empirical data on the defect-detection yield of HMI templates** (feedback, visibility, reversibility) versus conventional tests; all evidence is case studies (Harrison 2019; Bolton 2013).
- **No UI-specific frequency study of temporal property patterns** analogous to Dwyer 1999's 555-spec survey; whether minimum-dwell and bounded-response dominate UI requirements is our inference.
- **Coverage-criteria effect sizes are old and small** (2 Java apps, Strecker 2008); no data for event-pair/triple coverage on embedded touch UIs.
- **Quickstrom is a single author-run evaluation**; no replication, no comparison against exhaustive generation.
- **Virtual-time vs wall-clock validity** of UI temporal properties is not studied: no paper quantifies how many timing defects are visible in virtual time vs only on real renderers.
- **Abstraction-key validation:** no method in the corpus for proving that a declared state projection preserves all properties of interest (beyond "include every variable the property reads") -- a natural target for a property-based check [our inference].
- **Response-time thresholds** (100 ms feedback) come from HCI practice, not from this corpus **[practitioner]**.
- **Combined geometry + temporal logic** (e.g., "the key under the banner is hittable within 1 tap at all times") has no direct literature; it sits between this sub-topic and the occlusion/layout sub-topics.

---

## 8. Sources

### 8.1 Reused corpus (read summaries, not re-downloaded)
- `~/Downloads/research_papers/event-reactor-testing/summaries/`: `su2017_stoat.md`, `gu2019_ape.md`, `mao2016_sapienz.md`, `alshahwan2018_sapienz_fb.md`, `choudhary2015_android_tools.md`, `wang2018_android_industrial.md`, `li2019_humanoid.md`, `whittaker1993_markov_spec.md`, `whittaker1994_markov_stat_testing.md`, `prowell1999_cleanroom_book.md`, `groce2012_swarm_testing.md`, `claessen2000_quickcheck.md`, `hughes2016_dropbox.md`, `proptest_state_machine_docs.md`, `cirstea2024_tla_trace_validation.md`, `barr2015_oracle_survey.md`, `bornholt2021_s3_lightweight_fm.md`, `zhou2021_foundationdb.md`, `tigerbeetle_vopr_docs.md`
- `~/Downloads/research_papers/lui-usability-metrics/summaries/`: `Memon 2007 - Event-flow model of GUIs for testing.md`, `Yuan 2011 - GUI interaction testing event context.md`
- `~/Downloads/research_papers/event-reactor-formal/summaries/`: `newcombe2015_aws_tla.md` (35-step traces), `heitmeyer1996_scr_consistency.md` (disjointness/coverage of event tables = every (mode, event) handled exactly once)
- `~/Downloads/research_papers/lui-conformance/summaries/Romano 2021 - UI flaky tests.md` (45% of UI flaky tests are async-wait; supports virtual time)
- `~/Downloads/research_papers/rigor-levels-test-adequacy/summaries/`: `inozemtseva2014-coverage-not-correlated.md`, `bohme2020-exponential-cost.md` (random discovery cost grows exponentially -> exhaust the bounded space instead)
- `~/sw_projects/ai-code-rigor/docs/research/event-reactor-prior-art.md` Part 3 (testing tiers; R1-R5 preconditions; arc coverage; oracle independence)

### 8.2 New sources (this sub-topic; full text, summaries in `Articles/`)
1. Memon, Soffa, Pollack (2001). Coverage criteria for GUI testing. ESEC/FSE-9. DOI 10.1145/503209.503244
2. Memon, Banerjee, Nagarajan (2003). GUI ripping: reverse engineering of GUIs for testing. WCRE. DOI 10.1109/WCRE.2003.1287256
3. Strecker, Memon (2008). Relationships between test suites, faults, and fault detection in GUI testing. ICST. DOI 10.1109/ICST.2008.26
4. Banerjee, Nguyen, Garousi, Memon (2013). GUI testing: systematic mapping and repository. IST 55(10). DOI 10.1016/j.infsof.2013.03.004
5. Nguyen, Robbins, Banerjee, Memon (2014). GUITAR. Automated Software Engineering 21(1). DOI 10.1007/s10515-013-0128-9
6. Mesbah, van Deursen, Lenselink (2012). Crawling AJAX-based web applications (Crawljax). ACM TWEB 6(1). DOI 10.1145/2109205.2109208
7. Yandrapally, Stocco, Mesbah (2020). Near-duplicate detection in web app model inference. ICSE. DOI 10.1145/3377811.3380416
8. Baek, Bae (2016). Automated model-based Android GUI testing using multi-level GUI comparison criteria. ASE. DOI 10.1145/2970276.2970313
9. Amalfitano, Fasolino, Tramontana, De Carmine, Memon (2012). Using GUI ripping for automated testing of Android applications. ASE. DOI 10.1145/2351676.2351717
10. Amalfitano, Fasolino, Tramontana, Ta, Memon (2015). MobiGUITAR. IEEE Software 32(5). DOI 10.1109/MS.2014.55
11. Choi (2017). Guided GUI testing of Android apps with minimal restart and approximate learning (SwiftHand + DetReduce), PhD thesis, UC Berkeley. (SwiftHand OOPSLA 2013: DOI 10.1145/2509136.2509552)
12. Dong, Boehme, Cojocaru, Roychoudhury (2020). Time-travel testing of Android apps. ICSE. DOI 10.1145/3377811.3380402
13. Su, Wang, Su (2021). Benchmarking automated GUI testing for Android against real-world bugs (Themis). ESEC/FSE. DOI 10.1145/3468264.3468620
14. Kong, Li, Gao, Liu, Bissyande, Klein (2019). Automated testing of Android apps: a systematic literature review. IEEE Trans. Reliability 68(1). DOI 10.1109/TR.2018.2865733
15. Belli (2001). Finite-state testing and analysis of graphical user interfaces. ISSRE. DOI 10.1109/ISSRE.2001.989456
16. O'Connor, Wickstrom (2022). Quickstrom: property-based acceptance testing with LTL specifications. PLDI. DOI 10.1145/3519939.3523728
17. Bauer, Leucker, Schallhart (2011). Runtime verification for LTL and TLTL. ACM TOSEM 20(4). DOI 10.1145/2000799.2000800
18. Ho, Ouaknine, Worrell (2014). Online monitoring of metric temporal logic. RV 2014, LNCS 8734. DOI 10.1007/978-3-319-11164-3_15
19. Basin, Klaedtke, Mueller, Zalinescu (2015). Monitoring metric first-order temporal properties. JACM 62(2). DOI 10.1145/2699444
20. Dwyer, Avrunin, Corbett (1999). Patterns in property specifications for finite-state verification. ICSE. DOI 10.1145/302405.302672
21. Konrad, Cheng (2005). Real-time specification patterns. ICSE. DOI 10.1145/1062455.1062526
22. Harrison, Masci, Campos (2019). Verification templates for the analysis of user interface software design. IEEE TSE. DOI 10.1109/TSE.2018.2804939
23. Degani, Heymann (2002). Formal verification of human-automation interaction. Human Factors 44(1). DOI 10.1518/0018720024494838 (author preprint)
24. Rushby (2002). Using model checking to help discover mode confusions and other automation surprises. Reliability Eng. & System Safety 75(2):167-177. DOI 10.1016/S0951-8320(02)00022-4 (author preprint)
25. Bolton, Bass, Siminiceanu (2013). Using formal verification to evaluate human-automation interaction: a review. IEEE Trans. SMC: Systems 43(3):488-503. DOI 10.1109/TSMCA.2012.2210406
26. Thimbleby, Cairns (2010). Reducing number entry errors: solving a widespread, serious problem. J. R. Soc. Interface 7:1429-1439. DOI 10.1098/rsif.2010.0112

### 8.3 Cited but not obtained (abstract/secondary only; no summary written)
- Masci, Zhang, Jones, Curzon, Thimbleby (2014). Formal verification of medical device user interfaces using PVS. FASE, LNCS 8411. DOI 10.1007/978-3-642-54804-8_14 -- PVS analysis of a real infusion pump's C++ UI code found interaction design issues (abstract only).
- Navarre, Palanque, Ladry, Barboni (2009). ICOs: a model-based UI description technique (Petri-net based, cockpit UIs). ACM TOCHI 16(4). DOI 10.1145/1614390.1614393 (not obtained).
- Campos, Harrison (2008). Systematic analysis of control panel interfaces using formal tools. DSV-IS, LNCS 5136. DOI 10.1007/978-3-540-70569-7_6 (not obtained; its reachability/feedback results are summarised via Bolton 2013 and Harrison 2019).
- Jackson's small-scope hypothesis / Andoni et al. 2002-2003 evaluation (not found as open PDF) -- cited as [secondary].

---

## 9. Exhaustion report

- **Searches:** research-hub DOI/title searches (~10 queries), WebSearch (Themis PDF, Rushby, Degani, ICO, Masci, small-scope hypothesis, Elm/Redux model checking). Anna's Archive unavailable (session expired, DDoS-Guard 403; `annas login` not attempted per rules).
- **Downloaded and summarised:** 26 sources, ~31 MB of PDFs (budget 400 MB).
- **Download failures / corrections:** Themis via publisher failed -> author copy (tingsu.github.io); ICO (Navarre 2009) and Campos & Harrison 2008 not obtainable; Masci 2014 FASE blocked (Springer/core.ac.uk returned HTML); Rushby publisher PDF had unextractable fonts -> SRI author preprint; the DOI first used for Degani 2002 returned a different Human Factors article (variable-resolution displays) -> replaced by the correct author preprint (DOI 10.1518/0018720024494838). SwiftHand OOPSLA paper replaced by the author's 2017 thesis containing it.
- **Saturation:** core GUI-coverage lineage (Memon/Yuan/Strecker), state-abstraction lineage (Crawljax/Baek/APE/Yandrapally/Themis), RV lineage (Bauer/Ho/Basin), pattern lineage (Dwyer/Konrad) and HMI lineage (Degani/Rushby/Harrison/Bolton/Thimbleby) are each covered by their canonical papers; further papers in these lines would mostly be tool variants. Not covered: statecharts verification tools, Leveson 1997 mode-confusion paper (covered via Bolton 2013 Table III), ARINC 661 cockpit formal models (ICO not obtained), and any academic work on exhaustive exploration of Elm/Redux reducers (none found).
