# Declaration reporting and additional Rust contracts

The owner requested Issue #15 together with every additional Creusot candidate,
allowing dependency exploration to be skipped if verification requires excessive
complexity. Continue the unmerged PR #12; keep each topic in its own commit.

## Design

Keep actual shipping bodies in the existing verified decision crate. Preserve
borrowed identities, compact state and one-pass processing; no proof-only clone
of an algorithm, trusted application axiom or weakened evidence condition.

- Declaration reporting owns prose redaction, separate from locator rendering.
  Preserve email/plain text; redact credentials, queries and fragments inside
  URL tokens, failing closed for malformed URL tokens. CLI escaping stays in
  the CLI and JSON escaping stays in serde. Cover both declaration fields and
  both command surfaces.
- Evidence verification compares actual 32-byte content/source/request identities
  and enforces archive/sidecar strength and hashless request binding. Hashing,
  serialization integrity and metadata parsing stay outside the arithmetic core.
- Report aggregation verifies the real no-allocation traversal, including input
  errors and early incomplete exit; retain independent rank properties.
- Package-name normalization verifies byte-domain validity, canonical output and
  normalization semantics with a loop invariant; retain public string parsing.
- Metrics verification owns saturating counter merges and elapsed-time preservation;
  clocks/transport observations stay runtime-owned.
- Probe graph container support using the pinned verifier before restructuring.
  Skip full graph proofs if they require unsupported-container axioms, duplicate
  graphs or replacement of the existing efficient traversal; record evidence.

## Implementation and acceptance

1. Reproduce/fix #15 with isolated parameterized and generated regressions; commit.
2. Add evidence identities/strength contracts and public regressions; commit.
3. Add aggregation-loop contracts and regression properties; commit.
4. Add normalization contracts and independent public properties; commit.
5. Add counter-merge contracts and boundary/property cases; commit.
6. Record graph probe outcome (or verified implementation if straightforward).
7. Synchronize existing owners/user guidance, exact proof inventory, sensitivity
   witnesses and source hashes before one final PR push/CI run.

Each stage gets five self-review passes: requirements, real production connection,
typed/storage/performance invariants, independent test/proof sensitivity, and
reproduction/documentation boundaries. Tests use AAA, independent named cases
and properties. Run stable quality/tests, Lean freshness/adapters, strict Python
checks if tooling changes, bounded fuzz/CI and fresh three-target Creusot proofs.
