# Final integration evidence

This extends the unmerged PR #12 after b19f596. Topic commits separately address
[Issue #15](issue15.md), [name normalization](name.md), [report aggregation](aggregate.md),
[counter merging](accounting.md) and [evidence binding](evidence.md).
The [plan](plan.md) records the approved scope. [Graph verification](graph.md)
was probed and deferred under the owner's explicit permission; production graph
behavior remains unchanged.

## Observed local checks

On macOS arm64, the integrated source passed 422 Rust tests (existing ignored
benchmarks/fixtures remain ignored), strict Clippy, formatting and rustdoc.
The 137 Python tests, documentation reference/OKF validation, strict user-site
build, actionlint/ShellCheck and zizmor passed. The unchanged Lean model built
and all three generated corpora passed freshness checks; Rust correspondence
adapters are included in the full Rust suite. New tests follow AAA with named
independent cases and generated independent oracles.

Pinned Creusot ran inside Linux x86_64 Docker. Fresh no-cache verification passed
all 21 files and 43 named goals for each of Linux x86_64, macOS ARM64 and Windows
x86_64 targets. All fourteen compiling mutations were rejected at their intended
named goals. Recorded proof-input hashes match the final source; subsequent
review fixes affect root adapters/rendering, not verified kernel bodies. This is
cross-target verification, not native execution on three operating systems.
The expanded negative suite has a 600-second CI deadline instead of the original
300 seconds for five mutants; per-solver limits and the 30-minute job limit stay
unchanged. Hosted results are separately observable on PR #12.

An initial local rustdoc attempt loaded stale decision-crate metadata and could
not resolve newly added modules despite successful tests/Clippy. Cleaning only
that package's generated Cargo outputs and rerunning the same documented command
passed. No source workaround or relaxed gate was added.

## Review and limits

Five final passes checked (1) requested targets and authorized graph deferral,
(2) real production calls rather than duplicate proof algorithms, (3) borrowed
storage, enum states and no extra graph/status copies, (4) independent generated
oracles and compiling negative witnesses, and (5) exact proof inventory, source
snapshots, documentation ownership and reproduction limits.

A separate final reviewer found adjacent-URL and embedded-control credential
redaction gaps plus one direct archive comparison outside the shared rule.
Four URL regressions failed before the fixes and pass afterward; archive digest
validation now calls the verified rule. The reviewer otherwise found no required
type/storage/proof correction. No second independent approval of these fixes is
claimed. That review did not certify final common documentation, new executor
runs, cryptographic/parser/stdlib trust boundaries, native foreign-OS behavior
or the explicitly deferred graph proof. The executor checks cover the updated
common documentation and integration; cryptography, parsers, supplied contracts,
compiler/solvers and full dependency exploration retain their stated limits.
