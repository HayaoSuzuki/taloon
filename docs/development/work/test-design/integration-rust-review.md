# Rust integration test design review

Reviewed on 2026-11-14 in the working tree, without commits or production changes.
Scope is `tests/*.rs`, excluding `description.rs`, `evidence.rs`, and
`inspection.rs`, which have a separate review. The development entry point,
knowledge catalog, verification decision, and documentation procedure supplied
the applicable contracts. Existing assertions and generated properties were
retained across context splits. No product behavior, public contract, or design
decision changed, so no user guide or OKF concept update was required.

## File outcomes

| File | Outcome and oracle/loop assessment |
| --- | --- |
| `bundled_licenses.rs` | Retained: named declaration, archive, cache, and supplementation scenarios; generated added-term rejection. ZIP/TAR loops construct a single archive fixture. `values` crosses named license cases with the two container formats independently. |
| `cli.rs` | Changed: project discovery, missing policy, empty inventory, verdicts, missing artifact, invalid arguments, credentials, terminal controls, cache roots, cache idempotency, reacquisition, description selection, malformed policies, text presentation, shared cache, and distinct versions now execute in separate contexts. Named cases replace verdict, symlink-spelling, error-argument, and format loops. JSON/text clarification assertions are separate tests; policy cases explicitly state expected decision mode, including null. Command shape construction lives in a helper. Generated exit precedence remains an independent Boolean oracle. Remaining loops construct malicious archive members, executable traps, or retained cache fixtures. The retained-file assertion observes one whole cache-preservation outcome. |
| `cli_diagnostics.rs` | Changed: ordinary/verbose JSON and preview variants supply explicit named flags, eliminating fixture-selection branches. Other named diagnostic cases and generated distinct-suffix grouping remain. Artifact/config loops build fixtures. |
| `coverage_boundaries.rs` | Retained: each test exercises one rejected public boundary; structural variants are named cases with explicit error expectations. |
| `deny_policy.rs` | Changed: complete-run unused-denial warnings and incomplete-run suppression are distinct tests with unconditional assertions; remaining cases already isolate evidence/permission outcomes. |
| `design_performance.rs` | Retained: chain/fanout are independently named ignored benchmarks. Node/edge loops build fixtures. Five samples intentionally repeat the same measured workload and validate its node/path invariants, rather than representing independent behavior cases. |
| `evidence_ownership.rs` | Changed: rebinding compares a single typed `Result` with explicit expected error/value, without branching assertions. Cloning property retains record invariance, independently selected proof strength, and future-version warning expectations. |
| `evidence_performance.rs` | Retained: check/describe independently named ignored benchmarks; archive loop builds 64 inputs. Five timing samples intentionally repeat the same operation with fresh result vectors. |
| `legacy_expressions.rs` | Retained: named parser/limit/precedence cases plus recursive formula strategies with independent native Boolean evaluation for allow and deny semantics. No independent example loop. |
| `legacy_licenses.rs` | Changed: notice-boundary cases provide explicit notice/restriction text and expected expression; archive/sidecar fixture construction moves to a helper. Fixed aliases, text regressions, policy outcomes, Unicode/cache invariants, case masks, whitespace and added-condition properties remain. Character maps vary one generated input, rather than executing separate scenarios. |
| `license_diagnostics.rs` | Retained: named sidecar/cache causes and generated repeated-declaration cardinality oracle. Entry loop constructs the declaration header fixture. |
| `model_debug.rs` | Changed: generated representation choice now belongs to the strategy, alongside credential suffix generation. Named representation cases and positive host-preservation checks remain. |
| `observation_accounting.rs` | Changed: check/describe consumer choice now belongs to the strategy. Count loop builds one generated multiset; assertion loop checks its per-request cardinality invariant against integer counts, independently of ledger internals. |
| `oracle.rs` | Changed: all 1,347 frozen Lean rows are individually named rstest cases, selected by row and named with corpus IDs. The original typed adapters and expected values remain. A row-count guard makes corpus growth require updating the adapters. Remaining request/slot loop constructs one inspection observation fixture; it no longer runs independent corpus scenarios. |
| `package_names.rs` | Retained: independently named invalid-name boundary cases with explicit public error expectations. |
| `performance.rs` | Retained: ignored end-to-end warm-cache benchmark. Package/artifact loops construct the 10,000 distinct archives; five warm samples measure the same warmed cache. Cold execution is the required setup for the warm-cache property. |
| `permission_units.rs` | Retained: named constructor/identity/custom-scope cases. Generated set coherence compares ordinary string order/membership with typed set behavior, not independent parser correctness. Membership iteration checks one generated set invariant. |
| `policy.rs` | Retained: named configuration, expiry, scoped permission and deny cases; Boolean mask oracles, recursive permission monotonicity and normalization idempotence properties. Generated fixture/Boolean expression choices describe one property input. |
| `policy_index.rs` | Retained: named selector/hash/clarification cases plus independent integer identity/scope/permission and unused-entry expectations. Rule loop builds one policy document. Generated matching and warning collections use integer selectors, not the production index. |
| `policy_performance.rs` | Retained: independently named ignored global/scoped benchmarks. Catalog and request loops build fixtures; sample/request loops measure one fixed aggregate lookup workload, with explicit permitted-count checks. |
| `pure_coverage.rs` | Retained: named error/format boundary cases and individual public graph/identity regressions. Multiple observations describe a single graph or identity outcome. |
| `report_metrics.rs` | Changed: wide-integer saturation and order independence are separate generated properties. The merge loop is the operation over a generated collection. Saturation uses an exact u128 sum, capped once at u64 max; wall time remains independently checked. |
| `resolution.rs` | Changed: missing edge, ambiguous edge, unsupported revision and unsupported version are independently named cases. Other single-graph regressions and generated inventory expectations remain. |
| `resolution_matching.rs` | Retained: named selectors and failure codes; generated unrelated-record count with known independently chosen identity. Package loops build the lock fixture. |
| `resolution_paths.rs` | Retained: named graph boundaries and annotated extra-state regression. DFS exhaustively enumerates simple paths as an independent oracle for the production shortest-path traversal; its traversal loops are oracle computation over one generated graph. |
| `workflow.rs` | Retained: one hook/CLI equivalence scenario checks full-lock processing and unchanged lock bytes. Both command observations are necessary to that equivalence property. Package loop builds the direct/transitive fixture. |

`tests/common/{mod.rs,fixtures.rs,license_text.rs}` were inspected as shared fixture
support and left unchanged. Archive member iteration and license text transforms
construct fixtures; they do not conceal independent assertions or test cases.

## Verification

Focused stable verification used the pinned local toolchain and all features:

```sh
cargo test --locked --test cli --test cli_diagnostics --test model_debug --test observation_accounting --test report_metrics --test deny_policy --test evidence_ownership --test resolution --test oracle --all-features --quiet
cargo test --locked --test legacy_licenses --all-features --quiet
cargo clippy --locked --test cli --test cli_diagnostics --test model_debug --test observation_accounting --test report_metrics --test deny_policy --test evidence_ownership --test resolution --test oracle --test legacy_licenses --all-features -- -D warnings
```

The selected executables passed: CLI 90, CLI diagnostics 20, deny policy 9,
evidence ownership 18, model debug 5, observation accounting 3, policy oracle
1,348, report metrics 3, resolution 14; legacy licenses 153. Focused Clippy
passed. Owned edited files were formatted individually with Rustfmt edition
2024; no workspace-wide formatting was performed while other agents edited.
The parent review owns final combined-suite, documentation, shuffled and coverage
verification. Ignored timing benchmarks were audited but not executed here;
these results do not establish performance measurements or exhaustive coverage.

## Semantic case-name correction

The policy adapter now names all 1,347 cases by their actual conditions and
expected outcome, rather than a corpus row number. Permission names state the
expression and allow set; expiry names state the relation to evaluation date;
inspection names state both artifact evidence strengths, proof requirement,
permission, anomalies, warnings, escalation and exit outcome. Stable corpus IDs
select the expectations independently of line order. The corpus bytes are
unchanged. Focused oracle verification passed 1,348 tests after this correction;
this subsequent change is outside the earlier full-workspace coverage snapshot.

Case labels were subsequently shortened: normal sidecar/unique/no-warning
conditions are documented once above the cases; labels include deviations.
For example `mit_allow_none_denied` and
`mit_archive_archive_allowed_ok` retain the tested condition and verdict without
repeating the full configuration vocabulary. Corpus IDs and expectations remain
unchanged.
