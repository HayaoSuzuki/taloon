# Creusot adoption evidence and reviews

Base: `7722bae`, macOS ARM64 host, Linux ARM64 Docker proof execution.
Scope: [design](design.md) and [plan](plan.md). No existing documentation/code
contradiction was found in the inspected verification boundaries. During final
synchronization, the baseline's current "sole package" sentence was found to
predate this extraction: the workspace now has two packages. This discrepancy
was recorded before updating the baseline and adding the concrete verifier
isolation reason to the pure-core decision.

## Stage 1: observation accounting

The same production `taloon-decisions::observation` bodies were generated and
proved for x86_64 Linux, ARM64 macOS and x86_64 Windows. Three observation files contain six
named obligations per target. All succeeded with fresh generation and no solver
cache, using the archive-pinned ARM64 tool image and its successful
installation/project preflight. This is cross-target proof execution on Linux,
not execution on Windows/macOS. The new archive-pinned build procedure is still
being reproduced; final environment-build evidence belongs to stage 2.

Initial compilation exposed unsupported `matches!` inside Pearlite and ambiguous
wildcard derive imports. Contracts now use logical matches/equality and an
explicit ensures import. No trusted application axioms were added. The runner
initially expected a one-file summary, while Why3find emits the actual path;
its success check now uses the observed exact path. A rerun passed all targets.

Both compiling negative witnesses succeeded in generation and were rejected by
`vc_mark_duplicate_T` (duplicate invalidation omitted) and `vc_take_T` (slot left
Some(Duplicate) instead of None). The unmodified source was never changed by the witnesses.
Temporary reports are deliberately not committed or used as proof inputs.

The new slot tests initially failed to compile because the module was absent;
implementation then passed its fixed cases and independent generated event-history
oracle. Existing workspace tests passed before and after extraction. Added public
Inspection cases exercise success/failure duplicate permutations. Root/fuzz
lockfiles retain existing resolved versions and add the new kernel/proof graph.
Clippy and root/fuzz license/source checks passed. Ordinary `cargo tree -p
taloon-decisions --edges normal` contains only the crate, with no Creusot runtime.

Five separate self-review passes:

1. Contract scope: each observation input constructor maps to an exact postcondition;
   take also proves the final empty state. Arbitrary payloads are retained exactly.
2. Production connection: ResultLedger uses the actual verified type; map identity,
   foreign IDs and Evidence rebinding remain separate and are not claimed proved.
3. Storage/performance: retain one Box per first result, discard duplicate payloads,
   and allocate no extra Box for duplicate arrivals. BTreeMap lookup stays logarithmic.
   The initial three-state representation grew a retained slot from 8 to 16 bytes.
   A size regression failed (16 versus 8); keeping only Single/Duplicate in the
   map and using Option for absence restored 8 bytes and passed. Empty state is
   consumed through Option::take, so the exact consumption contract remains.
4. Test sensitivity: fixed/generated slot checks, public mixed-duplicate cases and
   two compiling rejected mutations cover distinct boundaries; timeout is failure.
5. Reproduction/trust: exact nonempty file/goal inventory, clean generation, no-cache
   solvers, input checksums and preflight prevent stale/empty proof success. The
   upstream compiler/stdlib/solvers and map adapter remain explicit trust boundaries.

The extraction has one responsibility (decision semantics), a smaller interface
than implementation/contracts, replaceable storage callers, documented guarantees,
explicit design review, hidden cardinality decisions, named module boundaries and
recorded design investment: software-design-philosophy 8/8 diagnostics (10/10)
for this limited extraction. This is not a score for the whole codebase.


## Stage 2: verdict and exit precedence

The production Status enum, pairwise aggregation and exit-code bodies now live
in the same crate. Reports call those bodies directly. Aggregation scans each
package at most once, stopping on Incomplete; diagnostics still override package
violations. Warning discovery retains short-circuiting for already failing
statuses. JSON spelling and the existing public re-export are unchanged. This
removes a second package scan without asserting a measured wall-time speedup.

Fresh verification on the same pinned ARM64 environment succeeded for all three
targets: five files and nine named obligations per target. All five compiling
mutations were rejected by the corresponding goal, including precedence,
incomplete exit and warning conjunction. Exit-code contracts needed explicit u8
literals because Pearlite defaults numeric literals to mathematical Int.

The original set-version workflow updates only the root package/lock record.
An internal crate inheriting that version produced a real `cargo metadata
--locked --offline` failure after injecting 0.1.999999. The internal nonpublished
crate now owns a fixed 0.1.0 version; the identical experiment then succeeded.
A durable three-case release regression failed with inheritance (3/3) and passes
with the fixed version. No production Python implementation changed. No hoimin
mutation target was selected because the changed Python file is test-only and
the faulty setting was Rust package metadata.

The Rust workspace suite, strict Clippy/rustdoc, unchanged Lean build and all
three corpus freshness checks passed. Python's randomized suite passed all 137
tests; Ruff formatting/ALL and ty strict checks passed. Workflow actionlint,
ShellCheck and offline zizmor passed. A final post-version-fix proof run also passed all three targets with five
files/nine goals each; its input snapshots include the final fixed manifest.
The final Rust workspace run passed 348 tests (18 deliberately ignored).
The archive-pinned build initially failed when menhirCST's GitLab transfer
ended prematurely (curl 18, opam 40). Retrying unchanged inputs succeeded,
reusing completed Docker layers from that attempt; this is not a successful
genuinely cold timing measurement. The resulting `taloon-creusot:local` ARM64
image passed installation/project preflight, all three targets' five-file /
nine-goal proofs and all five rejected compiling mutations. No source hash,
solver limit or package policy was weakened to recover from the transfer error.
Actual GitHub CI results are reported in PR #12 rather than inferred from local
cross-target execution.

Five separate self-review passes:

1. Domain/precedence: all nine pairs and twelve exit combinations are independent
   rstest cases; the exact contracts cover all typed inputs without preconditions.
2. Integration: an independent max-rank property exercises actual Report::new
   diagnostics, package aggregation and warning escalation, alongside unchanged
   public Lean adapters. Root traversal/serialization are tested, not claimed proved.
3. Types/storage/performance: one Status owner, exhaustive Enum matches, no
   allocation in combine/exit, one package scan and preserved warning short circuit.
   The serde feature adds no package to the previously shipped dependency set.
4. Proof/CI sensitivity: exact five-file/nine-goal inventory, all five compiling
   rejected witnesses, no trusted application function, required existing context
   and bounded tool invocations fail on skipped/empty/stale or unproved output.
5. Distribution/docs: internal version isolated from root release injection; real
   Cargo RED→GREEN and permanent parameterized regression, both lockfiles updated,
   exact verification-only license exceptions, and current owning guides updated.

A fresh gpt-6-astra reviewer inspected the whole branch's production code, tests,
runner/sensitivity scripts, pinned public recipe/manifests and CI integration.
It found no new Critical/Important issue beyond the already identified release
version fix. Documentation synchronization was unfinished at review time and
is completed here. The reviewer read existing proof logs rather than rerunning
commands; it did not claim completed image builds or GitHub/native OS execution.
The final storage regression was found after that review; it was fixed with a
RED→GREEN pointer-size test, the complete 348-test Rust suite, fresh three-target
proofs and all five rejected mutations. The reviewer did not rereview this final
representation change. Five self-review passes additionally checked compact
layout, Option consumption, payload disposal, public adapter behavior and exact
proof inventories. No review finding is intentionally deferred. The two-target extraction retains
8/8 design diagnostics (10/10); whole-application proof coverage is not claimed.
