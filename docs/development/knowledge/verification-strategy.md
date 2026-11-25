---
type: Decision
title: Formal models and generated behavioral tests
description: Use Lean oracles, property tests, fuzzing, and typed Rust boundaries without conflating their guarantees.
status: accepted
sources:
- id: quint-release-reservation
  resource: ../../../formal/quint/release_reservation.qnt
  working_tree: untracked
  sha256: 8ca8bcf46219cefd1d3902012e0562845a7815426e65d2ac28055496ff35a1e8
- id: quint-release-reservation-corpus
  resource: ../../../tests/fixtures/oracle/release-reservation.jsonl
  working_tree: untracked
  sha256: c67927134459f6cbcc4c133c625710c2ea7683680c09768e02f8e97ad4c1529a
- id: quint-release-reservation-replay
  resource: ../../../tests/test_release.py
  working_tree: modified
  sha256: 1230a1b386012ce6ab96668714e3066f77c0bf26720b5d6dba4063cbf7185341
- id: quint-http-routing
  resource: ../../../formal/quint/http_routing.qnt
  working_tree: untracked
  sha256: d4c27c22baee4185756574eee5e19d461a4ba2fb937be7563da9282da96ad8f3
- id: quint-http-routing-corpus
  resource: ../../../tests/fixtures/oracle/http-routing.jsonl
  working_tree: untracked
  sha256: c1f36893c2c716a411dc4ca3f2cae4275f0fd69bfc890b67d6601bde0aac9cea
- id: quint-http-routing-replay
  resource: ../../../src/acquisition/http_tests/http_routing.rs
  working_tree: untracked
  sha256: 441d1358ff26c140e343d2a343aeabf368c219bc7b4b0878ecb4c3a2b086a65e
- id: quint-legacy-scan
  resource: ../../../formal/quint/legacy_scan.qnt
  working_tree: untracked
  sha256: 8f95b64e8404edef85b4c1003064d9669f72e55e424c286be4d105da21cf5f1a
- id: quint-legacy-scan-corpus
  resource: ../../../tests/fixtures/oracle/legacy-scan.jsonl
  working_tree: untracked
  sha256: 9dde7dd0b1631af89d388dfff67e2091b42955a3ca511e27ec682e8e75d6d203
- id: quint-legacy-scan-replay
  resource: ../../../src/acquisition/http_tests/legacy_scan.rs
  working_tree: untracked
  sha256: d56893590bc9f815bbf4043aa08901a4b57992912669b3c15b262c659c2875c4
- id: quint-sidecar-fallback
  resource: ../../../formal/quint/sidecar_fallback.qnt
  working_tree: untracked
  sha256: ed98d1532485bb740e86a0ca6b9f04f5f4b9fd86a83aa68e737bad413ed2bb43
- id: quint-sidecar-fallback-corpus
  resource: ../../../tests/fixtures/oracle/sidecar-fallback.jsonl
  working_tree: untracked
  sha256: a8407cdf781359bf076991d8db6a4d02e7a2d4033163559c26ba95eac9c10472
- id: quint-sidecar-fallback-replay
  resource: ../../../src/acquisition/http_tests/sidecar_fallback.rs
  working_tree: untracked
  sha256: d01e3f02df72089b6ceaca3db664b768991790842c96c878f569df2f6d6945ec
- id: quint-description-evidence
  resource: ../../../formal/quint/description_evidence.qnt
  working_tree: untracked
  sha256: 28f02f676a3ef796c337e4ca5cfc1d8f1afa84da8ab72736cdb0eaa53de6c332
- id: quint-description-evidence-corpus
  resource: ../../../tests/fixtures/oracle/description-evidence.jsonl
  working_tree: untracked
  sha256: d5e3529c40bbe25f1dbc163569de2929bc16049587cbe2abf0c6093c4fe184e8
- id: quint-description-evidence-replay
  resource: ../../../tests/description.rs
  working_tree: modified
  sha256: e722d2661d19ca87f28325e53606433634afd7c230b1cdb255a528e2ee7abd2d
- id: quint-unused-policy
  resource: ../../../formal/quint/unused_policy.qnt
  working_tree: untracked
  sha256: 9e17530e47d4abc55fd886307491ba120912e17c021406169c581f733ba98e0f
- id: quint-unused-policy-corpus
  resource: ../../../tests/fixtures/oracle/unused-policy.jsonl
  working_tree: untracked
  sha256: a1525b355fa7250c1bc5b129a0147c1fa8845ef78a0a44f8a0429b1b400712ca
- id: quint-unused-policy-replay
  resource: ../../../tests/policy_usage.rs
  working_tree: untracked
  sha256: d66ce2e024c2a9c8fee05499ba2ce4c819e1b361ae5a78d9d41ce4515fd9501a
- id: warning-report-order
  resource: ../../../src/report.rs
  revision: c1686812868abd649d0b55986ab63c8734d397d9
- id: quint-scoped-policy
  resource: ../../../formal/quint/scoped_policy.qnt
  working_tree: untracked
  sha256: cb1132f1c6025c4ee76600f4856a9d4bef89f38fec0648144256382af35f4cf5
- id: quint-scoped-policy-corpus
  resource: ../../../tests/fixtures/oracle/scoped-policy.jsonl
  working_tree: untracked
  sha256: d787a9d967d211029df6bbe632644c433eed9081147a0d8c9ff4b04d447fe8da
- id: quint-path-ties
  resource: ../../../formal/quint/path_ties.qnt
  working_tree: untracked
  sha256: dbb198326220eb393a66cdfcfcee3f087fcc4df22e9d29dbb8b7c3743281d458
- id: quint-path-ties-corpus
  resource: ../../../tests/fixtures/oracle/path-ties.jsonl
  working_tree: untracked
  sha256: f6622b883310310afb316b314409cd905ba2adbf5b23175fc7b81829b37c03ff
- id: quint-locked-forks
  resource: ../../../formal/quint/locked_forks.qnt
  working_tree: untracked
  sha256: 913e845a10bb9aa62f5d50c8f8a4dd0d7241caa6c8ddaf146b7b5b05f7423778
- id: quint-locked-forks-corpus
  resource: ../../../tests/fixtures/oracle/locked-forks.jsonl
  working_tree: untracked
  sha256: 977c287f42440a111f94a90bee9c7b71cc0d96dbaa4b18c95aed8d202e151a4f
- id: quint-locked-forks-replay
  resource: ../../../tests/resolution_matching.rs
  working_tree: modified
  sha256: d64c47695208e4d16596c256758af8bac3e7210646fabf0091abe3e6746d1b36
- id: quint-extras
  resource: ../../../formal/quint/extras.qnt
  working_tree: untracked
  sha256: 3153085c191fb85df6ea0c943a70fee29cf984af9d891ee0cfd273e6f8e774d8
- id: quint-extras-corpus
  resource: ../../../tests/fixtures/oracle/extras.jsonl
  working_tree: untracked
  sha256: f467e1713e2af9b51857f7da68c15dff368e9cb6feaf44abd2a5af4620151c1e
- id: quint-extras-replay
  resource: ../../../tests/resolution_paths.rs
  working_tree: modified
  sha256: 2a7ee4a5320f899239c68337cd37c66b4aba59651c3b012414339b0295179198
- id: generated-resolution-graphs
  resource: ../../../tests/resolution_graphs.rs
  working_tree: untracked
  sha256: 31608d07d7324c67183380cc7a318b8ce5e5eadf13af19a701a52aff3c74fff4
- id: quint-shared-cache
  resource: ../../../formal/quint/shared_cache.qnt
  working_tree: untracked
  sha256: 982a45bce248a62edfb8685287bd8658612fadbafdcec66a37870957e804a5b1
- id: quint-shared-cache-replay
  resource: ../../../src/acquisition/http_tests/cache_interleaving.rs
  working_tree: modified
  sha256: 5f8b21b8c105cafa614accb44562a735ddd7a911afdaf7a19a0fb775235e81db
- id: quint-shared-cache-corpus
  resource: ../../../tests/fixtures/oracle/shared-cache.jsonl
  working_tree: untracked
  sha256: 053a6ef527010ce06054ba77d41df692736511bcc9b00c419629127a722b5ae3
- id: quint-connect-dependency-policy
  resource: ../../../deny.toml
  working_tree: modified
  sha256: bcc6c0ec7f22c06472c9cf4f7fc1bcd4d893fb030764e49836f456d1c7258f50
- id: quint-index-concurrency
  resource: ../../../formal/quint/index_concurrency.qnt
  working_tree: untracked
  sha256: bd4e761c0ac10a50becf806db6ff258b1508ce2225cd6762369c139b5688bb9d
- id: quint-lifecycle
  resource: ../../../formal/quint/lifecycle.qnt
  working_tree: untracked
  sha256: 5ccfe6f0fcc0e131c14b38d3d82f04fca09d152835c0a0d503ec920913cfd6aa
- id: quint-replay
  resource: ../../../src/acquisition/http_tests/lifecycle.rs
  working_tree: modified
  sha256: 38dd2c936ce7303d3ac3d97a343799fa89367018d2a9259941fafd31e635f60f
- id: quint-lifecycle-corpus
  resource: ../../../tests/fixtures/oracle/lifecycle.jsonl
  working_tree: untracked
  sha256: 1965be615b084cc86338c2fb18d050adca18d7e1df574ca596bd9cfd7a297be6
- id: quint-inspection
  resource: ../../../formal/quint/inspection.qnt
  working_tree: untracked
  sha256: 0bbc899d9964e3894ae14a77264ef7426412cae371b2b0be1ec8d9544994f685
- id: cli-verdict-property
  resource: ../../../tests/cli.rs
  revision: e8bd7490cd3c33af612077ec9384b60c8fa1c744
- id: acquisition-route-fixtures
  resource: ../../../src/acquisition/http_tests.rs
  revision: e8bd7490cd3c33af612077ec9384b60c8fa1c744
- id: quint-observation
  resource: ../../../formal/quint/observation.qnt
  working_tree: untracked
  sha256: 5ae3d505eaa24543d10127bb7460f0232816649ae8f88d2ccd79c8fd5e8d23fe
- id: quint-acquisition
  resource: ../../../formal/quint/acquisition.qnt
  working_tree: untracked
  sha256: 320ed8f4658e3929136a57f889786ade4a7221115c7416d7d178f5ffd1478091
- id: observation-traces
  resource: ../../../crates/taloon-decisions/tests/observation.rs
  working_tree: modified
  sha256: 3c834ee035029629f19af1850f8ed128c9035c103ced34930366b9cf34d0b7b7
- id: inspection-exit-property
  resource: ../../../tests/inspection.rs
  working_tree: modified
  sha256: 4bef33840a259b688562ffde10d2d583ce855275aae5eaf35b048b843a343baa
- id: quint-model
  resource: ../../../formal/quint/aggregation.qnt
  working_tree: untracked
  sha256: 7fe1c23e378edb1f34bef3522be9fb670be65388b7639d789d04f8771fa682f5
- id: quint-correspondence
  resource: ../../../crates/taloon-decisions/tests/aggregate.rs
  working_tree: modified
  sha256: 78b748c068101082d140893fc4adb0c07625c2045f81a0e831d951c3248e55b3
- id: allpair-quint
  resource: https://github.com/HayaoSuzuki/pytest-allpair/blob/369f173a32a83c0982b542b298a270988b5d1107/formal/quint/README.md
- id: python-mutation-verification
  resource: ../work/coverage/python-mutations.md
  working_tree: untracked
  sha256: 7664d6c921323244d0c61423cbcf35e75d047afa77690919bac011e7a283b818
- id: coverage-verification
  resource: ../work/coverage/review.md
  working_tree: untracked
  sha256: 50a8292678cdccfb7aa75734f1db6078eb19a5b4def894335a4734ab56c6315b
- id: coverage-configuration
  resource: ../../../pyproject.toml
  working_tree: modified
  sha256: 7dbcbc24d9891e0cdfb6b84512c0c6591054fd85ea791544f4c32cafb6c2098b
- id: creusot-image-adoption
  resource: ../work/creusot-registry-image/adoption.md
  working_tree: untracked
  sha256: 20b729bc1a80d6a99cb54c50f2e5b6c437608a05ed4b549acb09adc434c00adf
- id: creusot-registry-image
  resource: ../work/creusot-registry-image/review.md
  working_tree: modified
  sha256: f0ad79e29e87ea05c30ad250014d7cbc220f791b8e9f4683378c3009a1821ec6
- id: creusot-image-cache
  resource: ../work/creusot-image-cache/review.md
  working_tree: untracked
  sha256: 876251b68000ccddcacfc0be45a52f4488847ef87197e69976411882c549f9b9
- id: actionable-diagnostics-review
  resource: ../work/actionable-license-diagnostics/review.md
  working_tree: untracked
  sha256: f56ab86f925d5d2149ce47181fcafa15a434ecfd1fe18698fda05d0294e86473
- id: constrained-supplement-review
  resource: ../work/constrained-license-supplementation/review.md
  working_tree: untracked
  sha256: d1ff5e2be3367bada1c6a635b996e5b1ee9ac6967831400aeceebc4677d7b252
- id: creusot-build-retry
  resource: ../work/creusot-build-retry/review.md
  working_tree: modified
  sha256: d880aeb4d56637c80705be6a8a039d183d8f2896a7a17a9ce81d260af8735660
- id: legacy-license-review
  resource: ../work/legacy-license-interpretation/review.md
  working_tree: untracked
  sha256: dca68bb0997c79a7bf25ba5244afade94b4dc42a9e74b4b12e8976ba03982090
- id: bundled-license-review
  resource: ../work/bundled-license-evidence/review.md
  working_tree: untracked
  sha256: ed82b8bf06d4d89c63c8b074f3e1b400dfc3422bf7c7d6458a73ddc146c1b359
- id: expanded-contract-review
  resource: ../work/evidence-and-invariants/review.md
  working_tree: untracked
  sha256: 5697f25cb4816577b1eb311decb1294f6eef59d300f0479c1f16a38e59f239b4
- id: contract-aggregate
  resource: ../work/evidence-and-invariants/aggregate.md
  revision: 4b00c6b
- id: contract-name
  resource: ../work/evidence-and-invariants/name.md
  revision: 5481a60
- id: contract-accounting
  resource: ../work/evidence-and-invariants/accounting.md
  revision: 7f5e4c3
- id: contract-evidence
  resource: ../work/evidence-and-invariants/evidence.md
  revision: 7dfc332
- id: contract-graph
  resource: ../work/evidence-and-invariants/graph.md
  working_tree: untracked
  sha256: 12dd36c6089914719bc34714368000c4c8d0b342091150c6257daacc3e7d5597
- id: creusot-contracts
  resource: ../work/creusot-contracts/review.md
  working_tree: modified
  sha256: ff933df572ba1c9e7a8b88d53a0d7c5e3db628a83eccf7c8fec9bab79c44a0c2
- id: typed-acquisition-review
  resource: ../work/typed-acquisition/review.md
  working_tree: modified
  sha256: 00daddabad915d492d1583acaff483a254849c01dccbf3ddb26de235cdc30605
- id: terminal-text-review
  resource: ../work/terminal-text/review.md
  working_tree: untracked
  sha256: cb3282d33446737e80e3432c30dd0710a7624ef4f306b61aa64d91127ada745e
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

Pin Lean to v4.36.0, with its Linux release archive checksum in CI. Start suspect individual Lean
commands with an external 20-second deadline, one process at a time; preserve
heartbeat limits. Record finite case counts, time and memory when available.
Replace expensive enumeration with inductive lemmas instead of enlarging bounds
after a sharp cost increase. Start fuzz smoke runs at 1,000 executions and
4,096-byte parser inputs on nightly-2026-11-23. Record larger campaigns and
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


The generated graph checks exercise the public inspection report across two
workspace roots, workspace and root dependency groups, and root extras.
Each generated inventory includes an unreachable external record, and the
expected request set comes from the generated records. For each external
target, an independent reverse distance calculation over package/extra states
determines shortest paths. Tie order, activated extras, dependency markers
and package markers are checked at the diagnostic boundary. Markers remain
annotations, while extras affect traversal. This complements the fixed
same-name fork and origin-resolution cases.[^generated-resolution-graphs]

[^core-design-review]: [Core assessment and staged reviews](../work/core-accounting-and-resolution/review.md).
[^generated-resolution-graphs]: [Generated inspection graphs and reverse distance oracle](../../../tests/resolution_graphs.rs).


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


# Terminal text ownership and output

Named fixed cases and a 256-case Unicode property compare terminal output with
an independent numeric-scalar escape oracle. Separate Cow::Borrowed cases pin
unchanged-text ownership; they initially rejected the previous String type.
That type-level RED is an allocation-interface claim, not a previous wrong-output
claim. Existing process tests check rendered report behavior. Stable/shuffled
suites retain core/model correspondence; Lean and metadata/inspection fuzz do
not prove or exercise this binary-private helper. The stage record separates
executor results, fresh reviewer scope and synthetic timing limits.
[^terminal-text-review]

[^terminal-text-review]: [Terminal text staged review](../work/terminal-text/review.md).


# Typed acquisition regression evidence

Integer-label properties independently check worker coverage/affinity/bounds and
modern/legacy digest selection (256 cases each). Runtime RED/GREEN covers empty
scope and malformed/duplicate known fields. Independent whole-PR review found
one Important positional-array acceptance gap; six object-boundary cases and
one valid-digest-array HTTP case failed before the map-only fix and passed after.
The complete stable/generated/seeded suites passed 305 local tests afterward.
No second independent approval of the fix is claimed. Existing Lean adapters and
core fuzz retain their original scope; they do not prove serde wire shape,
worker load balance, cryptographic collision freedom or allocation performance.
The staged record separates the reviewer finding, executor fix pass, final
measurements and subsequent CI observation.[^typed-acquisition-review]

[^typed-acquisition-review]: [Typed acquisition staged review](../work/typed-acquisition/review.md).


The completion audit adds two named public hash-only lookup cases: the old
signature rejected them with E0061, and the simplified API passes. Full stable,
generated-shuffle and seed-12345 suites each pass 307 local tests. A separate
fresh completion reviewer confirms current ownership boundaries and identifies
no further required implementation refactor; it does not certify unobserved
executor or CI results.[^typed-acquisition-review]


# Direct Rust decision contracts

The recorded Creusot 0.14.0 baseline verifies the production bodies in `taloon-decisions`, including
observation-slot transitions, status precedence/exit decisions and the borrowed
report scan. The scan proves exact aggregate precedence, input-error handling,
empty input and early termination on incompleteness; root diagnostic/warning
classification and serialization remain separately tested adapters. No extra
status vector or cloned package payload is introduced.[^creusot-contracts][^contract-aggregate]

The production byte normalizer proves accepted ASCII name syntax, lowercase and
separator-collapse semantics, canonical output, length bounds, loop termination
and fixed-point/idempotence. Public string adapters retain their error codes.
Acquisition merges prove exact saturation and monotonicity for all twenty
counters while preserving elapsed time. Measuring clocks/network traffic is
outside those arithmetic contracts.[^contract-name][^contract-accounting]

Evidence contracts compare actual 32-byte identities: archive strength satisfies
both requirements, index evidence cannot satisfy archive verification, index
source and request must both match, pinned content must match exactly, and
hashless evidence cannot be rebound to another request. Sidecar construction
requires an advertised pinned digest. Production archive/sidecar/cache adapters
call these rules. Cryptographic collision resistance, metadata parsing, cache
schema/checksum validation and fresh hashing stay outside the proof; no signature
or whole-Evidence correctness claim follows.[^contract-evidence]

Full dependency exploration remains deferred with owner permission. Real ordered
map/set and borrowed visited-state probes lack contracts in the pinned verifier;
positive scalar and VecDeque FIFO controls prove. Keep the current efficient
traversal and existing Lean selection/independent graph properties, rather than
introduce trusted map axioms, duplicate graphs or a proof-driven replacement
algorithm. Reconsider when upstream contracts or a separately justified data
representation improvement supports the actual traversal.[^contract-graph]

The separate pinned compiler/Why3/solver environment is development-only. The
existing required formal CI context checks an exact 21-file/43-goal inventory on
three target architectures, with fresh generation, no proof cache and input
snapshots. Fourteen compiling mutations must fail their own named goals; these
test sensitivity, not all possible defects. No custom trusted application
function or map axiom is introduced. Compiler, solver and standard-library
contracts remain trusted; cross-target proofs inside Linux do not establish
native OS execution. Existing Lean proofs/corpora retain their original scope.
Final integration checks and independent-review fixes are recorded separately.
[^expanded-contract-review]
Commands, limits and CI deadlines belong to the [Creusot procedure](../../../formal/creusot/README.md).
Revisit contracts when production decisions or the pinned environment changes.
[^creusot-contracts][^contract-aggregate][^contract-name][^contract-accounting][^contract-evidence]

[^creusot-contracts]: [Initial adoption and five-pass reviews](../work/creusot-contracts/review.md).
[^contract-aggregate]: [Actual report loop and reviews](../work/evidence-and-invariants/aggregate.md).
[^contract-name]: [Name semantics, fixed point and reviews](../work/evidence-and-invariants/name.md).
[^contract-accounting]: [Counter/elapsed contracts and reviews](../work/evidence-and-invariants/accounting.md).
[^contract-evidence]: [Evidence binding contracts and reviews](../work/evidence-and-invariants/evidence.md).
[^contract-graph]: [Ordered-container feasibility and deferral](../work/evidence-and-invariants/graph.md).

[^expanded-contract-review]: [Expanded contracts final integration and limits](../work/evidence-and-invariants/review.md).

# Bundled-file interpretation checks

Named and generated cases exercise complete-template matching, changed/additional
terms, namespace/path attacks, bounded ZIP/tar reads, evidence-file restoration
and explicit declaration precedence. HTTP fixtures pin selective fallback and
warm/offline request counts; a public sqlparse 0.6.0 smoke exercise uses both exact
locked distribution hashes. This does not extend the Creusot or Lean proof scope
to license recognition, archive parsers or legal completeness. Retained
attribution files are not semantically proved to lack additional terms.
[^bundled-license-review]

[^bundled-license-review]: [Bundled-file tests, measurements and staged review](../work/bundled-license-evidence/review.md).

# Legacy metadata correspondence checks

Named public tests exercise precise legacy names, complete metadata texts,
strict expression precedence, repeated fields, unchanged proof strength and raw
cache restoration. Generated cases vary whole-name casing, additional terms and
Unicode header offsets. LF/CRLF archive/sidecar regressions distinguish separate
restrictions from multiple copyright notices; they rejected an intermediate
unfolded-text interpretation. HTTP fixtures pin zero extra archive calls and
network-free cached reinterpretation. Expanded notice grammar retains earlier
extra-sentence attacks and adds independent generated structures. These are
behavioral checks; existing Lean/Creusot claims do not expand to text matching,
metadata parsing or legal equivalence.[^legacy-license-review]

[^legacy-license-review]: [Legacy interpretation red/green and review evidence](../work/legacy-license-interpretation/review.md).

# Pinned environment acquisition recovery

Image construction retries once using the exact same verified recipe/context and
architecture parameters. Only the Docker build is retried; source/checksum gates
and all downstream proofs remain mandatory. Existing build/job deadlines cover
both attempts. Synthetic orchestration and rejected mutations check the wrapper;
actual hosted image/proof success is separate evidence. Reconsider on upstream
recipe changes or sustained service outages; do not substitute unpinned sources
to make a required context pass.[^creusot-build-retry]

[^creusot-build-retry]: [Observed acquisition failures and bounded retry review](../work/creusot-build-retry/review.md).

# Constrained legacy supplementation

Named wheel/sdist cases and generated extra-term guards exercise whole-value
eligibility, family agreement, strict declaration priority, conflicting files,
raw-record replay and interpretation origin. HTTP fixtures check single archive
escalation, fresh archive metadata authority over sidecar hints, zero-network
positive/negative warm reuse and offline pending observations. Real process tests
check both commands, compliant text visibility and evidence origin surviving a
separate policy clarification. This is behavioral coverage, not an extension of
Lean/Creusot claims to metadata semantics, archive readers or legal equivalence.
[^constrained-supplement-review]

[^constrained-supplement-review]: [Constrained supplementation staged review](../work/constrained-license-supplementation/review.md).

# Diagnostic causes and grouped presentation

Public sidecar/archive cases pin branch-owned causes, authoritative-field
precedence and cache reinterpretation. A generated repeated-field property uses
an independent cardinality expectation. Real CLI tests cover both commands,
summary/verbose/JSON, acquisition versus declaration failures, distinct
clarifications and credential/control-safe previews. A generated boundary case
requires declarations differing after the preview to remain separate. Unit
presentation cases additionally vary proof and status. Stable/shuffled suites,
bounded metadata/wheel/sdist fuzz and fresh-context review provide behavioral
evidence; this adds no formal proof of parsing, classification or rendering.
[^actionable-diagnostics-review]

[^actionable-diagnostics-review]: [Actionable diagnostic assessment and staged review](../work/actionable-license-diagnostics/review.md).

# Exact-key formal tool image cache

Sustained signed-snapshot acquisition failures can outlast both bounded build
attempts and correctly prevent release publication. In explicit bootstrap mode, CI reuses a serialized
amd64 tool image by exact runner/format/builder/preparer key, including the
checksum-pinned recipe identity. GitHub repository/ref cache isolation is an
explicit trust boundary; no prefix lookup or cross-repository image is used.
Both restored and newly built images pass architecture and installation checks;
failed preparation cannot save a cache. All current-source preflights, fresh
proof inventories/goals and compiling mutations remain mandatory. No proof
results or application inputs are cached. An initial cache miss still depends on
the pinned upstream, and earlier failed main runs still need their own successful
rerun before the unchanged release gate can pass. Reconsider key inputs whenever
the recipe, preparation or platform assumptions change.
[^creusot-image-cache]

[^creusot-image-cache]: [Observed outage, cache boundary and staged verification](../work/creusot-image-cache/review.md).

# Attested durable formal environment

A committed three-field pin selects explicit bootstrap or an own-repository
GHCR image at an exact SHA-256 digest, with source commit and current builder
hash. Missing, malformed or stale pins fail. The builder labels its own bytes;
consumption verifies OCI provenance for repository, trusted-main publisher,
source/signer commit and hosted runners before pulling, then checks image builder
identity and installation. A failed registry operation cannot select bootstrap:
mode classification is separate from external exit statuses. Bootstrap alone
uses the existing bounded builder/cache. All project preflights, fresh proofs
and mutations remain mandatory; no source outcomes are cached.

The trusted-main environment workflow builds/pushes/attests actual image bytes,
verifies the result and uploads a candidate pin without modifying the repository.
Adoption is a reviewed normal commit. The first pin now adopts the actual digest from successful trusted-main
publisher run 41450013770, binding its merged source and unchanged builder hash.
Normal source CI selects registry mode; bootstrap remains an explicit option
for future rotations. Initial environment publication still depends on upstream
availability.[^creusot-image-adoption] GHCR publication and local
synthetic tests are distinct evidence. This reduces ordinary proof jobs' snapshot
acquisition dependency, not all network or trusted-service dependencies.
[^creusot-registry-image]

[^creusot-registry-image]: [Durable image design, provenance boundaries and staged verification](../work/creusot-registry-image/review.md).

[^creusot-image-adoption]: [First published digest, source receipt and adoption checks](../work/creusot-registry-image/adoption.md).

# Source and test coverage

Coverage measurement includes maintained source and test code. Python uses
pytest-cov with child-process instrumentation and a 100% line threshold. Rust
uses cargo-llvm-cov with its default test-file ignore disabled; execute ignored
measurements and loopback tests as well as the ordinary suite. The
[development guide](../index.md#coverage) owns prerequisites and commands.
[^coverage-configuration]

Require meaningful observable expectations, not calls that merely execute lines.
An exclusion belongs inline with the invariant that makes the path unreachable,
or the reason a test failure guard cannot execute in a passing suite. Do not
exclude an entire file or reduce the denominator to hide testable failure paths.
Rust's nightly-only coverage attributes retain defensive checks in ordinary
builds. A single-platform run covers only the code compiled for that platform;
100% line coverage is not branch coverage or proof of semantic correctness.
The [verification record](../work/coverage/review.md) distinguishes merged mapped
source-line coverage from LLVM's native instantiation summary, which can disagree
with its own line records. Preserve both metrics; the gate checks actual mapped
source lines rather than changing LLVM's raw totals.[^coverage-verification]

[^coverage-configuration]: [Python dependency lock inputs and coverage configuration](../../../pyproject.toml).

[^coverage-verification]: [Source and test coverage verification](../work/coverage/review.md).

# Python mutation verification

Selected hoimin mutations complement line coverage by exercising assertions about
complete diagnostics, navigation, signed evidence, release identity and failure
propagation. The [verification record](../work/coverage/python-mutations.md)
retains the selected IDs, source fingerprints, replans and latest per-ID outcomes.
Equivalent mutants remain visible. Selected verification does not establish an
exhaustive mutation score or change the formal proof scope.
[^python-mutation-verification]

[^python-mutation-verification]: [Python mutation verification](../work/coverage/python-mutations.md).

## Creusot 0.15 environment refresh

The [third-party refresh](../work/third-party-refresh/review.md) tracks the
coordinated switch to Creusot 0.15.0, its standard contracts and Z3 5.3.2.
The prior image pin cannot be reused with the new builder. The [published image adoption](../work/third-party-refresh/image-adoption.md)
records the successful trusted-main publisher and exact adopted identity.
Proof generation, the full obligation inventory and negative witnesses remain
required; updating tool versions alone does not establish their results.

# Finite aggregation model

Following pytest-allpair's pruning check, use Quint with TLC for a finite
transition graph, an independent invariant, and a deliberately broken variant.
Aggregation was the first model because the public Rust function accepts all
inputs needed to represent every abstract state.[^allpair-quint][^quint-model]

The Rust model-based test covers one canonical package list and flag for each
reachable event set. Keep the existing proptest for list order and multiplicity.
Lean and Creusot retain their existing proof roles. A second hand-maintained
TLA+ model would duplicate the Quint specification. Trace export and replay
would add encoding and tooling for a state space whose representatives fit
in a small exhaustive test. Reconsider trace replay if a future model has
history-dependent observations or models a real multi-step API.
[^quint-correspondence]

See the [reproduction procedure](../../../formal/quint/README.md).

[^quint-model]: [Quint model and negative control](../../../formal/quint/aggregation.qnt).
[^quint-correspondence]: [Rust correspondence and proptest](../../../crates/taloon-decisions/tests/aggregate.rs).
[^allpair-quint]: [pytest-allpair's finite pruning check](https://github.com/HayaoSuzuki/pytest-allpair/blob/369f173a32a83c0982b542b298a270988b5d1107/formal/quint/README.md).

# Verification across taloon

Add finite observation and remote-acquisition models where the order of
events or routing affects the result. For observation, compare each consume
result to the records since the preceding consume. For acquisition, model
routes after identity and evidence strength have already been checked. The
concrete transport and cache fixtures supply correspondence for selected routes.
[^quint-observation][^quint-acquisition][^observation-traces][^acquisition-route-fixtures]

The inspection model combines two targets' observation histories, policy
permission, warning presence, and warning escalation into an exit code.
The Rust adapter covers all 128 input combinations through the public API,
and proptest varies order and multiplicity. The CLI process property checks
the exit value against the JSON report with real local archives.
[^quint-inspection][^inspection-exit-property][^cli-verdict-property]

Check the complete application using the layers in the table.

| Boundary | Checks |
| --- | --- |
| Lockfile and target resolution | Rust integration tests, generated graph properties and lockfile fuzzing |
| Metadata, archives and content identity | Rust fixtures, parser fuzz targets and Creusot evidence bindings |
| Acquisition and offline cache | Quint routing, Rust transport/cache fixtures and CLI process tests |
| Observation, inspection and policy | Quint observation, Lean corpora, Creusot contracts and Rust model-based/property tests |
| Report, description and exit | Quint aggregation and inspection exit, Lean selection corpora and Rust report and CLI integration tests |
| Packaging, release and repository tooling | Python tests including binary smoke, release properties and workflow and documentation checks |

This coverage spans the main execution paths and development tooling. The
exhaustive claims apply to the documented finite models. Revisit the
acquisition abstraction when a route, cache admission rule or strength
requirement changes. Expand it to include scheduling only when the intended
interleavings and observable outcomes can be expressed independently.

[^quint-observation]: [Observation transition model](../../../formal/quint/observation.qnt).
[^quint-acquisition]: [Remote acquisition routing model](../../../formal/quint/acquisition.qnt).
[^observation-traces]: [Rust observation traces and property test](../../../crates/taloon-decisions/tests/observation.rs).
[^inspection-exit-property]: [Public inspection and exit-status property](../../../tests/inspection.rs).
[^acquisition-route-fixtures]: [Rust acquisition route fixtures](../../../src/acquisition/http_tests.rs).
[^quint-inspection]: [Inspection and exit model](../../../formal/quint/inspection.qnt).
[^cli-verdict-property]: [CLI process and JSON exit property](../../../tests/cli.rs).

# Acquisition-to-report lifecycle

Use a single Quint transition for one completed inspection on a bounded
two-package graph. Carry validated archive digests across checks so that
retries, failures, lock changes, offline runs and explicit cache clearing
constrain later observations. The invariant connects exact cache membership
to package completeness and report precedence. A negative control keys
the child cache lookup too broadly, so TLC can show an older digest masking an
offline miss.[^quint-lifecycle]

Generate fixed-seed ITF traces from the verified model and compare a
normalized corpus byte-for-byte in CI. Replay through the binary's
acquisition layer and the library's inspection API with a disk cache and
scripted transport. Resolve both packages from uv.lock across three graph shapes and observe
the shortest diagnostic path on failures. This checks the wiring between model
boundaries while the smaller models retain their independent checks. A
separate Lean trace generator would duplicate the transition rules.
Revisit the domain when adding cache admission behavior, retry policy, or
new report precedence rules. Add concurrent schedules when the contract
specifies how overlap affects observations.[^quint-lifecycle-corpus][^quint-replay]

In the same CI job, Quint Connect replays additional generated histories.
proptest varies the Quint seed, dependency topology, and wheel content.
Compare package status, report exit, cache hits, archive attempts, and
exact digest membership after each check. Keep the cache across checks
within each trace.[^quint-replay]

[^quint-lifecycle]: [Finite acquisition-to-report model](../../../formal/quint/lifecycle.qnt).
[^quint-replay]: [Rust trace adapter](../../../src/acquisition/http_tests/lifecycle.rs).
[^quint-lifecycle-corpus]: [Versioned model-generated traces](../../../tests/fixtures/oracle/lifecycle.jsonl).

# Concurrent package-index initialization

Model contention within a single index entry as an exclusive owner,
an initially empty snapshot, and two worker outcomes. A completed load
publishes a result that waiters reuse, including terminal errors. A panic
relinquishes ownership without publishing. The invariant distinguishes
publication count from initializer attempts. Verify outcome correspondence
with the existing transport fixtures for successful sharing, shared error
with independent archive verification, and panic recovery. Revisit the
model if an entry permits invalidation, cancellation, or replacement
while a batch is running.[^quint-index-concurrency][^acquisition-route-fixtures]

[^quint-index-concurrency]: [Concurrent index-initialization model](../../../formal/quint/index_concurrency.qnt).

# Shared disk-cache interleavings

Extend the acquisition lifecycle with a separate model of archive
publication and deletion. Only a published digest may satisfy a fresh
lookup. An opened file remains readable across unlink, and an in-flight
writer can publish after a completed clear. The latter is allowed by the
existing cache clear contract. Treat clear as an operation between
checkpoints, with two digest-specific writers and one retained reader.
[^quint-shared-cache]

Generate and replay fixed traces through concurrent Rust acquisition batches
sharing a disk cache. Test-only hooks pause the actual cache writer and reader
at the filesystem boundaries. Compare archive identity and report status as
well as what each step leaves available offline. Keep the existing tests for
same-digest replacement and damaged records. Revisit the abstraction if
locking, cache-key semantics, file publication, or clear guarantees change.
[^quint-shared-cache-replay][^quint-shared-cache-corpus]

Use Quint Connect in the formal CI job to replay additional generated
traces. proptest varies the Quint seed and the archive bytes. Project the
actual event result and offline visibility at each step, reusing the
checkpoint driver used by the fixed corpus. The runner is a dev dependency.
Allow its MPL-2.0 terminal diagnostics dependency at the exact audited
version. Review the exception when updating or removing the runner.
[^quint-shared-cache-replay][^quint-connect-dependency-policy]

[^quint-shared-cache]: [Shared disk-cache transition model](../../../formal/quint/shared_cache.qnt).
[^quint-shared-cache-replay]: [Shared-cache trace replay](../../../src/acquisition/http_tests/cache_interleaving.rs).
[^quint-shared-cache-corpus]: [Generated shared-cache traces](../../../tests/fixtures/oracle/shared-cache.jsonl).
[^quint-connect-dependency-policy]: [Version-scoped license exception for test diagnostics](../../../deny.toml).

# Extras on cyclic paths

Use a finite Quint graph to check why a visited state includes its activated
extras. Vary activation at the root, activation on the cycle's return edge,
and a shorter path from the root. Compare the traversal with the shortest
path derivable from this graph shape. A model keyed only by package loses
the only reachable path when activation occurs on return.[^quint-extras]

Check every generated terminal case through the public Rust resolver. Keep
the reverse-distance property for larger graph shapes and multiple extras.
The trace adapter reads the scenario and expected path from the model corpus.
Revisit the model when extra activation or path tie semantics change.
[^quint-extras-corpus][^quint-extras-replay][^generated-resolution-graphs]

[^quint-extras]: [Finite cyclic extras model](../../../formal/quint/extras.qnt).
[^quint-extras-corpus]: [Generated terminal extras cases](../../../tests/fixtures/oracle/extras.jsonl).
[^quint-extras-replay]: [Public resolver replay](../../../tests/resolution_paths.rs).

# Conditional edges and same-name forks

Use finite selector tables to check exact version, source, and constraint
intersection over same-name locked identities. A unique edge retains its
marker as an annotation. A non-unique ordinary edge rejects the lock, while
a non-unique manifest dependency group omits its origin and preserves
locked coverage. The negative control ignores source selection.
[^quint-locked-forks]

Replay all 192 combinations against the public resolver and check the exact
inventory and selected path. Keep existing behavioral coverage for input
diagnostics and unrelated records. Revisit the finite domain when conditional
edge matching or manifest origin handling changes.
[^quint-locked-forks-corpus][^quint-locked-forks-replay]

[^quint-locked-forks]: [Finite locked fork selection model](../../../formal/quint/locked_forks.qnt).
[^quint-locked-forks-corpus]: [Generated fork selectors and decisions](../../../tests/fixtures/oracle/locked-forks.jsonl).
[^quint-locked-forks-replay]: [Public resolver replay for fork selectors](../../../tests/resolution_matching.rs).

# Tie order for annotated dependency paths

Rank complete routes on a fixed graph and compare the result with the
shortest route derivable directly from the graph shape. Tie resolution
uses canonical edge order, and ordinary edges precede optional groups.
Preserve the selected marker and extras annotations. Each entry origin
selects its own route. A negative control preserves wire order and loses
that tie rule.[^quint-path-ties]

Use all 32 generated inputs in the public Rust resolver. Reverse both
package and root-edge order and vary the marker comparisons, active extras,
ordinary edge and shortcut. Keep the larger generated graph suite for
different topology. Revisit this model if diagnostic ordering or extra
normalization changes.[^quint-path-ties-corpus][^quint-extras-replay]

[^quint-path-ties]: [Finite canonical path selection model](../../../formal/quint/path_ties.qnt).
[^quint-path-ties-corpus]: [Generated annotated paths](../../../tests/fixtures/oracle/path-ties.jsonl).

# Scoped policy and proof strength

Model the composed inspection decision for a hash-scoped exception and
clarification, with independent identity, digest, mode, and proof inputs.
Check which cases require archive proof before interpreting a declaration.
Once interpretation succeeds, apply global deny precedence and record the
matching override identifiers. The negative control admits a wrong-hash
exception.[^quint-scoped-policy]

Replay all 96 combinations through the public inspection API, using real
archive and sidecar evidence. Compare both the requested proof and the
reported verdict and interpretation. Keep Lean and property tests for
expression structure and existing focused cases for policy selectors and
unused entries. Revisit the model when proof admission or override precedence
changes.[^quint-scoped-policy-corpus][^inspection-exit-property]

[^quint-scoped-policy]: [Finite scoped policy and proof model](../../../formal/quint/scoped_policy.qnt).
[^quint-scoped-policy-corpus]: [Generated policy decisions](../../../tests/fixtures/oracle/scoped-policy.jsonl).

# Unused policy warnings across inspection results

Count all effective expression units, including each OR alternative.
Account for exact package/digest exceptions and hash-bound clarifications.
Incomplete runs suppress unused warnings. Complete runs retain the policy
verdict and may escalate compliance under strict warnings. The negative
control accounts for only one OR alternative.[^quint-unused-policy]

The initial oracle used the policy's emission order and disagreed with
the public report. Report assembly sorts these diagnostics by message
within their common error code. Use that ordering in the normalized
corpus and replay exact messages and exit codes across the 128 finite
cases. Revisit the model if usage accounting or warning presentation
changes.[^warning-report-order][^quint-unused-policy-corpus][^quint-unused-policy-replay]

[^quint-unused-policy]: [Finite policy usage model](../../../formal/quint/unused_policy.qnt).
[^quint-unused-policy-corpus]: [Generated warning and exit decisions](../../../tests/fixtures/oracle/unused-policy.jsonl).
[^quint-unused-policy-replay]: [Public report replay for unused policy warnings](../../../tests/policy_usage.rs).
[^warning-report-order]: [Public report warning order](../../../src/report.rs).

# Description evidence grouping and incomplete observations

Model two artifact slots for a selected package. Resolve observation
cardinality, content binding and declaration interpretation before grouping
by the exact expression. Unknown counts are per artifact. An unclaimed
request affects report completeness while successful artifact groups
remain visible. The negative control accepts the first of a duplicate pair.
[^quint-description-evidence]

Replay all 324 finite inputs through the public Description API with real
archive and sidecar observations. Compare diagnostic codes, group membership
and exit status. Preserve the separate Lean coverage of graph selection and
the existing Rust cases for source and identity binding. Revisit this model
when description accounting or grouping changes.
[^quint-description-evidence-corpus][^quint-description-evidence-replay]

[^quint-description-evidence]: [Finite description evidence accounting](../../../formal/quint/description_evidence.qnt).
[^quint-description-evidence-corpus]: [Generated description outcomes and groups](../../../tests/fixtures/oracle/description-evidence.jsonl).
[^quint-description-evidence-replay]: [Public description report replay](../../../tests/description.rs).

# Sidecar admission and archive fallback

Compose index admission with metadata verification, archive fallback and
offline reuse. Index admission requires a unique matching locked URL and
archive digest. Metadata admission binds the response bytes and package
identity. Strong proof bypasses both index and sidecar. The negative control
accepts a response with the wrong metadata digest.[^quint-sidecar-fallback]

Replay 240 cases through the binary acquirer using JSON and HTML fixtures,
wheel bytes and an actual disk cache. Compare request counts, report proof,
declaration, and diagnostics. Reopen the cache in offline mode to verify
the effect of successful and failed paths. Keep the Lean index corpus for
parser lookup cases. Revisit the composed model when sidecar admission,
fallback, proof strength, or cache eligibility changes.
[^quint-sidecar-fallback-corpus][^quint-sidecar-fallback-replay]

[^quint-sidecar-fallback]: [Finite sidecar admission and fallback model](../../../formal/quint/sidecar_fallback.qnt).
[^quint-sidecar-fallback-corpus]: [Generated cold and offline acquisition expectations](../../../tests/fixtures/oracle/sidecar-fallback.jsonl).
[^quint-sidecar-fallback-replay]: [Binary acquisition replay](../../../src/acquisition/http_tests/sidecar_fallback.rs).

# Pending and completed bundled license observations

Model the sequence from an inconclusive sidecar through archive admission,
fresh declaration interpretation and cache reuse. A failed archive fetch
keeps the sidecar pending. Offline inspection can read that pending evidence,
and the next online run retries the archive. Completed archive observations
remain reusable even when license interpretation is inconclusive. The
negative control fetches the archive again for a completed negative
observation.[^quint-legacy-scan]

Replay 16 five-step schedules through the binary acquirer and public
inspection, with real wheel and metadata bytes and a persistent disk cache.
Check archive authority, proof and interpretation origin as well as
request counts and errors. Keep the parser and family-compatibility
properties for text classification. Revisit the lifecycle when pending
evidence, archive interpretation, or cache reuse changes.
[^quint-legacy-scan-corpus][^quint-legacy-scan-replay]

[^quint-legacy-scan]: [Finite bundled license scan lifecycle](../../../formal/quint/legacy_scan.qnt).
[^quint-legacy-scan-corpus]: [Generated pending and completed cache decisions](../../../tests/fixtures/oracle/legacy-scan.jsonl).
[^quint-legacy-scan-replay]: [Binary replay for archive license scans](../../../src/acquisition/http_tests/legacy_scan.rs).

# HTTP routing, retries, and authentication

Compose per-hop retry limits, redirect limits, HTTPS enforcement, and
origin-bound credential forwarding. Use a permanent downgrade of
credential eligibility after an origin change within a request.
Compare final attempts with an independent count by route and retry
pattern. The negative control restores authentication on return to the
initial origin.[^quint-http-routing]

Replay all 96 sequences through the production HTTP wrapper with a
scripted transport and synthetic credentials. Check actual outgoing URLs,
credentials, terminal status, and attempt and byte totals. Loopback tests
remain the evidence for the TLS and response-reader paths. Revisit the
model if retry budgets, redirect handling, or credential scope changes.
[^quint-http-routing-corpus][^quint-http-routing-replay]

[^quint-http-routing]: [Finite HTTP routing and retry model](../../../formal/quint/http_routing.qnt).
[^quint-http-routing-corpus]: [Generated HTTP attempt transcripts](../../../tests/fixtures/oracle/http-routing.jsonl).
[^quint-http-routing-replay]: [Production HTTP wrapper replay](../../../src/acquisition/http_tests/http_routing.rs).

# Release reservation retries and reconciliation

Model the lease read, competing atomic update, push budget and final
reconciliation over a linear four-commit history. Compare stable tag and
ordering ref state and the reported result after competing reservations
and rejected or unacknowledged pushes. The negative control omits the final
read and loses a reservation that was created on the fifth push.
[^quint-release-reservation]

Regenerate the 32 terminal cases in CI and replay them in the frozen Python
environment against local bare Git. A second checkout updates the real local
remote before the stale caller pushes. Inject acknowledgement faults after
the subprocess returns. Check both refs and the attempt count. The small
Lean reservation model covers the snapshot decision. Revisit the Quint
schedule if the lease, retry budget or acknowledgement recovery changes.
[^quint-release-reservation-corpus][^quint-release-reservation-replay]

[^quint-release-reservation]: [Finite release reservation interleavings](../../../formal/quint/release_reservation.qnt).
[^quint-release-reservation-corpus]: [Generated reservation decisions and refs](../../../tests/fixtures/oracle/release-reservation.jsonl).
[^quint-release-reservation-replay]: [Local bare Git reservation replay](../../../tests/test_release.py).
