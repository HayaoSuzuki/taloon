# Randomized tests and strict Python checks: review evidence

Base: `ae0c29932017eb789f751c2d5d171f7ca36e473f`, after rebasing onto
merged describe PR #8. Local verification: macOS arm64, 2026-11-01, working-tree
changes on `ci/python-quality-and-randomized-tests`. This record supersedes
pre-rebase local results for this task. GitHub results apply to their own SHA.

## Design self-review: five passes

1. Scope: stable checks remain authoritative; a separate pinned nightly job adds
   harness shuffle. Product behavior, permissions and supported platforms stay
   outside this task. No new formal model is claimed for tooling configuration.
2. Reproduction: libtest seeds apply per executable; fixed seeds cannot reproduce
   arbitrary concurrent completion. pytest-randomly is required, rather than an
   optional plugin that could silently disappear.
3. Quality boundary: Ruff ALL checks maintained Python tooling and tests; ty uses
   all-error strict analysis and ignores no type-ignore comments. Runtime Python
   dependencies remain empty. Only formatter conflicts and scoped exceptions
   are allowed; arbitrary metadata uses object values rather than Any.
4. Test design: independently reported cases replace loops; one scenario/property
   per function, AAA phases, fixed boundaries plus independent generated oracles.
   rstest has default features disabled and is test-only. Existing untouched Rust
   suites are not claimed to have been rewritten.
5. Ownership: developer commands and design guidance stay in the existing guide
   and verification decision. User commands and release policy have not changed,
   so no user-guide update or new distribution decision is needed.

## Implementation self-review: five passes

1. Compared release reservation/recovery changes with the base: atomic Git
   reservation, lost responses, source ancestry, published release matching and
   staging refusal remain covered by isolated cases. Executable resolution and
   precise generic annotations replace implicit subprocess assumptions.
2. Compared metadata/provenance and docs validators: source matching, attestation
   signatures, hash mutation, metadata rejection and unknown metadata preservation
   remain exercised. JSON guards reject invalid container shapes explicitly.
3. Checked test isolation: temporary repositories, environment monkeypatches,
   independent parametrized cases, property fixtures and optimized Python probes
   do not depend on test order. Production wheel assertions now remain active
   under Python optimization.
4. Checked CI and hooks: frozen dependency groups include release test imports;
   new quality hooks check the whole maintained Python surface. Stable Rust and
   explicit ignored/TLS coverage remain separate from the added shuffle job.
5. Checked documentation, source attribution, lock changes and exclusions:
   hoimin references identify the revision inspected; no runtime dependency or
   license-policy relaxation is introduced. Build and generated fixture output
   remain excluded from Python lint/type scope.

## Final verification self-review: five passes

1. Re-read the owner's Rust/Python requirements against the final guide, agent
   instructions, CI and local hook configuration; checked the rebased base.
2. Repeated stable and nightly workspace tests after the final Rust case split;
   134 passed in each run with generated and fixed nightly seeds observed.
3. Repeated Python tests after the independent version invariant change; all
   134 passed under generated, fixed and cached seeds. Same-seed collection order
   repeated exactly; another seed changed it.
4. Re-ran all seven local hooks, strict docs checks, workflow checks and dependency
   checks; exercised the actual built wheel on both supported Python versions.
5. Obtained an independent whole-change review and a follow-up on its observation;
   inspected the staged diff, source attribution and working-tree cleanliness
   before committing. No unresolved findings remain.

## Observed checks

- Ruff 0.17.3 formatting and ALL lint; ty 0.0.95 strict analysis: passed.
- pytest-randomly 5.0.0: 134 cases passed with generated seed 2459279572,
  seed 12345 and cached replay;
  collection-order comparison repeats for the same seed and differs for another.
- Stable Rust formatting, Clippy and workspace tests passed after the rebase.
  All 134 cases passed in stable and pinned nightly workspace tests, the latter
  with generated seeds and seed 12345.
  The expanded policy suite includes 256 generated Boolean-permission cases.
- All seven prek hooks passed, including Ruff formatting/lint and ty. A real
  macOS arm64 wheel built from this tree passed the updated installation/cache
  smoke probe using Python 3.14 and 3.15. This is local platform evidence.
- cargo-deny 0.20.2 licenses/sources and cargo-audit 0.22.2 passed with the added
  test-only dependency graph. Workflow actionlint, ShellCheck and offline zizmor
  passed; offline audits do not establish remote action provenance.
- Focused mutation verification used the released hoimin 0.14.47 wheel. Three
  selected JSON mapping guard mutations (negation, OR-to-AND, empty key generator)
  were killed, each with a fresh passing baseline. This is targeted sensitivity
  evidence, not a whole-tooling mutation score. Jobs 1, workspace cap 8GiB and
  reserve 10GiB remained unchanged; macOS memory enforcement was best effort.
  Execution/delivery cleanup was clean and the exact temporary directory removed.
  Initial CLI/output-format and interpreter-copy setup errors were diagnosed
  before replanning; they are not counted as test results.

Independent whole-change review found no Critical/Important issue and independently
passed 44 focused tests. Its minor observation about a formula-shaped version
oracle was addressed with ordering, floor and predecessor freshness invariants.
CI observations are recorded in the PR. Existing Lean corpus
and fuzz algorithms are unchanged; no additional proof coverage is claimed.
