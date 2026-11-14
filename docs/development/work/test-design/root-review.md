# Description, evidence and inspection test audit

Scope: all test functions in `tests/description.rs`, `tests/evidence.rs` and
`tests/inspection.rs`. This record concerns the uncommitted changes based on
`f8e3127`; final combined execution evidence is recorded separately.

| File | Findings and changes | Retained patterns |
| --- | --- | --- |
| `description.rs` | Split missing, duplicate, foreign, identity and order scenarios; separate custom declarations, unknown queries, unsupported sources, invalid declarations and empty projects. Named project/member cases. All 50 Lean selection rows now run independently, separated by successful and missing-root contexts. Split generated graph closure and input-order invariance into distinct properties. Separate hashless description and inspection substitution scenarios. | Graph-building loops and independent Floyd-Warshall oracle; two evaluations define the single order-invariance property. Multiple assertions inspect one graph/report. |
| `evidence.rs` | Split archive/sidecar strength, identity, metadata versions, duplicate fields/members, encoding and record corruption contexts. Named input cases. Separate archive and sidecar generated cache roundtrips, expand valid expression domain to operands and operators, add generated foreign names with guaranteed distinct identity. | ZIP byte-patching loop constructs one malformed archive. Paired small/large descriptions define a size-invariance scenario. Byte mutation spans all positions and all eight bits. |
| `inspection.rs` | Split complete, partial, empty, duplicate, weak, foreign, identity and warning scenarios. Named policy/source cases. Exhaust four finite order contexts with rstest instead of repeatedly generating booleans. Add generated duplicate multiplicities from 2 through 31 with an independent incomplete/invalid-result expectation. | Result builders may branch to construct success/failure fixtures. Reversing results and evaluating twice defines one order property. |

AAA boundaries are made explicit in changed scenarios. Error outcomes are
captured before assertions. Generated input strategies do not invoke production
parsers to construct expectations. The selection oracle uses a different
algorithm from the production queue traversal. Existing readable boundary cases
remain alongside generated properties.

No production contract or user-visible behavior changes: no user-guide or OKF
contract update is required. This audit does not claim branch coverage,
exhaustive input coverage or correctness solely from line execution.
