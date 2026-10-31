# Design note: the event-reactor architecture

Status: proposal. Evidence: [event-reactor-prior-art.md](../research/event-reactor-prior-art.md). Citations use
the keys in [references.md](../research/references.md).

## Goal

Make the **whole application a pure function of its input event log**. If that holds:

- every run can be replayed from a seed;
- deterministic simulation needs no runtime interception;
- reactors and selectors are small and pure, so proofs and mutation testing are cheap;
- time is just another event, so days of simulated use run in seconds.

High rigor becomes cheap because most of the work is paid once, in a small verified engine. Each app then pays
mainly for writing its specs. The research is clear that per-app cost does not reach zero: a verified engine
faithfully executes wrong rules (Bourke et al. 2017; Yang et al. 2011).

## The six elements

| | Element | Rule |
|---|---|---|
| **E** | Events only | Events are the only input and output of the system. Nothing else crosses the boundary. |
| **A** | Adapters | Small, at the periphery. Each has a simulation of its exact external API, and the simulation is contract-tested against reality. |
| **R** | Reactors | Pure: (event of this instant, declared selector values) → output events and commands. |
| **L** | Log and selectors | All state is an append-only event log plus pure, memoized selectors over it. |
| **W** | Static wiring | Every reactor declares its input event types, the selectors it reads, and its output event types. |
| **U** | UI as an adapter | Taps become events. The screen is a selector tree that computes a view model. |

No existing system combines all six. Crux is closest on E, U and W in Rust on mobile. re-frame is closest on R
and selectors (its coeffects and signal graph). LiveStore is closest on L, with an event log materialized into
SQLite on the device. Lustre/SCADE is the safety-critical precedent for static dataflow wiring. See the fit
matrix in the research document.

## Core semantics

### Events

- **Immutable**, with content-addressed ids (a hash of the canonical encoding).
- Envelope: `id`, `type@version`, HLC timestamp, device id, payload.
- **Versioned from the first commit**, for example `cart.item_added@1`. Upcasters are pure functions with their
  own tests. Old versions are retired by copy-and-transform, because run-time upcasting otherwise accumulates
  forever (Overeem et al. 2017, 2021).
- **Explicit policy for unknown events** (warn, ignore, fail or handler), because mobile fleets run mixed app
  versions for months.
- **Three classes**, declared in the wiring:
  - *durable facts* are persisted and synced;
  - *session* events are persisted locally and never synced;
  - *ephemeral* signals (scroll, hover, appear) live in an in-memory ring and are never logged. Routing them
    through the durable path is what caused the TCA performance wall.
- **No personal data in events.** Reference it by id in a separate erasable store. Crypto-shredding is an option,
  with legal caveats.

### Log, order and merge

- Each device keeps its own append-only log. The merged state is the **union** of events, a grow-only set CRDT
  (Shapiro et al. 2011).
- **Total order:** `(HLC timestamp, device id, event id)`. HLC extends happens-before and stays within ε of
  physical time (Kulkarni et al. 2014).
- Two devices with the same set of events compute the same derived state, provided selectors are deterministic
  (Gomes et al. 2017; Baquero et al. 2014).
- Clock, randomness, ids and locale come in from adapters as events or coeffects; reactors never read them
  ambiently. A device whose clock runs far ahead is handled by a deterministic clamp or quarantine rule, not by
  dropping the user's events.

### Selectors

- Pure, deterministic functions of the ordered log, or of other selectors.
- Each declares its inputs, its cache key and equality, and its eviction policy. Reselect's thrashing and
  memory-leak history is the cautionary tale.
- Each declares its **algebra**. The algebra decides which capabilities the engine may use. Each capability needs
  a different law, so they do not come together for free:

| Declared algebra | Unlocks | Source |
|---|---|---|
| Plain left fold | Sequential replay only | — |
| Associative (monoid) | Parallel prefix-scan replay, segmented per entity | Blelloch 1990; Gibbons 1996 |
| Commutative monoid | Inserting late events with any merge order | — |
| Abelian group (has inverses) | Cheap retraction; incremental = from-scratch by construction | Budiu et al. 2023 (DBSP) |
| Finite-state fold | Parallel replay by composing transition tables | Mytkowicz et al. 2014 |

- **Bans** inside selectors that must converge: floats, NaN, hash-iteration order, ambient time and locale. Use
  integers, fixed-point or decimal, or reproducible summation (Demmel & Nguyen 2013).
- Materialized snapshots are allowed as caches. The invariant `snapshot == fold(log)` is checked in tests and
  nightly. Selector materializations are written in the same local transaction as the append, so read-after-write
  holds on the device.

### Reactors

- **Run-to-completion:** one input event is one instant, as in QP.
- Outputs are either *derived facts* or *commands* to adapters.
- **Derived facts are not persisted as truth.** They are recomputed, or given deterministic ids
  (`hash(reactor id, trigger event ids, engine version)`). A reaction persisted from one device's partial view
  would make devices diverge.
- **Commands** (send an email, charge a card, print) leave through an outbox with idempotency keys.
- **Monotone reactions** may fire at once. **Non-monotone reactions**, such as those that depend on something not
  having happened ("no payment by…"), wait for causal stability, the point where no earlier event can still
  arrive (CALM; Hellerstein & Alvaro 2020). Corrections are compensation events, as in Bayou (Terry et al. 1995).
- **Rules are written as decision tables in a decidable fragment**: enums, intervals, linear integer arithmetic,
  bit-vectors, over a declared input domain. Rules outside the fragment are Rust reactors that carry their own
  proofs (Verus or Kani) at the component's level.

### Evaluation: no fixpoint loop

- **Within an instant**, the wiring graph must be acyclic. Evaluation is one pass in topological order, so it
  terminates and is deterministic by construction. There is no iterate-until-stable loop and no iteration cap.
  This is the Lustre/SCADE design (Halbwachs et al. 1991; Benveniste et al. 2003). An untrusted scheduler computes
  the order and a verified validator checks it, the Vélus pattern (Bourke et al. 2017).
- **Across instants**, a reactor's output event can trigger another reactor. Termination needs an acyclic
  event-type → reactor graph, or a ranking function per cycle that the SMT solver checks. A fuel bound is not
  accepted, because it silently brings the iteration cap back.
- **Initialization check:** every selector a reactor reads must be defined whenever it fires, the analogue of
  Lustre's clock calculus.
- **Per-instant budget:** selector recompute plus the log append, measured per event class in CI. Long work
  becomes a sequence of events.

The cost is expressiveness. Esterel users rejected strict acyclicity. Simultaneous business logic needs an
explicit delay event, a dedicated solver node with its own proof, or a stratified recursion construct.

### Adapters

- **Sans-IO:** a pure state machine for the protocol (retries, timeouts, connection state), plus a trivial driver
  loop that performs the I/O. Only the driver is unproved.
- Every adapter ships a **simulation of the exact external API** with an explicit fault model: latency, error
  codes, offline, duplicate and reordered delivery, process death.
- The simulation is only as good as its fidelity:
  - **contract tests** replay signed recordings of real API traffic against the simulation, per PR;
  - **trace validation** against a spec runs at L4 (Cirstea et al. 2024 found spec–implementation discrepancies in
    every system they studied);
  - storage simulations borrow SQLite's fault-injecting VFS (out of memory, I/O errors, torn writes).

### UI

- **Input:** taps, keys and gestures become semantic UI events, each classified as durable, session or ephemeral.
- **Output:** a selector tree computes the view model. It is sent to the renderer as **typed key-path patches**
  derived from selector deltas, not as a full view model per render. Photoroom had to retrofit this onto Crux.
- **The rigorous artifact is the view model and its accessibility tree.** Pixels belong to a renderer adapter at
  L2–L3; no renderer can be qualified.
- The renderer is open (see the open decisions).

### Storage

- The event store sits behind an adapter.
- **Default: SQLite in WAL mode.** Use `synchronous=FULL` for durable facts and checkpoint explicitly, for example
  when the app goes to the background. SQLite reports 100% MC/DC for its core, which is unusually strong evidence
  for an off-the-shelf component.
- **Candidate: Turso**, behind the same adapter until 1.0. It is a Rust rewrite with MVCC, change data capture,
  DBSP incremental views and deterministic simulation testing. Its incremental views are explicitly not
  production-ready yet. Because the file formats are compatible, Turso can run against SQLite as a differential
  oracle in tests.
- **Compaction** is allowed only past causal stability, and only for event classes no peer still needs to merge.
  A device can come back after months (Kleppmann et al. 2019).

### Sync

- Devices exchange log segments by anti-entropy. Stability detection decides when non-monotone reactions and
  compaction may run.
- The protocol is modelled in TLA+, liveness included (Newcombe et al. 2015).
- Simpler deployments may let a server assign the total order, as LiveStore does. The engine semantics are the
  same.

## Proof obligations and where they live

| # | Obligation | Discharged by | Paid |
|---|---|---|---|
| 1 | Merge is a semilattice; `(HLC, device, id)` is a strict total order that extends happens-before | Lean or Isabelle | Once (engine) |
| 2 | Purity and bit-determinism of selectors and reactors | Language bans, plus replaying a corpus on every target platform in CI | Once, plus CI |
| 3 | Reactor outputs are derived or have deterministic ids; monotonicity check; non-monotone reactions and commands wait for stability | Static checker | Engine, plus per app automatically |
| 4 | Wiring acyclic within an instant; verified schedule validator; termination across instants | Graph check; SMT ranking functions | Engine, plus per app automatically |
| 5 | Each selector's declared algebraic laws; incremental equals from-scratch | SMT, or property tests outside the decidable fragment; a Lean theorem for the engine | Per selector, automatic |
| 6 | Decision tables: overlap, gaps, masked rules, output totality per hit policy | SMT over the decidable fragment (Calvanese et al. 2018; Heitmeyer et al. 1996) | Per app, automatic |
| 7 | Sync protocol and stability detection | TLA+/TLC, with liveness | Once |
| 8 | Engine matches its reference interpreter | Randomized differential testing. The generator covers late events, clock skew and version skew, and its own coverage and mutation score are tracked. | Once, plus CI |
| 9 | `snapshot == fold(log)`; upcasters preserve meaning; adapter simulations match recordings | Property and replay tests; contract tests | Per app |

## Level mapping

| Part | Level | Main evidence |
|---|---|---|
| Engine core: log, order, selector evaluation, scheduler validator | L5 | Obligations 1, 2, 4, 5 and 8 |
| Reactors, selectors, upcasters, decision tables | L4–L5 per app | Automatic obligations 3–6, exhaustive reactor-level tests, mutation testing |
| Adapter protocol state machines | L3–L4 | Simulation, contract tests, trace validation at L4 |
| Thin I/O drivers | L2–L3 | Contract tests against real APIs |
| Renderer | L2–L3 | View-model and accessibility-tree checks; a few real-device smoke tests |
| Event store | SQLite's own evidence, plus obligation 9 | |

## Testing like a human, fast

The research separates two goals that are easy to confuse:

- **Estimating field reliability** comes from *usage-profile* testing. Cleanroom's Markov usage models (Whittaker &
  Poore 1993; Musa 1993) give a reliability estimate and a stopping rule.
- **Finding bugs** comes from *diversity*: swarm testing (Groce et al. 2012), coverage-guided exploration, and
  stateful property tests against a reference model. Arts et al. (2015) raised 200+ issues with models 8–18× smaller
  than the suites they replaced; Bornholt et al. (2021) prevented 16 production issues for a 13% code overhead.
  Biasing toward production traffic gave no benefit for bug finding in ShardStore, and plain random testing remains
  a strong GUI baseline.

Both run, and they are reported separately. Oracles are always deterministic:

- invariants checked after every event;
- the reference model;
- a fixed catalogue of metamorphic relations (commuting events, duplicate delivery, snapshot plus suffix equals
  full replay, equivalent UI paths give an equal view model);
- view-model and accessibility-tree checks.

**LLM GUI agents only generate sessions**, offline. Their event logs are frozen into the replay corpus. They never
decide pass or fail.

Pipeline, adapted from the testing report:

| Tier | Budget | Content |
|---|---|---|
| Inner loop | < 30 s | Lints (no ambient time, randomness or I/O in reactors; no test-only branches); golden replays; stateful property-test smoke run with invariants |
| Per PR | < 10 min | Full replay corpus; 10k–50k model-based sequences with automatic shrinking; metamorphic suite; minimal arc coverage of the usage model; random UI events; adapter contract tests; accessibility-tree checks; generator coverage report; diff-scoped mutation testing |
| Merge | ≤ 60 min | 10× volume with swarm configurations; deterministic simulation with adapter faults (offline flaps, timeouts, duplicate and reordered delivery, process death, clock skew) |
| Nightly | hours | 10⁵–10⁶ profile-weighted sessions with human-like timing; adversarial sessions steered away from the profile; trace validation; reliability estimate per release |
| Release | — | Certification batch on held-out seeds from the frozen profile; real-device smoke on a small matrix |
| Field | continuous | Anonymised input logs from the field (failure-preserving) feed the replay corpus and re-estimate the usage profile |

**Speed:** a pure Rust reducer at roughly 1–10 µs per event would cover 10⁵ sessions in minutes on a many-core
CPU. That is an estimate to measure, not a result. For this workload the default is multi-core CPU, not GPU. The
evidence for GPU test execution is up to 27× against *one* core, which is below parity with 32 cores, and it
breaks down on branch divergence (Rajan et al. 2014). The simulator exposes a batch-step API with
struct-of-arrays state so a data-oriented GPU port stays possible later (Shacklett et al. 2023). The GPU pilot in
`pilots/gpu-exhaustive` measures a different workload: exhaustive checks of numeric kernels.

## Relation to existing work

- **Crux**: same boundary and shell model; mutable model, no log, view model not incremental.
- **re-frame**: coeffects and the layered subscription graph are the direct precedent for selectors and adapters.
- **LiveStore**: event log materialized into SQLite on the device, versioned events, rebase-based sync. Limited
  to data that fits in memory.
- **Lustre/SCADE**: static dataflow, causality analysis, a qualified code generator. Precedent for obligations 4
  and 8.
- **Bayou** and **pure operation-based CRDTs**: a deterministically replayed global log with corrections.
- **QP active objects**: run-to-completion semantics. This design moves the state out of the actors and into the
  log plus selectors.
- **Earlier internal work** by the project owner: a decision-table rule language checked by SMT, and a pure UI
  interaction reducer verified with specs, refinement and mutation testing. The rule language and verification
  method carry over. That work's CRDT-document state model and capped fixpoint engine do not.

## Open decisions

1. **Own core or Crux?** Recommendation: own the core abstractions (event, reactor, selector, adapter, engine),
   because the ecosystem churns. Crux broke its API four times in about a year, Replicache went into maintenance
   mode, and ElectricSQL pivoted. Reuse Crux's shell and FFI ideas, and reassess Crux as a shell layer once its API
   stabilises.
2. **Renderer:** a short mobile spike comparing candidate Rust renderers on iOS and Android startup, frame time,
   binary size and accessibility.
3. **Store default:** SQLite now, Turso behind the adapter, with differential tests between them.
4. **Decimal type and float policy** for selectors that must converge.
5. **Sync topology:** server-assigned order or peer HLC order.
6. **How far the decision-table fragment extends** before reactors fall back to Rust with Verus or Kani.

## Roadmap

| Phase | Deliverables |
|---|---|
| 1. Kernel | Event envelope and log; HLC order; selector and reactor traits with declared wiring; acyclicity and initialization checks; reference interpreter; deterministic simulation harness; obligations 1, 2, 4 and 6 |
| 2. Edges | Sans-IO adapter framework with simulations and contract tests; SQLite event store; view-model patch protocol; obligation 9 |
| 3. Distribution and testing | Sync with a TLA+ model and stability detection (obligations 3 and 7); usage-model and swarm session generators; engine differential testing (obligation 8) |
| 4. Product surface | Renderer choice and mobile shells; field-log anonymisation and replay loop |
