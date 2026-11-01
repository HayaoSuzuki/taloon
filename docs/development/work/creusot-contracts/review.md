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

