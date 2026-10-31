# Event-reactor architecture: prior art, formal foundations and testing

Status: research synthesis, 31 October 2026. It merges three sub-topic reports written for this project: practical prior art, formal foundations, and fast human-like testing.

How to read this document:

- Citations are inline as (Author Year). Full entries are in [references.md](references.md), §22. Framework and product documentation is cited by project name and year, for example (LiveStore docs 2026).
- **[unverified]** marks a claim the research notes flag as not re-checked or cited from memory. **[secondary]** marks a figure taken from a source that reports another source. **[inference]** marks reasoning in this document that no source measured.
- Findings are paraphrased. Quotes are under 25 words.
- Related design note: [functional core and GPU-accelerated validation](../architecture/functional-core.md).

**The proposed design under review.** Events are the only I/O. Small adapters sit at the edge, and each comes with a simulation of its exact external API. Reactors are pure functions from events plus selectors to events. State is an append-only event log plus memoized selectors. Reactor-to-event wiring is declared statically. The UI is one more adapter: each screen is a selector tree that produces a view model. The target is mobile-first Rust with a local store on SQLite or Turso, reactor rules written as decision tables with SMT obligations, and sync across several devices.

For the fit matrix the design is split into six elements:

| Code | Element |
|---|---|
| **E** | Events are the only input and output |
| **A** | Small adapters, each with a simulation of its exact external API |
| **R** | Reactors are pure: (events, selectors) → events |
| **L** | All state is an append-only log plus memoized selectors |
| **W** | Reactor-to-event wiring is declared statically |
| **U** | The UI is an adapter; each screen is a selector tree → view model |

---

## Summary: the decision-relevant findings

1. **No existing system combines all six elements, but each element has mature precedent.** Crux is closest on E, W and U for mobile Rust. re-frame is closest on pure reactors, selectors and coeffects. LiveStore is closest on an on-device event log. Sans-IO protocol libraries and QP active objects are closest on deterministic-core discipline. The design is roughly Crux + LiveStore + re-frame's signal graph + sans-IO time handling, with adapters raised from "mock" to "exact API simulation".

2. **Reactor outputs must not be stored as truth in a synced log.** Treating the log as a grow-only set makes convergence of *derived* state almost trivial (Shapiro et al. 2011; Baquero et al. 2014). But a reactor that appends events based on the prefix one device has seen can produce different reactions on different devices. Store only external input facts and re-derive reactor outputs, or give outputs deterministic ids and fire non-monotone reactions only on the causally stable prefix (Hellerstein & Alvaro 2020). Side effects that have already left the device need idempotency keys and an outbox.

3. **"Deterministic" must mean bit-identical across platforms and code versions.** Floats, NaN payloads, hash-map iteration order, locale, time zone and Unicode normalisation all break replay equality (Goldberg 1991; Watt 2018). Version skew between devices running different selector code is another source of divergence. Time, randomness, ids and locale should enter as input events, as re-frame's coeffects and quinn-proto's caller-supplied time do.

4. **Acyclic, topologically scheduled evaluation is the right replacement for capped fixpoint iteration, within one step.** Lustre and SCADE have used this for decades in certified systems (Halbwachs et al. 1991; Benveniste et al. 2003). It does not bound cascades *across* steps (A emits, B reacts, A reacts again). That needs an acyclic event-type graph or an SMT-checked ranking function per cycle.

5. **"Monoid selectors" give three different benefits, each with its own algebraic law.** Parallel replay needs associativity (Blelloch 1990). Order-free merge of device logs needs commutativity, or a tree of partial aggregates. Cheap retraction needs inverses, as in DBSP's abelian groups (Budiu et al. 2023). Max, min and "latest" have no inverse. Floats are not associative.

6. **A small verified engine plus a reference interpreter and differential testing concentrates rigor where it pays, but per-app cost does not fall to zero.** CompCert's verified passes survived about 6 CPU-years of Csmith fuzzing with no wrong-code bugs, while the bugs found were in unverified parts (Yang et al. 2011). What stays per app is writing the specification: rules, domain invariants and the unverified rim (serializer, storage, sync transport, FFI, UI bindings).

7. **Make the view-model protocol incremental from day one.** Photoroom's adoption of Crux found that re-serialising the full view model on every render did not scale for a rich editor. They retrofitted typed patches and then automatic before/after diffing (Denis 2026). Selectors that emit deltas, DBSP-style, avoid that retrofit.

8. **Split durable facts from ephemeral signals.** TCA's documented performance wall came from scroll, appear and disappear noise going through one main-thread store (TCA #1290). LiveStore separates `synced` from `clientOnly` events. Only durable facts belong in the persisted log. Each event type should declare its class as part of the static wiring.

9. **For testing, the core recipe is determinism + seeded generation + automated oracles + shrinking, and this design gets determinism almost for free.** Stateful property testing against an executable reference model has the strongest evidence: 200+ issues at Volvo with models 8–18× smaller than the hand-written suites they replaced (Arts et al. 2015), and 16 production issues prevented in Amazon S3 for a 13% code overhead (Bornholt et al. 2021). If every adapter is simulated and reactors are pure, a session is a fold over its log, so replay, shrinking and time travel are cheap.

10. **"Human-like" testing and bug-finding are different objectives.** Usage-profile testing earns its keep for reliability estimates and prioritisation (Musa 1993; Prowell et al. 1999). Diversity finds more bugs: plain random Monkey beat research tools in 2015 (Choudhary et al. 2015), swarm testing found 42% more distinct crash causes (Groce et al. 2012), and biasing toward production traffic gave ShardStore no benefit. LLM GUI agents are good session *generators* and poor *oracles*: freeze their sessions into logs and let invariants and models decide pass or fail. Simulated adapters need contract tests and trace validation against real-API recordings, because simulator fidelity is the blind spot of the whole approach (Zhou et al. 2021; Cirstea et al. 2024).

---

## Part 1 — Prior art

### 1.1 Fit matrix

Legend: ● matches, ◐ partly, ○ absent or different.

| System (version seen) | E | A | R | L | W | U | Testing story | Main documented failure mode |
|---|---|---|---|---|---|---|---|---|
| Crux (crux_core 0.20.0, Aug 2026) | ● | ◐ | ◐ | ○ | ● | ● | Effects resolved in pure Rust tests with no shell; fluent `expect_`/`resolve_` helpers since 0.18 | Full view model per render did not scale (Denis 2026); heavy API churn before 1.0; async runtime advances only when the shell calls in |
| Elm / TEA (0.19.1) | ● | ◐ | ● | ○ | ● | ● | `update` is pure, so tests are function calls | Nested-component boilerplate; awkward child-to-parent messages; slow single-maintainer ecosystem |
| Redux + Reselect (v5.x) | ◐ | ○ | ● | ◐ | ◐ | ● | Reducers and selectors are pure | Size-1 caches thrash; identity equality is fragile; primitive-keyed caches grow without bound |
| re-frame | ● | ● | ● | ◐ | ● | ● | Coeffects fixed to constants; effects stubbed with no-op handlers | No persistence or log; layer-2 subscriptions rerun on every db change |
| Spotify Mobius | ● | ● | ● | ○ | ● | ● | Given/when/then tests of Update and Init | No persistence or selectors; mature but quiet |
| TCA (1.26.2; 2.0 pending) | ● | ● | ● | ○ | ● | ● | Exhaustive TestStore; non-exhaustive mode since Oct 2022 | Exhaustive tests brittle under composition; single main-thread store hits a performance wall |
| Event sourcing / CQRS (server) | ◐ | ◐ | ◐ | ● | ◐ | ○ | Given events, when command, then events | Schema evolution, rebuild time, privacy, learning curve, read-after-write, immature tools (Overeem et al. 2017, 2021) |
| Sans-IO (h11, h2, quinn-proto, rustls) | ● | ● | ● | ○ | ● | n/a | Bytes in, bytes out; 100% branch coverage realistic | Limited to one protocol; timers and buffering semantics designed by hand |
| Hexagonal (Cockburn 2005) | ◐ | ● | ○ | ○ | ◐ | ● | Test harness as primary adapter; in-memory mocks as secondary adapters | Mocks drift from the systems they stand in for |
| QP active objects (Samek 2008) | ● | ◐ | ○ | ○ | ● | ◐ | QUTest trace-based tests on host and target | Responsiveness bounded by the longest run-to-completion step; state lives inside objects |
| LiveStore (docs v0.5.0) | ● | ◐ | ● | ● | ● | ● | Not documented in the pages read | Data must fit in memory (~1 GB); tens to low hundreds of users per log; no compaction story found |
| Replicache → Zero | ◐ | ○ | ◐ | ◐ | ◐ | ● | — | Replicache in maintenance mode; Zero maturity unclear (0.22 seen) |
| ElectricSQL (rewritten Jul 2024) | ○ | ○ | ○ | ◐ | ○ | ◐ | — | Abandoned the general write path; read-sync only |
| PowerSync | ○ | ○ | ○ | ◐ | ○ | ◐ | — | Strictly ordered upload queue causes head-of-line blocking |
| Evolu | ◐ | ○ | ○ | ● | ○ | ◐ | — | Last-write-wins only; HLC timestamps leak metadata |
| Automerge 3 / Eg-walker | ● (ops) | ○ | ● (merge) | ● | ○ | ○ | Property and fuzz tests of convergence | Full history grows; fixed by columnar compression and transient materialisation |
| Turso (pre-1.0) | — | — | — | ◐ (CDC + DBSP views) | — | — | Deterministic simulation testing plus Antithesis | IVM views "not production-ready" (Oct 2025); SQLite compatibility incomplete |
| SQLite | — | — | — | storage | — | — | 100% branch coverage and 100% MC/DC of the core; fault-injecting VFS | WAL grows when long readers block checkpoints; durability depends on `synchronous` |

### 1.2 Key lessons per system

**Crux (Rust core, native shells).** `update(event, &mut model)` returns a Command that carries typed effect requests and follow-up events; `view(&model)` returns a view model, and rendering is itself an effect. FFI types are generated, so a core change breaks shell builds at compile time (Crux docs 2026).
- R is only partial: the model is mutated in place, and Commands can run async orchestration without model access. L is absent: there is no log, and `view` is one non-memoized selector.
- **API churn.** Capabilities were deprecated in 0.16 and removed in 0.17; the FFI layer moved from UniFFI to BoltFFI in 0.19; the typegen backend changed several times (Crux releases 2026). The book says breaking changes are unavoidable before 1.0.
- **View-model scaling.** Photoroom moved through hand-written change enums, typed key-path patches, an open-source patch crate, and finally automatic before/after diffing applied by each shell to a long-lived view model. That costs building the view model twice per event. They also hand-wrote a C-ABI FFI, swapped JSON for MessagePack and forked the type generator (Denis 2026). No published numbers on bridge serialisation cost exist.
- Adopters named by the vendor (Red Badger 2026; Koch 2026) are vendor claims.

**Elm, Mobius, Iced, TCA.** The Elm Architecture pairs a model, a pure `update` returning a model and commands, `view`, and declarative subscriptions; the runtime owns all effects (Czaplicki 2012; Elm guide).
- Nested components need message and command mapping at every level; practitioners cap nesting at about two levels. Elm 0.19.1 dates from about 2019 **[unverified]**.
- Mobius applies the same split in Spotify's Android apps and states no backward-breaking changes are planned: evidence that the pattern holds on mobile at scale (Mobius docs).
- Iced (TEA in Rust) is self-described as experimental and lists no mobile target.
- TCA's exhaustive TestStore asserts every state change and received action and fails on effects still running at the end. Point-Free added a non-exhaustive mode because large composed features made exhaustive integration tests long, dependent on child internals, and fragile (Point-Free 2022). Child actions bubble through every reducer layer on the main thread; a 10k-row list sending appear and disappear events caused hitches on a recent iPhone (TCA #1290). Maintainers advise against a store per cell. Performance regressions in 1.5–1.8 are deferred to 2.0, unshipped as of 1.26.2 (TCA #2846). Exact TCA release years are not in the pages read **[unverified]**.

**Redux and Reselect.** Normalised state (entity tables by id, references by id) is Redux's documented fix for duplication and spurious re-renders, and is what SQLite projections give for free (Redux docs). Reselect's history is a cautionary tale for memoized selectors: the v4 cache of size 1 thrashed when one selector served many list items; identity-equality inputs defeat caching; extra arguments leak into caches; primitive-keyed caches never get collected, and v5.3's `maxSize` must be set on both caches (Reselect docs). **Every memoized selector needs a stated cache key, equality and eviction policy.**

**re-frame.** Event handlers receive coeffects (the db plus injected world values such as current time and storage reads) and return an effects map; effect handlers are the only impure code. Subscriptions form a demand-built, shared, auto-disposed signal graph: cheap layer-2 extractors run on every change and stop propagation when the result equals the previous one; layer-3 views do the heavy work only when inputs change (re-frame docs). Coeffects are the cleanest precedent for making time, randomness, ids and storage reads into inputs, and the layered graph with value-equality pruning is a proven selector-tree design.

**Event sourcing in practice.** An interview study of 19 systems and 25 engineers found five challenges: event schema evolution; a learning curve dominated by eventual consistency (read-after-write, decisions from stale query-side data); immature tooling; slow projection rebuilds; and privacy and GDPR (Overeem et al. 2021). Evolution tactics are versioned events, weak schema, upcasting, in-place transformation and copy-and-transform. Run-time techniques cannot merge or split streams and accumulate conversion code forever; batch techniques let old code be deleted but break strict immutability or cost a full copy (Overeem et al. 2017). A practitioner account adds that event meanings drifted within a month, each projection is another consumer to change, form-style UIs fit task events poorly, replay debugging rarely paid off, and process managers were needed (Kiehl 2019). Crypto-shredding (a key per data subject, deleted on erasure) is the common GDPR answer (Verraes 2019); a practitioner forum reports it may not be considered sufficient in some jurisdictions, and key custody is the real burden **[unverified]**. Three of the 19 systems used snapshots.

**Sans-IO, functional core, ports and adapters.** Sans-IO protocol libraries do no I/O and no async in the core: bytes in, events and bytes out. Claimed benefits are determinism, mock-free tests, realistic 100% branch coverage and reuse across event loops; quinn-proto has the caller supply the time and drive timeouts (Sans-IO guide; quinn-proto docs). Hexagonal architecture asks for an app drivable by tests without a UI or database (Cockburn 2005). "Functional core, imperative shell" is the same idea at app scale (Bernhardt 2012) **[unverified: cited from memory]**. The proposed reactors are sans-IO generalised to the whole app. What the design adds over Cockburn is that the simulation models the *external API's* behaviour, not just the port. That only holds up with contract tests running the same scenarios against simulation and reality, as SQLite's SQL Logic Test does differentially.

**QP active objects.** Each active object is an encapsulated hierarchical state machine with a private queue, sharing nothing and using immutable events from pools. Run-to-completion means each event moves the machine from one stable state to the next in one indivisible step; responsiveness is bounded by the longest step (Samek 2008). QUTest checks traces from a target-side fixture, runs most tests on the host and the same tests on hardware (Quantum Leaps docs). Moving state from actors into a global log plus selectors gains replay, time travel and one consistent snapshot. It loses encapsulation (any reactor could read any selector unless reads are declared), headroom on small targets, and step latency, which now includes selector recomputation and log append and fsync.

**Local-first and log-based sync.** Full character-level CRDT history quickly became a memory and disk problem in early prototypes and could not be truncated, because a peer may return months later (Kleppmann et al. 2019). Automerge 3 keeps the compressed columnar format at runtime (Automerge 2025). Eg-walker keeps the event graph and materialises merge state only transiently, claiming an order of magnitude less memory than CRDTs (Gentle & Kleppmann 2025). LiveStore is the closest match to L: events commit atomically to a local log and are materialised into SQLite; reactive SQL queries drive the UI; sync pulls before pushing, rebases local events and takes a server-set total order with last-write-wins by default; events have versioned names; unknown events from other app versions get a configurable policy (warn, ignore, fail, callback) (LiveStore docs 2026). Other designs: Replicache reconciles rebased mutators on the server and is now in maintenance mode (Replicache docs); PowerSync never resolves conflicts on the client but suffers head-of-line blocking (PowerSync docs); Evolu uses HLC timestamps, last-write-wins and Merkle diffing (Evolu docs); Ditto uses CRDT deltas over a radio mesh (Ditto docs). ElectricSQL narrowed in 2024 from a full local-first platform to read-path-only sync, which is evidence that the general bidirectional write path is the hard part (ElectricSQL 2024). Battery and storage cost figures exist only as vendor statements.

**Turso, SQLite and DBSP.** Turso is pre-1.0; its README says it runs in production at some organisations but tells users to keep independent backups. SQLite compatibility is incomplete. Concurrent transactions (MVCC) and change data capture are supported; encryption, DBSP views and full-text search are experimental. Its live materialised views support join, filter, aggregate and union, have no CTEs and an unstable on-disk format, and are described as not production-ready (Turso 2025). The React Native SDK offers a local database with push/pull sync and offline writes; partial sync and real-time subscriptions are future work (Turso 2026). Turso is tested with its own deterministic simulation suite plus Antithesis, with no published bug statistics. SQLite's test code is about 590 times the library; TH3 gives 100% branch and MC/DC coverage of the core in the default configuration; SQL Logic Test runs about 7.2M differential queries; anomaly tests cover out-of-memory, I/O errors and power loss. In WAL mode, long readers can block checkpoints and let the WAL grow without bound, and `synchronous=NORMAL` can lose the last commits on power loss (SQLite 2026). DBSP is the formal version of "memoized selectors over a log": general, compositional incremental view maintenance with work proportional to the size of the change (Budiu et al. 2023).

### 1.3 Lessons for our design

1. **Incremental view-model protocol from day one.** Selector trees emit typed key-path patches, preferably generated by the selectors themselves (DBSP-style deltas) rather than by snapshot diffing.
2. **Durable facts vs ephemeral signals.** Only durable facts are persisted. Ephemeral UI events go to an in-memory ring. Each event type declares its class (durable, session, ephemeral) in the static wiring.
3. **Store input facts; re-derive reactor outputs.** Sync reorders events, so outputs that are not external commands are recomputed, not persisted as truth. External effects already sent need idempotency keys and an outbox.
4. **Time, randomness, ids and locale are input events.** Reactors never read clocks; simulation adapters own the clocks. This is what makes replay and deterministic simulation possible.
5. **Version events from the first commit.** Upcasters are pure, tested functions; copy-and-transform retires old versions. Mobile fleets run mixed app versions for months, so unknown-event behaviour is defined explicitly.
6. **Projections and selectors update in the same local transaction as the append.** On the device this keeps read-after-write and avoids the eventual-consistency hurdle. Rebuilds are targeted per selector; boot time is budgeted by persisting materialised state and replaying only the tail.
7. **Every memoized selector declares cache key, equality and eviction.** In Rust: `Arc` plus `PartialEq`, or content hashes. DBSP or Turso IVM is a later optimisation behind the same selector interface.
8. **Exhaustive tests at reactor level; properties and obligations at composition.** Reactors are small and pure, so exact output-set assertions are affordable. TCA's 2022 retreat shows exhaustive integration tests do not scale.
9. **Contract-test every simulation against reality.** Run each scenario against simulation and real adapter. Borrow SQLite's fault-injecting VFS for the storage simulation, and Turso/Antithesis-style deterministic simulation for the whole system.
10. **Bound each reactor step.** On mobile the step includes selector recomputation plus log append and fsync. Budget it per event class in CI. Long work becomes a sequence of events (Samek's "Reminder" pattern).
11. **Declare reads, not just events.** Wiring lists each reactor's input selectors as well as its input and output events, so the dependency graph can be checked for hidden coupling.
12. **Storage default.** SQLite in WAL mode as the durable log on iOS and Android; short reader transactions; explicit checkpoints, for example when the app is backgrounded; `synchronous=FULL` for durable facts, `NORMAL` only for re-derivable data. Turso stays behind the storage adapter until 1.0.
13. **Privacy and compaction are first-class.** Personal data is referenced by id in a separate erasable store, or crypto-shredded with the legal caveat noted. Snapshot-and-truncate is safe only for event classes no peer still needs to merge. Prefer soft-delete events.
14. **Expect process managers.** Retries, sagas and timeouts need state. Here that state is a selector over the log and the timeout is a time event from the clock adapter. The pure-reactor model needs an explicit orchestration pattern; Crux's closure combinators did not scale.
15. **Avoid the Elm nesting tax with code generation.** Wiring is derived by macros or a build-time registry, not hand-mapped at every level.
16. **Budget for ecosystem churn.** Crux broke its API four times in about a year, Elm stagnated, Replicache went into maintenance, ElectricSQL pivoted. Own the core traits (event, reactor, selector, adapter) and keep third-party engines behind adapters.

---

## Part 2 — Formal foundations

| # | Hypothesis | Verdict |
|---|---|---|
| H1 | The event log is a grow-only set (G-Set CRDT), so convergence is trivial if selectors are deterministic over an (HLC, device)-ordered log | **Supported for derived state; incomplete as stated** |
| H2 | Acyclic or stratified evaluation in topological order replaces capped fixpoint iteration, so termination and determinism hold by construction | **Strongly supported within one step; open across steps** |
| H3 | Monoid selectors give parallel replay, incremental maintenance and merge at once | **Partly supported; "at once" overclaims** |
| H4 | A small verified engine core + reference interpreter + randomized differential testing is L5 evidence for the engine, so per-app rigor cost tends to zero | **Supported for the engine; "zero per-app" is false** |

### 2.1 H1: the log as a grow-only set

**Evidence for.**
- Under set union, replicas that delivered the same updates have equivalent state; state-based and operation-based CRDTs are equivalent formulations (Shapiro et al. 2011).
- "The log is the state and views are functions of it" is established: the pure operation-based CRDT keeps a partially ordered log, answers queries from it and delivers by tagged causal broadcast (Baquero et al. 2014). The proposal adds a total order via HLC.
- The proof shape is mechanised: states converge if concurrent operations commute (Gomes et al. 2017). Folding the *same totally ordered sequence* everywhere is a stronger and simpler condition, so convergence follows once event sets are equal and the order is shared.
- HLC respects causality (if e happens before f, HLC(e) < HLC(f)), uses constant-size timestamps and stays close to physical time (Kulkarni et al. 2014). Breaking ties by device id gives a strict total order.
- Bayou applied writes in one global order with deterministic merge procedures, rolling back and re-executing when earlier writes arrived (Terry et al. 1995). The visibility/arbitration formalism states the design precisely (Burckhardt et al. 2014). Local-first experience found conflicts rarer than feared (Kleppmann et al. 2019).

**Caveats.**
1. **Persisted reactor outputs break the trivial argument.** Two devices can append different reactions to the same inputs because a late event changes the prefix each one saw. Bayou needed tentative versus committed writes, and the Dataflow Model needed retractions (Akidau et al. 2015). Options: (a) treat outputs as derived and recompute them; (b) fire reactors only on the causally stable prefix, at a liveness cost when a device is offline; (c) give outputs deterministic ids, for example a hash of reactor id, triggering event ids and engine version, so duplicates collapse under union; and (d) combine with a monotonicity analysis. By CALM, only monotone reactions are safe before stability. Reactions using absence ("no payment received by…") or treating an aggregate threshold as final need stability or coordination (Hellerstein & Alvaro 2020).
2. **External side effects cannot be retracted.** Emails, payments and printed output wait for stability or are idempotent with compensation.
3. **Convergence is not correctness.** HLC orders concurrent events by approximate wall time, not intent. Last-write-wins silently drops concurrent updates. Each selector needs an explicit concurrency semantics: multi-value, merge, or flag the conflict.
4. **HLC bounds depend on clock drift.** A device whose clock runs far ahead drags everyone's logical time forward. The HLC paper's remedy, ignoring out-of-bound messages, is not acceptable for a user's offline events, so a deterministic clamping or quarantine policy must be part of the semantics.
5. **Bit-determinism across platforms.** Floating-point non-associativity (Goldberg 1991), Wasm NaN payload nondeterminism (Watt 2018), hash-map order, locale, time zone and Unicode normalisation all break it.
6. **Code version is part of the state.** Devices running different selector versions over the same log diverge. Version and migration must be recorded in the log or made deterministic.
7. **The delivery assumption must be discharged.** Published CRDT proofs have failed on unrealistic network assumptions (Gomes et al. 2017). Equal event sets need an explicit eventual-delivery model with dedup by content-addressed ids.
8. **Log growth.** Compaction and snapshots are safe only past causal stability (Baquero et al. 2014).

**Proof obligations.**

| Obligation | Difficulty | Tool |
|---|---|---|
| Merge is a join-semilattice (union with idempotent, content-addressed ids) | trivial | Lean or Isabelle, once |
| (HLC, device) is a strict total order extending happens-before | small | Lean or Isabelle (port of Kulkarni's theorem) |
| Equal event sets ⇒ equal derived state, for every pure selector | trivial given purity | proof once; purity enforced by a language subset, not per-selector proof |
| Eventual delivery, anti-entropy, causal-stability detection | moderate | TLA+/TLC or Apalache (bugs needing 35-step traces: Newcombe et al. 2015) |
| Incremental insert of a late event = full replay | moderate | engine proof once (from-scratch consistency, Hammer et al. 2014); property and differential tests per selector |
| Monotone reactions are safe to fire before stability | static analysis | CALM/Bloom-style monotonicity checker on the reactor DSL |
| Cross-platform bit-determinism | empirical | differential replay of the same log on every target (native, Wasm, mobile) in CI |

### 2.2 H2: acyclic, stratified evaluation in topological order

**Evidence for.**
- Lustre forbids syntactically cyclic definitions; SCADE can forbid feedback without an explicit delay, and users accept this (Benveniste et al. 2003). Without zero-delay loops, a block diagram is functional, statically schedulable, and each step is loop-free code with a boundable worst-case execution time (Halbwachs et al. 1991).
- SCADE's code generator has been qualified since 1999; the vendor reports about 100 DO-178B/C level-A systems (Colaço et al. 2017) **[vendor-affiliated]**. Causality and initialisation checks are specified as type systems.
- Vélus proves in Coq that the scheduled step function implements the stream semantics. Its scheduler is untrusted code checked by a verified validator, a cheap pattern worth copying (Bourke et al. 2017).
- SCR consistency checking includes lack of circularity alongside disjointness and coverage (Heitmeyer et al. 1996).
- Capped iteration can make results depend on evaluation order and on the cap, and hitting the cap silently yields a non-fixpoint. Defining a reaction as a fixpoint caused years of causality trouble in Esterel and Signal; Lustre avoided it by forbidding delay-free loops (Benveniste et al. 2003).
- Where recursion is needed, stratified Datalog and non-monotone recursion have clean incremental semantics as nested-stream fixpoints in DBSP, verified in Lean (Budiu et al. 2023). Differential Dataflow and Naiad make the iteration coordinate part of time, keeping cyclic dataflow deterministic (McSherry et al. 2013; Murray et al. 2013).

**Caveats.**
1. **Expressiveness loss is real.** Esterel v4 required acyclicity and users rejected it, because legitimate programs had cycles that could never activate. Constructive semantics accepts them at the cost of complex analysis (Berry, draft). Some business logic, such as mutually dependent prices or discounts, is naturally simultaneous and must be restructured: an explicit delay to the next step, a solver node with its own proof, or a stratified recursion construct.
2. **Each step terminates; cascades may not.** When a reactor's output is fed back through the log, that feedback is the "delay" that breaks the cycle, so A→B→A can run forever across steps. Termination needs an acyclic event-type → reactor graph (cascade depth ≤ graph depth) or a ranking function per cycle. Otherwise a fuel bound quietly brings the iteration cap back.
3. **Multi-rate firing.** Reactors fire on different conditions. Lustre needs a clock calculus for this (Halbwachs et al. 1991; Biernacki et al. 2008). The event engine needs the equivalent: every selector a reactor reads is defined whenever it fires.
4. **Node functions must also be deterministic**: the bit-determinism issue from H1.

**Proof obligations.**

| Obligation | Tool |
|---|---|
| Wiring graph (selectors + reactors within a step) is acyclic | decidable graph check; small verified checker, or untrusted checker plus certificate |
| Computed schedule is a valid topological order | validator in Lean or Coq (Vélus pattern) |
| Scheduled evaluation = denotational semantics | engine proof once, or reference interpreter plus differential testing |
| Cascade termination across steps | acyclic event-type graph, or SMT (Z3/cvc5) ranking-function check per cycle |
| Stratification of recursive selectors (negation only across strata) | syntactic check |
| Every read selector is defined at each firing | type-system-style initialisation check |

### 2.3 H3: monoid selectors

**Evidence for.**
- Parallel scan works for any associative operator with O(n) work and O(log n) depth; segmented scans give per-entity folds in one pass (Blelloch 1990).
- Any finite-state fold is a monoid under function composition even when the step is not associative. Data-parallel state machines reached up to 21× on 8 cores by exploiting convergence to few active states (Mytkowicz et al. 2014).
- A list function computable both left-to-right and right-to-left is a homomorphism; others can often be fixed by tupling extra state (Gibbons 1996).
- With abelian groups the benefits arrive together: DBSP's linear operators are their own incremental versions, joins have a three-term rule, a chain rule composes incrementalisation, and Z-sets model retractions; all checked in about 5K lines of Lean (Budiu et al. 2023). Differential Dataflow generalises to a lattice of versions for out-of-order, multi-source updates (McSherry et al. 2013).

**Caveats.**
1. **Three properties, three laws.** Parallel prefix replay needs associativity. Merging device logs inserts earlier events into the middle of the order, which is not concatenation: the summary must be commutative, or you keep a balanced tree of partial aggregates and recompute O(log n) nodes per insertion. Retraction needs inverses; max, min and "latest" have none, so retraction means storing a multiset or recomputing.
2. **Most business selectors are not naturally monoids.** "Status after the last event" is right-biased last-write-wins, which is associative. Guarded state machines (overdraft rules, booking states) are monoids only through function composition, at a cost proportional to the reachable state space. Unbounded state does not parallelise in practice.
3. **Floating point is not a monoid** (Goldberg 1991). Different merge trees on different devices give different bits, breaking H1. Use integers, fixed-point or decimal, or reproducible summation at roughly 1–2× the cost of a plain sum (Demmel & Nguyen 2013).
4. **Integer overflow.** Wrapping arithmetic is associative but may not mean what the rule author intended. Use checked or big-integer arithmetic with a stated domain.
5. **Total vs partial time.** DBSP's theorems assume a totally ordered logical time. A multi-device log must be re-sorted, so a late event invalidates suffix aggregates unless the summary commutes.

**Proof obligations.**

| Obligation | Tool |
|---|---|
| Per-selector laws: associativity, identity, and where claimed commutativity and inverse | SMT when the combine is in a decidable fragment (linear integer arithmetic, bit-vectors, enums); otherwise property tests; library combinators proved once in Lean |
| Incremental result = from-scratch result | engine proof once (DBSP chain-rule style) |
| No floats or NaN in convergent selectors, or a reproducible-sum type | type-level ban in the DSL |
| Function-composition summaries have bounded state | static bound; reject or fall back to sequential |

### 2.4 H4: verified engine core, reference interpreter and differential testing

**Evidence for.**
- Vélus proves that generated assembly implements Lustre's stream semantics, with competitive timing (Bourke et al. 2017). Extensions are feasible but costly: adding reset was its own POPL paper (Bourke et al. 2020).
- After about 6 CPU-years of Csmith testing, CompCert's verified middle end had no wrong-code bugs while every other compiler tested miscompiled; the bugs found were in unverified parts (Yang et al. 2011). This evidence comes from an independent group.
- Mechanising the Wasm specification produced a verified executable interpreter, found specification issues, and supported differential fuzzing of production engines (Watt 2018). CakeML shows end-to-end verification of a full language implementation (Kumar et al. 2014).
- A TQL-1 qualified generator lets users drop or automate parts of code review, low-level testing and structural coverage objectives (Taft 2016) **[vendor slides]**. The SCADE handbook claims 100% model coverage implies 100% generated-code coverage, MC/DC included, under documented conditions (Ansys handbook) **[vendor]**.
- Translation validation, checking each run instead of verifying the generator, is the cheaper middle path (Pnueli et al. 1998).
- Proof infrastructure amortises: over half of one Isabelle CRDT development is reusable consistency and network modelling (Gomes et al. 2017), and DBSP's Lean development stayed small because the calculus is small.

**Caveats.**
1. **A verified engine faithfully executes wrong rules.** Per-app work that remains: stating domain invariants and checking them by model checking or bounded SMT; decision-table checks (cheap and automated); validating that rules match intent. Per-app cost falls toward *the cost of writing specifications*. SCR found missing cases that had survived two independent human reviews (Heitmeyer et al. 1996): evidence that automated checks are cheap and valuable, not that specs are free.
2. **Bugs live in the unverified rim**: serializer, storage, sync transport, clock source, UI bindings, FFI and host functions, the DSL parser. Watt needed explicit axioms about host functions.
3. **Spec validity.** The verified engine is only as good as its formal semantics. The reference interpreter must be readable, reviewed and checked against hand-written conformance tests.
4. **Qualification practice is mostly testing, not proof.** Exhaustive generator testing is the main qualification cost (Taft 2016). The ladder must say which evidence class L5 means: mechanised proof, qualified-by-testing, or both.
5. **Proof cost is front-loaded and grows with features.** Keep the kernel tiny and compile richer constructs into it (the SCADE 6 and Vélus strategy).
6. **Differential testing is only as strong as its generator.** Watt's fuzzing found no engine bugs, which is weak evidence either way. Track generator coverage and mutation score, and generate well-formed logs that include adversarial orderings, late events and clock skew.

**Evidence per layer.**

| Layer | Evidence |
|---|---|
| Kernel semantics (fold over ordered log, acyclic schedule, incremental = from-scratch) | Lean or Coq mechanisation, kept under about 10K lines by keeping the calculus DBSP-small |
| Production engine vs reference interpreter | randomized differential testing with a Csmith-style generator of logs and programs, on every target platform |
| Scheduler and planner outputs | translation validation (verified checker of untrusted output) |
| Sync protocol | TLA+/TLC including liveness (a missed liveness check let a lock-manager bug through at AWS: Newcombe et al. 2015) |
| Per app | automated decision-table checks plus optional invariants by bounded model checking |

### 2.5 Condensed obligation list

The level column is this document's suggestion against [rigor-levels.md](../rigor-levels.md) §4 **[inference]**; it is not in the source report.

| # | Obligation | Suggested level |
|---|---|---|
| 1 | Union/dedup semilattice; strict total order extends happens-before (Lean, once for the engine) | L5 (engine) |
| 2 | Purity and bit-determinism: the DSL bans floats (or uses reproducible sums), NaN, hash order, ambient I/O and time; differential replay on every target in CI | L3 (lint + replay); L4 (all targets) |
| 3 | Reactor outputs derived (not stored) or deterministically identified; monotonicity check; non-monotone reactions and external effects gated on causal stability | L3 (any synced component) |
| 4 | Wiring acyclicity plus verified schedule validator; event-type cascade acyclicity or SMT ranking function | L3 (graph checks); L5 (verified validator) |
| 5 | Per-selector algebraic laws by SMT; incremental = from-scratch theorem for the engine | L3 (laws by property test); L4 (by SMT); L5 (engine theorem) |
| 6 | Decision tables: overlap, gap, masked-rule and output-totality checks in a decidable condition fragment | L3 |
| 7 | Sync protocol and causal-stability detection: TLA+ with liveness | L4 (matches the existing concurrent-design gate) |
| 8 | Engine vs reference interpreter: randomized differential testing with a generator covering late events, clock skew and version skew | L4 |

### 2.6 Decision-table checking notes

- **Definitions exist.** For DMN, overlap (against the hit policy), missing rules (against the completeness indicator) and masked rules that never fire under priority policies are defined formally (Calvanese et al. 2016, 2018). SCR defines disjointness (pairwise conjunction false), coverage (disjunction true) and non-circularity (Heitmeyer et al. 1996).
- **SMT vs geometric.** A plain SMT query returns only a boolean; it does not list *which* rules overlap or *which* regions are uncovered. Calvanese et al.'s sweep-line algorithm over hyper-rectangles reports them in seconds for tables of 1000+ rules. Listing maximal overlap sets is clique enumeration (NP-hard, fine on sparse tables). SMT is more expressive (linear arithmetic, cross-column constraints) and returns witnesses; enumerating all gaps needs iterative blocking clauses. Use both: SMT for witnesses and cross-column logic, geometric enumeration for reports.
- **Pitfalls.** Keep conditions in a decidable, fast fragment: enums, intervals, linear integer arithmetic, bit-vectors; no non-linear arithmetic, strings or quantifiers. Completeness is relative to a declared input domain; without domain constraints you get spurious gaps. Check totality of the *output* per hit policy too.
- **Prior art gap.** No source combines decision tables and SMT obligations with reactor or effect architectures, and no paper applying SMT specifically to DMN was retrieved.

---

## Part 3 — Testing like humans, fast

### 3.1 Why this architecture makes testing cheap

FoundationDB, TigerBeetle and Antithesis spend a lot of engineering, or a hypervisor, to make runs reproducible from a seed (Zhou et al. 2021; TigerBeetle docs; Antithesis docs). ShardStore found that hidden nondeterminism such as randomised hash-map iteration breaks shrinking (Bornholt et al. 2021). Facebook's Sapienz deployment had to assume all tests are flaky (Alshahwan et al. 2018). If reactors are pure and every adapter is simulated, a session is `fold(reducer, log)`, and replay, carving, time travel and shrinking come almost for free; record-and-replay of native code costs under 2× (O'Callahan et al. 2017), and here it costs nothing beyond keeping the log.

### 3.2 Techniques table

The level column is remapped to this repo's ladder ([rigor-levels.md](../rigor-levels.md) §4): **L0** throwaway, **L1** internal tools, **L2** product code, **L3** core logic, HPC and security boundaries, **L4** high-integrity, **L5** proven. It gives the lowest level at which the technique should be *required*; the remap notes follow the table.

| Technique | What it finds | Evidence | Cost / speed | Determinism | Gaming risk and countermeasure | Level |
|---|---|---|---|---|---|---|
| Implicit oracles on random event streams (crash, panic, assertion, timeout, unhandled event) | Crashes, unhandled states, infinite loops | Monkey beat research tools overall (Choudhary et al. 2015); top method coverage on 22 of 41 industrial apps (Wang et al. 2018) | Very cheap; microseconds per event in-process | Full if seeded and adapters simulated | Agents can swallow errors with catch-alls; lint blanket catches, keep assertions on | **L1** |
| Golden replay of event logs (log → expected view model / output events) | Regressions in known flows | Carved unit tests keep most system-test fault detection at far lower cost (Elbaum et al. 2009) | Cheap; replay is a fold | Full | Agents re-bless goldens; golden updates need a reviewed diff with a reason tag | **L1** |
| Invariants on log + selectors, checked after every event | State corruption, broken rules, illegal UI states | Core oracle in FoundationDB simulation and ShardStore | Cheap if O(1)/O(n) per step | Full | Tautologies when the same agent writes invariant and code; owner-authored invariants, mutation score on the invariants | **L2** |
| Stateful property testing vs executable reference model | Feature interactions, ordering bugs, spec misreadings | 200+ issues, models 8–18× smaller, bugs in 20 implementations (Arts et al. 2015); 16 issues prevented, 13% code overhead (Bornholt et al. 2021); model properties most common idiom, 17 of 30 interviewees (Goldstein et al. 2024) | Model 1–20% of implementation; thousands of sequences/s | Full; shrinking needs it | Weak generators; model copied from implementation; require distribution reports, independent model, mutation testing | **L2** smoke under sealed seeds; **L3** against a sealed independent model |
| Automatic shrinking to a minimal failing log | Makes failures actionable; yields the regression fixture | 61 → 6 operations (Bornholt et al. 2021); Sapienz minimises sequence length (Mao et al. 2016) | Seconds to minutes per failure | Full | Shrinker converging on one bug hides others (Hughes 2016); add known-bug filters | **L2** |
| Metamorphic relations over logs (commuting permutation, idempotent replay, snapshot + suffix ≡ full replay, time shift, equivalent tap paths ⇒ same view model) | Ordering, idempotency, serialisation bugs without a full oracle | 3–6 diverse relations reach ≥ 90% of oracle-level detection (Chen et al. 2018) **[secondary]** | Cheap (2× replay) | Full | Trivially true relations; fixed owner-held catalogue | **L2** ≥ 1; **L3** ≥ 3 from distinct categories |
| Markov usage model / operational profile (Cleanroom statistical testing) | Field-weighted failures; reliability estimate and stopping rule | About 10× fewer customer problems, confounded with other measures (Musa 1993); Cleanroom teams passed more operational tests (Selby et al. 1987); no isolated effect size for the statistical-testing part | Cheap generation; model needs analyst time, free from field logs after launch | Full (seeded walks) | Profile tuned to avoid fragile paths; derive it from field logs or owner, report arc coverage | **L3**; reliability estimate **L4** |
| Swarm testing (random per-run subsets of event types, personas, faults) | Bugs masked by all-features-on configurations | +42% distinct crash causes in a week (Groce et al. 2012); used in FoundationDB | Free | Full | Low | **L3** |
| Coverage- or model-guided GUI exploration (Stoat, APE, Sapienz) applied at view-model level | Crashes in hard-to-reach screens | Stoat about 3× the crashes of Monkey/Sapienz on 93 apps (Su et al. 2017); APE 537 crashes on 1,316 apps (Gu et al. 2019) | Moderate; minutes to hours on devices, fast at view-model level | Full at view-model level, low on devices | Low | **L3** |
| Learned human-like interaction (Humanoid, trace corpora) | Realistic paths; regressions on common flows | Top-10 next-action accuracy 85%, but the model was not what drove coverage (Li et al. 2019) | Training cost; cheap inference | Deterministic if frozen and seeded | Low | Optional; not a gate |
| LLM GUI agents as session generators (GPTDroid, DroidAgent, AutoDroid) | Semantic inputs, multi-step goal flows | GPTDroid 71% activity coverage, +36% bugs (Liu et al. 2024); DroidAgent 61% vs 51%, 317 of 374 tasks realistic (Yoon et al. 2024); AutoDroid 71% task success (Wen et al. 2024); about USD 13–22 per app run at 2023 prices | Expensive, seconds per action | Non-deterministic; freeze outputs into logs | Cherry-picked sessions; version the corpus; never an oracle | **L3**, as offline corpus generator only |
| Deterministic simulation with adapter fault injection (FoundationDB, VOPR, Antithesis) | Retry, timeout, offline, partial-failure and liveness bugs | Production experience, qualitative (Zhou et al. 2021; TigerBeetle docs); cannot catch performance bugs or third-party behaviour | Faster than real time; embarrassingly parallel | Full by construction | Test-only branches; forbid `cfg(test)` paths in reactors, inject faults only at adapters | **L3** seeded simulation; **L4** fault injection mandatory |
| Record and replay of field sessions as a regression corpus | Real-world regressions, long-tail usage | Carving keeps most system-test effectiveness (Elbaum et al. 2009) | Storage and anonymisation | Full | Low | **L3** |
| Failure-preserving anonymisation of recorded logs | Makes field corpora shareable | No sensitive data shared on studied inputs (Clause & Orso 2011) | Moderate | Full | Low | **L3** when field logs are used |
| Contract tests for simulated adapters (recorded real-API examples) | Simulator drift: shapes, codes, semantics | Pact docs; FoundationDB bugs where the real OS contract was weaker than assumed | Cheap per PR; periodic real recordings | Full against recordings | Agent updates simulator and recording together; recordings come from real runs and are signed | **L3** |
| Trace validation of adapters and reactors against a spec | Semantic simulator infidelity, atomicity mismatches | Discrepancies found in every case study (Cirstea et al. 2024); data-loss behaviours in two sync services (Hughes et al. 2016) | Model-checking cost; nightly | Full | Low | **L4**; linked to a design proof at **L5** |
| Linearizability / isolation checking (Knossos, Porcupine, P-compositionality, Elle) | Sync and concurrency anomalies across devices | P-compositionality about 10× faster (Horn & Kroening 2015); Porcupine much faster than Knossos (Athalye 2017, blog) **[unverified]**; Elle linear-time (Kingsbury & Alvaro 2020) | NP-hard in general, linear with designed workloads | Full on recorded histories | Low | **L4**, only for multi-writer sync |
| View-model and accessibility-tree structural checks on the UI adapter | Unlabelled controls, unreachable actions, small targets, view-model/render mismatch | Not established by the corpus (engineering practice) | Cheap at view-model level; pixel diffs costly and flaky | Full at view-model level | Re-blessed snapshots | **L2** view model; **L3** accessibility tree |
| Mutation testing of the oracle suite | Weak tests and oracles: the main anti-gaming lever | Standard; Goldstein et al. 2024 found practitioners rarely evaluate their properties | Moderate to high, parallel | Full | Directly counters gaming | **L2** report; **L3** gate (as in the existing ladder) |
| Statistical evaluation of generators (many seeds, Mann-Whitney, ground-truth bugs) | False confidence in a technique | All 32 fuzzing papers surveyed had evaluation flaws (Klees et al. 2018) | Compute only | Full | Counters cherry-picking | **L3** |
| GPU batch simulation of sessions | Same bugs as CPU simulation, more sessions | Up to 27× vs *one* CPU core, with divergence limits (Rajan et al. 2014); 5–33× vs a 32-thread CPU only with a data-oriented rewrite (Shacklett et al. 2023) | High engineering cost | Achievable with care over floats and ordering | n/a | Optional at any level; not a rigor requirement |

**Remap notes.** The source report assumed a generic L0 none … L5 critical scale. Most rows map one-to-one. These were adjusted to line up with gates already in [rigor-levels.md](../rigor-levels.md) §4:
- *Stateful property testing* is split: L2 already requires hidden held-out property tests under gate-owned seeds, and L3 requires differential testing against a sealed reference and a model property per data-structure module.
- *Metamorphic relations* follow the existing numerical-kernel gates: ≥ 1 at L2, ≥ 3 from distinct categories at L3.
- *Deterministic simulation* follows the existing split: seeded simulation with identical trace hashes at L3, fault injection mandatory at L4.
- *Trace validation*: L4 for checking recorded real-API traces against the simulator; L5's existing gate links the design proof to code by trace validation or refinement.
- *Mutation of oracles*: L2 already gates pseudo-tested functions and reports survivors; L3 gates zero unexplained survivors.
- *Learned human-like interaction* and *GPU batch simulation* carry no rigor claim, so they are optional rather than placed at L4–L5.

### 3.3 Recommended tiered pipeline

**Preconditions, enforced as lints and gates.**
- **R1 Purity.** Reactors and selectors take (state, event) → (state′, output events) with no clock, RNG, I/O or hash-order dependence. Time and randomness arrive as events or from seeded adapters.
- **R2 Adapters are the only effect boundary.** Each has a real implementation and a simulated one with an explicit fault model (latency, error codes, offline, duplicate delivery, reordering), plus fault-injection hooks that fire only in simulation.
- **R3 Every run reproduces from (seed, commit, generator config)**, and every failure is emitted as a minimal event log plus its seed.
- **R4 Batch-step API**: `step(sessions[], events[])` with struct-of-arrays state where feasible, so CPU vectorisation and a later GPU port stay possible (Shacklett et al. 2021, 2023).
- **R5 Oracle independence.** Reference model, invariants, metamorphic catalogue and usage profile are owned or reviewed by the human owner, or written by a different agent than the implementation. Implementation agents cannot edit oracle files in the same change without a flagged review. This is the existing sealed-oracle principle (rigor-levels P6).

| Tier | When / budget | What runs |
|---|---|---|
| **0. Per commit** (agent inner loop) | < 30 s | Type check and lints (no `cfg(test)` in reactors, no clock or RNG outside adapters, no blanket error swallowing); golden replay of the smallest critical logs; stateful property smoke: about 200 sequences × ≤ 50 events against the reference model, fixed seeds, all invariants after every step |
| **1. Per PR** | < 10 min, parallel | (1) Full replay corpus: historical minimal failing logs, carved prefixes, frozen LLM-harvested sessions, anonymised field sessions; byte-identical view model and outputs, or a human-approved golden diff. (2) Stateful property testing, e.g. 10k–50k sequences, rotating plus fixed regression seeds; shrunk failures are added to the corpus. (3) Metamorphic suite. (4) Minimal arc coverage of the usage model: every (view-model state, UI event) arc at least once (Prowell et al. 1999). (5) Random event fuzz through the UI adapter at view-model level. (6) Adapter contract tests against signed real-API recordings. (7) UI structural checks on the accessibility-tree rendering of each view model. (8) Coverage and generator-distribution report; fail if a new event type or reducer branch is never generated. (9) Diff-scoped mutation on changed reactors and selectors |
| **2. Merge to main** | ≤ 30–60 min | Property testing at 10× Tier 1 volume with swarm configurations (Groce et al. 2012); deterministic simulation with adapter faults: connectivity flaps, timeouts, server errors, duplicated and reordered delivery, app background/kill/restore (process death = replay from the persisted log), clock skew; oracles are invariants, recoverability after faults stop, and liveness bounds; full mutation run on core reactors (weekly if too slow) |
| **3. Nightly** | hours | (1) Profile-weighted sessions from a Markov usage model, estimated by the owner before launch and from field transition frequencies after; several personas (Whittaker & Thomason 1994) with human-like timing (think times, double taps, back navigation, rotation, backgrounding); track the usage-vs-testing discriminant and a reliability bound per release, since zero failures means no information, not zero risk (Prowell et al. 1999). (2) Adversarial, coverage-guided sessions that push transition probabilities toward unvisited arcs, reported separately from the reliability estimate. (3) Scale: e.g. 10⁵–10⁶ sessions × 50–500 events across all cores, optionally on a self-hosted many-core machine or rented servers. A pure Rust reducer at about 1–10 µs per event would give on the order of 10⁷–10⁸ events per core-minute, so a 16-core machine covers 10⁵ sessions in minutes **[inference: engineering estimate, measure before buying GPUs]**. (4) Trace validation of staging-canary real-API traces against the adapter simulator (L4+); linearizability or isolation checks if several devices write. (5) Weekly offline corpus harvesting with LLM GUI agents and persona goals; logs are de-duplicated, triaged and frozen into the replay corpus; the LLM never decides pass or fail. (6) Every failure is shrunk, de-duplicated by (invariant id, last event type, shrunk-log hash) and filed with (seed, commit) |
| **4. Release** | per release | Statistically sized certification batch from the *frozen* usage profile with held-out seeds the implementation agents never saw; accept if zero (or ≤ k) failures in N sessions gives the target failure-intensity bound at the chosen confidence (Musa 2004). Real-device smoke on a small device matrix: UI adapter only, a few frozen sessions replayed through platform automation, screenshot diff limited to this tier |
| **5. Field loop** | post-launch | Capture input event logs (already the app's source of truth); anonymise with failure-preserving substitution and re-check that the outcome is preserved (Clause & Orso 2011); fold them into the replay corpus and the usage-profile re-estimate, so the operational profile comes from data rather than guesses (Musa 1993) |

Mapping to the repo's CI lanes ([rigor-levels.md](../rigor-levels.md) §6): Tiers 0–1 are the PR lane, Tiers 2–3 the nightly lane, Tier 4 a release gate.

**Anti-gaming summary for agent-written code.** Separate oracle and implementation authorship (R5). Mutation-test the oracles. Make generator-distribution and coverage reports mandatory artifacts. Use held-out certification seeds and a frozen profile. Require a reasoned, human-reviewed diff for golden or recording updates. Ban test-only code paths in reactors. Compare any new generator or "smart tester" against a random baseline over many seeds with statistics (Klees et al. 2018).

---

## Part 4 — Consensus, bias and gaps

### 4.1 Dominant consensus (high confidence, multi-source)

- **Determinism enables everything else.** Seeded randomised generation, automated oracles and shrinking are the core recipe for stateful and reactive systems (FoundationDB, TigerBeetle, ShardStore, Quviq/Volvo, Jane Street, Facebook with flakiness caveats). Sans-IO, re-frame coeffects and Mobius make the same point on the architecture side.
- **Executable reference models are the most productive oracle**, followed by invariants and assertions. Simple random testing is a surprisingly strong baseline for GUI event streams (Choudhary et al. 2015; Wang et al. 2018).
- **Acyclic dataflow with explicit delays gives determinism and termination per step**, across the synchronous-language, Datalog/IVM and SCR traditions.
- **CRDT convergence reduces to semilattice joins or commutation over a causal order**; associativity enables parallel scan.
- **Verified cores reduce defects in the verified portion**; bugs concentrate in the unverified rim.
- **In practice, the hard parts of event-log systems are evolution, privacy, rebuild cost and the bidirectional sync write path**, not the log itself (Overeem et al. 2021; ElectricSQL 2024; Kleppmann et al. 2019).

### 4.2 Secondary and contested positions

- **Acyclicity vs constructive causality.** Esterel's school holds that acyclicity is too restrictive and constructive semantics is worth its cost. This bears on H2's expressiveness.
- **Monotonicity vs CRDT objects.** The CALM/Bloom line argues monotonicity is the right abstraction (Hellerstein & Alvaro 2020). This bears on H1's reactor outputs.
- **Completeness as heuristic.** The Dataflow Model treats completeness as a watermark heuristic, not something provable (Akidau et al. 2015).
- **Usage realism.** Cleanroom and Musa argue operational-profile testing maximises delivered reliability per test hour. ShardStore, Humanoid and swarm testing point the other way for bug finding. They optimise different objectives: use both and report them separately.
- **Exhaustive vs non-exhaustive tests.** TCA moved from exhaustive to optional non-exhaustive tests for composed features; exhaustive assertions remain right for small pure units.
- **LLM testing agents.** Recent papers report sizeable coverage and bug gains, but mostly from single-group benchmarks with few seeds, LLM nondeterminism and possible training-data contamination.

### 4.3 Bias risks

- **Vendor or advocate authorship:** SCADE figures (about 100 level-A systems, the coverage implication) and TQL-1 savings come from vendor-affiliated authors or documents; no independent cost study was found. Quviq papers (Arts, Hughes), TigerBeetle and Antithesis docs, Crux adopter claims, Turso production claims, Ditto energy statements, Musa (reliability-engineering advocate) and the Cleanroom book (self-reported ROI confounded with other measures).
- **Deployer self-report:** AWS TLA+ and ShardStore, Facebook Sapienz KPIs, Photoroom's Crux account. Success-selection bias applies.
- **Proponent authorship:** most CRDT and local-first papers. Gomes et al. note a history of flawed CRDT proofs to their credit.
- **Independent evidence:** Csmith on CompCert comes from an independent group, which strengthens H4's engine claim.
- **Small, old or benchmark-specific studies:** Selby 1987 (15 student teams); Android tool comparisons on differing app sets, where Wang et al. 2018 shows rankings change on industrial apps.
- **Publication bias:** null results for "smart" testers are rarely published; Humanoid's honest negative finding is an exception.
- **Qualitative "works in practice" claims** for local-first and Automerge.

### 4.4 Gaps

**Architecture and practice**
- No system or paper treats the *combined* design: event-sourced reactors with persisted outputs, multi-device HLC sync and static wiring. The reactor-output convergence argument is made by analogy from Bayou, pure op-based CRDTs and CALM.
- No prior art combines decision tables and SMT obligations with reactor or effect architectures; no SMT-for-DMN paper was retrieved.
- No published measurements of Crux FFI serialisation overhead or view-model size limits; Photoroom's argument is structural.
- No independent battery or energy measurements for event-log local-first apps on phones.
- No empirical study of event sourcing on mobile clients: log growth per user-month, replay and boot time, migration across app versions in the field.
- LiveStore performance figures live only on an external dashboard and CI logs; log compaction is undocumented in the pages read. Turso has published no simulation or Antithesis bug statistics, its mobile SDK engine lineage is not explicit, and Zero's 1.0 status is unconfirmed.

**Formal**
- No empirical study of HLC under long offline periods or adversarial clocks.
- No literature on cross-platform bit-determinism of selector code beyond the Wasm NaN caveat.
- No quantitative per-application cost data for projects with a qualified generator versus without.
- Byzantine or malicious devices (forged HLCs, log tampering) are not covered.

**Testing**
- No head-to-head study of Markov usage testing vs stateful property testing vs random on the same event-driven app, with seeds and statistics. This would be a natural benchmark for this project.
- No standard deterministic metric of session realism. Candidate: divergence between generated and field transition matrices plus think-time distribution distance.
- Accessibility and visual oracles for the UI adapter were not covered by evidence; the structural checks above are engineering practice.
- No study measures how often simulated-adapter testing misses bugs that real-API testing catches, beyond FoundationDB anecdotes.
- Shrinking quality for long human-like sessions (hundreds of timed events) is unstudied.
- GPU batch simulation of application logic, as opposed to physics or RL, has essentially no literature.

**Cited from memory or not re-checked** **[unverified]**: Bernhardt 2012; the Elm 0.19.1 date; Mobius UpdateSpec details; rustls and h2 internals; TCA release years; the Porcupine speed-up (blog claim).

**Sources targeted but not obtained** (findings are absent or come only through citing works): Erb et al. 2016 on retroactive actor programming via event sourcing; Kahn 1974 on process-network determinism (via Benveniste et al. only); classic stratified-negation papers (via DBSP and CALM only); Berry & Gonthier 1992 (the constructive-semantics draft was used instead); Nagar & Jagannathan, CAV 2019 (automated CRDT verification); a DMN decision-table tool demo by Laurson & Maggi; the Latte (CHI 2021) and Groundhog (ASE 2022) accessibility-testing papers; MacIver & Donaldson, ECOOP 2020 (the Hypothesis reducer); Prowell & Poore 2004 and Poore, Mills & Mutchler 1993 on Markov reliability certification (via the Cleanroom book only). These are listed in [references.md](references.md) §22, not counted as read.
