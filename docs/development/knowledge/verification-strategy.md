---
type: Decision
title: Formal models and generated behavioral tests
description: Use Lean oracles, property tests, fuzzing, and typed Rust boundaries without conflating their guarantees.
status: accepted
sources:
- id: owner
  resource: ../work/initial-design/implementation-approval.md
  working_tree: untracked
  sha256: 1593dc51b5663e513152059e50b39795063a696bf3b6b417c9a5c8fae959834c
- id: hoimin-model
  resource: https://github.com/HayaoSuzuki/hoimin/blob/542ec59c002d72fdff0137aa90113ffa59d70844/formal/HoiminOracle/HoiminOracle/MutationScoreExitPolicyModel.lean
- id: hoimin-generator
  resource: https://github.com/HayaoSuzuki/hoimin/blob/542ec59c002d72fdff0137aa90113ffa59d70844/formal/HoiminOracle/MutationScoreExitPolicyAuditMain.lean
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
