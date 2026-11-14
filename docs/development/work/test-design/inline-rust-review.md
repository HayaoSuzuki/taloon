# Inline Rust and decision-crate test review

Reviewed the uncommitted working tree on 2026-11-14 against the
[development test-design rules](../../index.md#test-design) and
[verification decision](../../knowledge/verification-strategy.md). This record
covers every Rust file containing test functions under `src/` and every file
under `crates/taloon-decisions/tests/`. Source production behavior was not changed.
No commits were created. The separately owned root integration tests and Python
tooling are outside this record.

The initial mismatch was test organization, not a disputed product contract:
cache and HTTP tests combined unrelated operations, index correspondence hid
432 row/representation combinations inside loops, and some generated checks
combined different properties. Those tests are now independently executable.
The 216-row corpus remains unchanged; its original cardinality assertion is
retained separately. Named cases select both representations for every row.

## Per-file outcome

Paths in this table are relative to the repository root. “Retained” means the
review found one scenario/property per function with an observable expectation;
AAA may be represented by separated statements rather than literal comments.

| File | Outcome and oracle review |
| --- | --- |
| `src/cli.rs` | Split explicit nonexistent netrc precedence from home fallback; retained document-boundary and progress-time cases. |
| `src/cli/text_tests.rs` | Retained named escaping/borrowing cases and generated numeric Unicode oracle, independently expressed from the implementation predicate. Timing loops are measurements, described below. |
| `src/cli/json.rs` | Split JSON value preservation from exclusion of literal terminal controls. The latter now checks independently specified numeric scalar ranges instead of calling the production predicate. |
| `src/cli/artifacts.rs` | Retained named proof/status grouping and diagnostic-remedy cases; assertions inspect one rendered body per function. |
| `src/cli/describe.rs` | Retained separate missing dependency and missing origin rendering contexts. |
| `src/declaration.rs` | Retained named URL-redaction regressions and generated exact-output oracles for ordinary email and embedded credentials. |
| `src/report.rs` | Retained exact serialization fixtures and generated rank-maximum exit oracle. Package/warning construction is setup. |
| `src/description/mod.rs` | Retained separate insufficient-proof and future-metadata-warning outcomes. Multiple fields describe the same described artifact. |
| `src/resolution/mod.rs` | Retained single unsupported-source artifact-builder boundary. |
| `src/acquisition/tests.rs` | Split content-key and proof-strength cache contexts; local first acquisition, unchanged replay, changed replay, pinned mismatch, and removed-file cases; named corrupt/interrupted/incompatible records; named global-cache source/identity/format cases; separate roundtrip and corruption properties. Mutation generation now covers all eight bit positions. |
| `src/acquisition/cache.rs` | Retained producer-name recognition, filesystem boundaries and worst-case escaping. Split generated clear preservation from repeat-clear idempotence. Neighbor assertions now compare the entire observed collection after one clear. |
| `src/acquisition/http.rs` | Retained separate complete response, oversized body and truncated body tests. The complete-response fields and captured request describe one send operation. Local sockets are required. |
| `src/acquisition/http_tests.rs` | Split panic/empty progress, cold/warm/offline snapshots, weak-cache online/offline behavior, redirect authentication/downgrade/retry/missing host, association/query/hash/script contexts, successful archive reuse/total failure, malformed index/HTTP size, cold/offline distinct associations, and cold/offline selected description. Named invalid-sidecar and ignored TLS variants replace hidden loops. Split signed-query acceptance, rejection and display-redaction properties. Explicit named cases replace online/offline `values` axes. |
| `src/acquisition/partition_tests.rs` | Retained named worker/group boundaries and generated integer-label oracle. Replaced the branching same-group assertion with a logical implication. Different groups may share one of eight workers; the oracle deliberately does not demand distinct workers for every distinct group. |
| `src/acquisition/index.rs` | Isolated all Lean row/representation pairs into named cases. Split arbitrary JSON, constructed JSON and HTML scope properties. Generated association expectations use an independent integer frequency table and compare the full result collection; wrong-hash rejection is a separate property. Fixture construction chooses representation outside assertions. |
| `src/acquisition/index/html_tests.rs` | Retained named base/order/raw-text/precedence/protocol cases and generated encounter-order URL oracle. Historical-anchor loops construct one page. Timing samples are measurements. |
| `src/acquisition/index/json_tests.rs` | Retained named malformed/duplicate field cases and integer advertisement-selection oracle; extension data is generated independently. Timing samples are measurements. |
| `src/evidence/tests.rs` | Retained isolated payload/header/archive/proof boundaries and unsupported-version property. Member-writing and byte-offset loops construct one archive fixture. |
| `src/evidence/archive_files.rs` | Retained named format/path/size/count boundaries. Extended the generated UTF-8 preservation property from sdist alone to strategy-selected wheel and sdist fixtures, comparing extracted text directly with the generated input. |
| `src/evidence/legacy.rs` | Retained named identity-resolution cases and generated batch oracle using maximum length plus a set of identities, independent of incremental production state. |
| `src/evidence/legacy/expression.rs` | Retained named longest-unique-prefix cases and single-pass traversal property. Visit count and selected prefix describe the same traversal. |
| `src/evidence/text_match.rs` | Retained isolated complete-text, altered-term and attribution contexts; generated structural substitutions preserve a fixed independently specified identity, while extra-condition strategies require rejection. |
| `src/evidence/text_match/variants_tests.rs` | Retained named structured BSD/Apache/MIT/MPL regressions and generated extra-term/contact guards; setup assertions verify substitutions actually changed the fixture. |
| `src/evidence/text_match/spdx_variants_tests.rs` | Expanded marker, license-family and reference axes into explicitly named cases, preserving every combination. Retained generated mixed-marker identity and added-term rejection properties. |
| `src/evidence/text_match/mit_preamble.rs` | Retained named invalid contact-domain cases; one predicate result per invocation. |
| `src/evidence/text_match/mit_preamble_tests.rs` | Retained named preamble, contact and count/byte boundaries plus generated corporate/contact/dated-holder properties with literal identity or rejection expectations. |
| `crates/taloon-decisions/tests/accounting.rs` | Retained named saturation/elapsed cases and generated `u128` arithmetic oracle, monotonicity and elapsed preservation as separate properties. |
| `crates/taloon-decisions/tests/aggregate.rs` | Retained named precedence cases and independent maximum-rank property. |
| `crates/taloon-decisions/tests/evidence.rs` | Retained named strength/sidecar cases and separate source/request/content-binding mutation properties. Changed indices determine expectations independently. |
| `crates/taloon-decisions/tests/name.rs` | Retained named boundary cases and separate component-join, ASCII-domain, fixed-point and length properties. Separator loop constructs one input. |
| `crates/taloon-decisions/tests/observation.rs` | Retained separate absent/present/consumed/duplicate contexts. Repeated writes construct one slot history. Generated expected outcomes split history at consumption events, independently of the production slot state. |
| `crates/taloon-decisions/tests/status.rs` | Retained named status/warning truth tables and independent maximum-rank fold oracle. |

`src/main.rs`, `src/acquisition/mod.rs` and `src/evidence/mod.rs` declare test
modules but contain no additional test functions. No decision-crate test files
were omitted or edited: their existing organization and independent properties
already met this review's requirements.

## Deliberately retained repetition

- Fixture construction loops assemble archive members, historical links, source
  tenants, byte mutations or a single generated name. They do not run hidden
  independent test cases.
- Partition assertions quantify over all pairs in one generated partition. The
  implication checks that a semantic group is never split; collection cardinality
  checks also reject lost or duplicated requests.
- Concurrent publication repeatedly observes one shared publication history while
  two writers replace the same record. Those repetitions are the concurrency
  scenario, not input variants. Progress tests likewise inspect intermediate
  messages before releasing a blocked transport; interleaved assertions are
  necessary to observe the blocked state before the final result exists.
- The ignored JSON/HTML parsing, grouping, terminal rendering and cold acquisition
  measurements retain sample/iteration loops. Samples estimate the same operation
  on the same fixture; independent input sizes/patterns remain named rstest cases.
  No timing threshold establishes behavioral correctness. The TLS listener's
  accept loop implements one retrying connection fixture, separately parameterized
  as bad TLS and stalled handshake.
- Generated arbitrary-input scope checks permit parser rejection. A successful
  parse must retain only the exact locked scope; an associated URL must have the
  exact independently specified metadata path/query. Conditional acceptance here
  states the property domain, rather than selecting different expected fixtures.
- The Lean adapter dispatches on the independently generated expected-outcome
  enum. Associated cases additionally verify that the returned proof can create
  the expected evidence; this is correspondence for that same association.

## Verification and limits

Local environment: macOS arm64, repository Rust 1.99.1 toolchain. These commands
were executed during the review; counts belong to their recorded snapshots:

- `cargo test --locked -p taloon-decisions --quiet`: 76 passed, no failures.
- `cargo test --locked --lib --all-features --quiet`: 387 passed after the
  generated wheel/sdist text-domain change, no failures.
- `cargo test --locked --bin taloon --all-features --quiet`: 778 passed and
  12 ignored at the intermediate HTTP-split snapshot. Further final checks are
  recorded by the enclosing test-design review.
- `cargo test --locked --bin taloon acquisition::index --all-features --quiet`:
  493 passed, 4 ignored at the generated index-property snapshot.
- `cargo clippy --locked --workspace --all-targets --all-features -- -D warnings`:
  passed before the final JSON-property, archive-domain and corpus-count additions.
- `cargo fmt --all`: applied after the final source edits.

The first sandboxed binary run failed three unchanged loopback socket tests with
`Operation not permitted`. Running with local sockets enabled resolved those
failures. A temporary overstrong partition assertion rejected valid worker
co-location above eight groups; it was corrected to the semantic implication and
the temporary seed for that invalid assertion was removed. This was a test-oracle
error, not a production defect.

This audit adds no claim of exhaustive input coverage, legal equivalence, new
Lean proofs or production correctness. User guidance and product contracts do
not change. The owning verification decision may link this evidence without
changing its existing behavioral/formal guarantee boundaries.

The independent cross-review identified two sensitivity regressions in the
intermediate split: the pinned local-file mismatch had lost its warm-cache
premise, and the script fixture used an unmatched relative URL. Before final
freeze, the local test restored original-evidence warmup before file mutation,
and the script fixture gained a matching base outside the script. This preserves
both the cache-bypass regression and the ability to detect links incorrectly
parsed from script contents. Source production prefixes were compared with HEAD
for all changed mixed source/test files and remained byte-identical.

Final targeted verification after those corrections:
`cargo test --locked --bin taloon changed_pinned_local_archive_is_rejected --all-features --quiet`
and
`cargo test --locked --bin taloon script_contents_cannot_advertise_sidecars --all-features --quiet`
each passed one test with 797 filtered out. The enclosing review owns the final
whole-tree stable, shuffled and fresh-directory coverage runs.

The first fresh coverage pass identified two test-design gaps introduced by this
review. Arbitrary index text almost never formed a valid scoped sidecar, so the
HTML/JSON link generators now explicitly mix the valid scoped URL with arbitrary
text; both accepted and rejected inputs exercise the same scope invariant. Cache
damage fixtures now use an exhaustive enum rather than a string selector with an
unreachable fallback. Neither change adds a coverage exclusion.
