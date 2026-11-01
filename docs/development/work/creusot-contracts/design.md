# Creusot contracts for production decisions

The owner approved implementing both high-priority candidates: result accounting
and report status/exit precedence, on a dedicated branch with separate commits.
The existing Lean models, corpus adapters, properties and fuzz targets remain.

## Approach

Use a small workspace crate, `taloon-decisions`, for production observation-slot
transitions and verdict decisions. The shipping crate calls these exact bodies;
Creusot verifies that same crate. An optional serde feature preserves report
serialization. `creusot-std` is selected only under `cfg(creusot)`; ordinary builds
need no verifier, solvers or proof runtime. Pin Creusot 0.14.0 and its dedicated
nightly separately from Rust 1.99.1.

A duplicate-only proof model would leave an unnecessary correspondence gap.
Verifying the whole application would require specifications for parsers, I/O
and external crates. Neither is chosen. The small crate isolates existing pure
semantics, without replacing BTreeMap with a slower proof-friendly container.

## Contracts and boundaries

Observation slots distinguish Missing, Single(payload) and Duplicate. Recording
into Missing preserves the exact payload; recording into either occupied state
makes Duplicate. Taking a slot empties it, returning exactly its sole payload
or the corresponding typed missing/duplicate failure. Duplicate input cannot
become successful evidence. BTreeMap identity selection/removal and Evidence
rebinding remain outside Creusot; tests exercise their connection through the
actual ResultLedger and public Inspection/Description APIs. No trusted custom
function or external BTreeMap axiom is introduced.

Verdict construction prioritizes input errors/incompleteness over violations,
and violations over compliance. Exit codes are 2, 1, or warning-dependent 0/1,
respectively. Contract-bearing code owns those decisions; report traversal,
warning discovery and serialization retain regression/property/oracle coverage.

## Acceptance

Generate fresh nonempty obligations and prove them with bounded solvers. Reject
stale/missing proof targets and failed generation/proof. Demonstrate sensitivity
with compiling mutations to duplicate handling, consumption and exit precedence.
Retain fixed named cases plus independent proptest properties, AAA and one
scenario/property per test. Run existing Lean correspondence and Rust checks.
Record five self-reviews per stage, actual commands and limits. Integrate the
proof command into CI without altering required GitHub settings in this task.

No user-visible behavior or CLI option changes are intended. Update the owning
verification decision and developer commands; no user guide change is needed.
