---
type: Decision
title: Formal models and generated behavioral tests
description: Use Lean oracles, property tests, fuzzing, and typed Rust boundaries without conflating their guarantees.
status: accepted
sources:
- id: html-index-filtering-review
  resource: ../work/html-index-filtering/review.md
  working_tree: untracked
  sha256: f040c5b2244aa4d8e5dcaa5c6762316c0a59a1d85e193b32269d1983154b3f55
- id: evidence-ownership-review
  resource: ../work/evidence-ownership/review.md
  working_tree: untracked
  sha256: 9da1686b4f85fe2e04ea4f700aea467fd9ca15b8678e4459b53699adb5a33d78
- id: permission-unit-review
  resource: ../work/permission-units/review.md
  working_tree: untracked
  sha256: 38137e298604b7ba9efc0d82deda8f5ca649d9106e7005866847968929f08eca
- id: policy-evaluation-review
  resource: ../work/policy-evaluation/review.md
  working_tree: untracked
  sha256: fb1ab90f04a90894c3876baf58456f31b3e08be6661077e217065f1d91037ef7
- id: core-design-review
  resource: ../work/core-accounting-and-resolution/review.md
  working_tree: untracked
  sha256: 9dbaff1fe9d2990398de44ec0ee376a531b48e8ae67bf9826a424d62cb870434
- id: describe-model
  resource: ../../../formal/TaloonOracle/README.md
  working_tree: modified
  sha256: 52fd4b66dbd99ceecf8e8eae2419fb72467a12b99a9ebb1473effc18cec04f4a
- id: acquisition-measurements
  resource: ../work/index-acquisition/measurements.md
  working_tree: modified
  sha256: 6aaeeade6f9e0f1a495c5328501465ef1aa571660070ecd2957f3b71f4a2f67a
- id: owner
  resource: ../work/initial-design/implementation-approval.md
  working_tree: untracked
  sha256: 1593dc51b5663e513152059e50b39795063a696bf3b6b417c9a5c8fae959834c
- id: hoimin-model
  resource: https://github.com/HayaoSuzuki/hoimin/blob/542ec59c002d72fdff0137aa90113ffa59d70844/formal/HoiminOracle/HoiminOracle/MutationScoreExitPolicyModel.lean
- id: hoimin-generator
  resource: https://github.com/HayaoSuzuki/hoimin/blob/542ec59c002d72fdff0137aa90113ffa59d70844/formal/HoiminOracle/MutationScoreExitPolicyAuditMain.lean
- id: random-order-reference
  resource: https://github.com/HayaoSuzuki/hoimin/blob/39d35f3980c428ab15258b519a233b86f1551ffa/.github/workflows/ci.yml
- id: random-order-review
  resource: ../work/randomized-tests-and-python-checks/review.md
  working_tree: untracked
  sha256: db9b9efe91527e984e0e7feb4332b827b8c834d56a0351a8697d4e24056f5f2f
---
# Decision and current status

The owner requested formal proofs informed by hoimin, active fuzzing and
property-based testing, and deliberate use of Rust types/enums. The owner also
selected inline implementation with at least five self-reviews per stage.
The [implementation plan](../work/initial-design/implementation-plan.md) assigns
these checks to the stages that introduce each behavior. This is an approved
verification design. The [initial model and correspondence record](../../../formal/TaloonOracle/README.md)
and [stage reviews](../work/initial-design/implementation-reviews.md) record
the implemented permission/evidence/completeness/exit slices and operational
Rust checks, final integration/performance measurements, and independent review.[^owner]

Follow hoimin's separation of typed semantic models and a deterministic corpus
generator. Keep kernel-checkable proofs in imported model/proof modules;
serialization, finite cases, and deliberately broken variants belong to the
executable audit boundary. hoimin's model uses an inductive Status and typed
structures for counts/policy, while its audit executable generates and
freshness-checks JSON expectations. Use that organization for taloon-specific
semantics rather than copying hoimin's mutation-test domain.[^hoimin-model][^hoimin-generator]

# Claims and correspondence boundary

The initial Lean model covers atomic permission units, AND/OR, evidence strength,
expiry, complete target/result accounting, and exit precedence. WITH expressions
remain atomic permission units. Prove permission monotonicity, rejection of weak
evidence/missing targets, and precedence under explicit premises. Check broken
variants that swap AND/OR, promote sidecars, accept missing results, or prefer a
violation over incompleteness. Do not claim these proofs establish parser,
network, filesystem, or complete Rust correctness.[^owner]

Record correspondence before implementing the model. Initial cases are
model-only until a real Rust public adapter can configure the same premise and
observe the complete result. Promote exercised cases to strict only after that
mapping exists. Keep internal-fixture and infrastructure-error classifications
separate. An unbounded theorem about model expressions or dates does not imply
that a finite Rust representation accepts the model's entire domain.[^owner]

| Premise/observation | Planned Lean representation | Planned Rust configuration/observation | Initial mode |
| --- | --- | --- | --- |
| Explicit-unit permission | Inductive expression, allowed atom set, Boolean evaluation | Public policy parser/evaluator with the same expression and units | model-only |
| Evidence sufficiency | Required/observed strength constructors | Public Evidence construction and Inspection result | model-only |
| Expiry boundary | Ordered evaluation/expiry days | Public policy with same valid UTC dates; input error or valid policy | model-only |
| Complete result accounting | Expected IDs and observed terminal outcomes | Public Inspection with same request/result identities; report status | model-only |
| Exit precedence | Typed aggregate status and warning escalation | Public Report exit_code with same complete/incomplete outcomes | model-only |

Lean generates schema-versioned JSONL expectations. Rust tests translate each
case to real public behavior and compare stable observations; they do not
recompute expected values. Keep every case isolated. Distinguish a mismatch
from panic, timeout, parse/setup failure, or missing tooling. Preserve minimal
counterexamples and reproduction commands. CI regenerates and diffs the corpus
before running strict correspondence.[^owner]

# Generated tests and representation

Use property tests for permission monotonicity, normalization, graph/result
permutations, source identity separation, hash mutation, cache roundtrips, and
exit precedence. Begin with 256 cases, bounded expression/graph sizes, and
persisted minimized regressions. Derive expectations independently. Property
testing supplements fixed cases and model correspondence.[^owner]

Use a separate cargo-fuzz workspace for policy TOML, uv.lock, metadata,
wheel/sdist bytes, and structured inspection results. Seed valid semantic cases
as well as malformed inputs. Run bounded smoke campaigns, retain crash inputs
as regressions, and keep libFuzzer/nightly requirements out of the runtime
package. An installed taloon user needs neither Lean nor fuzz tooling.[^owner]

Use validated newtypes for names, versions, dates, hashes, and opaque request
IDs. Use exhaustive enums for sources, artifacts, evidence, status and diagnostic
codes. Keep constructors private or validating; public callers cannot set a
verification Boolean. Model remote hash requirements and unsupported sources
as distinct states so omission does not become acceptance. Types narrow invalid
states; behavioral tests and correspondence establish the remaining rules.[^owner]

# Randomized test execution

Retain the stable Rust suite and add a separate pinned nightly libtest shuffle
job, following the inspected hoimin implementation. Shuffling applies within
individual executables and preserves the usual ignored-test boundary. Native
seed output and explicit seed replay enable order-dependent failure diagnosis;
parallel scheduling and Cargo executable order remain outside that promise.
Python tooling uses required pytest-randomly from the existing docs/test group,
with frozen resolution and default collection/random-generator seeding. Neither
tool becomes a runtime dependency.[^random-order-reference][^random-order-review]

Keep invocation and reproduction commands in the [developer guide](../index.md#randomized-test-order).
Revisit when the nightly harness options/toolchain or pytest plugin compatibility
changes. Random order supplements behavioral/property/formal checks; it does not
prove that every possible ordering is safe.[^random-order-review]

# Test design and Python quality

New and modified tests separate Arrange, Act and Assert, with one scenario or
property per function. Use independently executed rstest/pytest cases for input
variants and proptest/Hypothesis to generalize meaningful domains. Fixed examples
remain readable regressions; generated checks require independent expected values
or invariants. Existing Lean correspondence and fuzz smoke remain separate
sources of evidence.[^random-order-review]

Maintain Python tooling under Ruff ALL and ty all-error strict checks, including
tests. The formatter owns two conflicting lint rules; narrow test conventions
and reasoned boundary exceptions remain explicit. No blanket annotation or type
ignore bypass is enabled. These development gates do not add Python runtime
dependencies. Revisit exceptions on tool upgrades and when the maintained Python
surface changes. Commands and test review rules belong in the
[developer guide](../index.md#python-quality).[^random-order-review]

# Resource limits and reconsideration

Pin Lean to the inspected hoimin version, v4.32.2. Start suspect individual Lean
commands with an external 20-second deadline, one process at a time; preserve
heartbeat limits. Record finite case counts, time and memory when available.
Replace expensive enumeration with inductive lemmas instead of enlarging bounds
after a sharp cost increase. Start fuzz smoke runs at 1,000 executions and
4,096-byte parser inputs on nightly-2026-10-08. Record larger campaigns and
their limits rather than implying exhaustive input coverage.[^owner]

Revisit the model when permission semantics, evidence trust, completeness, or
exit status changes. Revisit adapters when public API or metadata formats
change. Consult the owner before narrowing inspection scope or requiring new
runtime software. Record test/proof outcomes with their actual revisions and
environment when implementation exists.[^owner]

[^owner]: [Owner approval and added verification requirements](../work/initial-design/implementation-approval.md).
[^hoimin-model]: [Inspected hoimin mutation-score model](https://github.com/HayaoSuzuki/hoimin/blob/542ec59c002d72fdff0137aa90113ffa59d70844/formal/HoiminOracle/HoiminOracle/MutationScoreExitPolicyModel.lean).
[^hoimin-generator]: [Inspected hoimin corpus generator and sensitivity checks](https://github.com/HayaoSuzuki/hoimin/blob/542ec59c002d72fdff0137aa90113ffa59d70844/formal/HoiminOracle/MutationScoreExitPolicyAuditMain.lean).

# Exact index lookup correspondence

The [index model and adapter worksheet](../../../formal/TaloonOracle/README.md#exact-index-lookup-model)
extend verification to per-run index filtering. Kernel proofs cover lookup
preservation for included canonical URL keys, wrong-hash rejection and duplicate
rejection. The finite generator produces 216 typed cases with scope-loss,
hash-bypass and duplicate-acceptance sensitivity witnesses. A private binary
adapter exercises both real JSON/HTML parsers (432 observations), plus successful
metadata evidence construction without sidecar promotion. It does not prove
HTTP, URL normalization, cryptographic hashes or OnceLock synchronization.
Concurrent transport/panic regressions and two bounded 256-case parser properties
cover those separate implementation risks. CI builds the model and checks both
corpora for freshness.[^acquisition-measurements]

[^acquisition-measurements]: [Measured acquisition and stage reviews](../work/index-acquisition/measurements.md).

# Description selection correspondence

The description model proves external inventory filtering, root retention and
selection under explicit canonical-identity/workspace premises. Its separate
50-case corpus crosses ten four-node graph profiles with five root sets; the
public Description adapter checks actual request inventories. A 256-case
independent transitive-closure property supplements those profiles. The model
does not prove general queue correctness, unbounded reachability completeness,
lock parsing, HTTP, hashes or license truth. See the [formal worksheet and
reproduction commands](../../../formal/TaloonOracle/README.md#description-selection-model)
for sensitivity witnesses, finite bounds and observed results.[^describe-model]

[^describe-model]: [Description model correspondence and reproduction](../../../formal/TaloonOracle/README.md).

[^random-order-reference]: [Inspected hoimin nightly shuffle job](https://github.com/HayaoSuzuki/hoimin/blob/39d35f3980c428ab15258b519a233b86f1551ffa/.github/workflows/ci.yml).
[^random-order-review]: [Randomized test-order verification and review](../work/randomized-tests-and-python-checks/review.md).


# Core observation and graph refactoring evidence

The shared observation ledger is checked through both public command surfaces,
using independent cardinalities, arrival permutations and failed duplicates.
Existing Lean completeness expectations remain unchanged and the strict Rust
adapter still exercises Inspection::evaluate. Enum storage and performance are
outside the Lean claim. Generated DFS enumeration supplies an independent
shortest-path oracle; fixed regressions cover cyclic extras, tie order and marker
provenance. Debug redaction has fixed and generated cases plus a persisted
minimal witness. The [stage record](../work/core-accounting-and-resolution/review.md)
separates model proofs, exercised correspondence, red/green regression evidence,
fuzz bounds and measured timings.[^core-design-review]

[^core-design-review]: [Core assessment and staged reviews](../work/core-accounting-and-resolution/review.md).


# Scoped policy follow-up evidence

Public fixed/generated tests check the indexed policy against integer
identity/hash selectors: additive Boolean permissions, original exception IDs
and order, normalized identities, custom proof requirements, hashless observed
content, all-OR-unit usage and clarification warnings. The unchanged Lean corpus
and strict adapter retain their original scope; no model theorem about the new
private map is asserted. Synthetic timing samples and limits are in the
[policy measurement record](../work/policy-evaluation/performance.md).[^policy-evaluation-review]

[^policy-evaluation-review]: [Policy evaluation assessment and staged review](../work/policy-evaluation/review.md).


# Permission unit invariants

Public constructor regressions and 256-case generated checks cover atomic
validation, custom WITH scope/proof requirements and canonical Eq/Ord/Borrow
coherence. Positive catalogue sampling follows the existing strict parser's
supported subset; fixed negatives retain rejected GNU-plus spellings. These
checks do not establish independent parser correctness or expand the existing
Lean model. The [measurement record](../work/permission-units/performance.md)
separates the type guarantee from timing observations.[^permission-unit-review]

[^permission-unit-review]: [Validated permission unit assessment and reviews](../work/permission-units/review.md).


# Immutable payload correspondence

Named archive/sidecar/hashless cases preserve exact rebinding outcomes and error
codes. A 256-case generated cloning property preserves declarations, future-minor
warnings and proof strength. Existing cache corruption, bounded expansion and
concurrent-publication tests retain wire/validation coverage. Stable and shuffled
suites, metadata/inspection fuzz and unchanged Lean corpus adapters provide
separate finite evidence. Arc allocation, serde implementation and performance
are outside the Lean theorem claims; the staged record identifies those limits
and the fresh whole-PR reviewer.[^evidence-ownership-review]

[^evidence-ownership-review]: [Immutable payload staged review](../work/evidence-ownership/review.md).


# HTML streaming preservation

Named HTML cases preserve base encounter order, raw-text/RCDATA suppression,
late version errors, duplicate target rejection and attribute precedence. A
256-case generated property derives expected absolute URLs independently from
base encounter order and irrelevant history. The unchanged index model adapter
still exercises both representation parsers; it does not prove html5ever or
arbitrary browser semantics. Existing metadata/inspection fuzz remains separate
core regression evidence and does not reach the binary-private HTML sink. The
staged report records those boundaries and measured synthetic parser costs.
[^html-index-filtering-review]

[^html-index-filtering-review]: [HTML index staged review](../work/html-index-filtering/review.md).
