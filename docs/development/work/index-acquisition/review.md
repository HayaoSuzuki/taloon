# Final index acquisition review

On 2026-11-01, a fresh read-only reviewer (`gpt-6-astra`, task
`index_acquisition_final_review`) reviewed `4242995..6d6db60` against the
[approved design](design.md), [plan](index-acquisition-plan.md), implementation,
tests, model correspondence and [measurements](measurements.md). The reviewer
found no Critical, Important or Minor issue, set aside no considered behavior
as outside the specification, and assessed the change as ready to merge.
It did not rerun tests or independently reproduce the timings. No review fix
pass was required.

The reviewer specifically examined full scope precollection, endpoint query and
HTTP-context isolation, OnceLock retry after initialization panic, retained
traffic accounting, exact URL/hash association, per-URL failure isolation,
independent archive fallback, and warm/offline bypass. It found the private
catalog's ownership and interface comments consistent with the design intent.

Author verification on the reviewed implementation: 92 normal Rust tests,
real TLS rejection/timeout regression, formatting, strict Clippy and rustdoc;
95 Python tooling tests, document links/OKF validation and strict user-site build;
45 workflow run blocks through ShellCheck and pinned zizmor with no findings.
Lean model/generator builds and both corpus freshness checks passed. The
216-case index adapter exercises both representations; parser properties use
256 bounded cases each. The normal suite also executes the existing 1,347-case
policy adapter. These are local macOS arm64 observations, not hosted CI results.
The PR records subsequently observed hosted checks.

Implementation rulings, in order:

1. Continue inline in the authorized feature checkout rather than a separate
   worktree. This preserves branch isolation, with weaker filesystem isolation.
2. Keep worker-local summary locks outside returned batches so parent recovery
   after join retains observed traffic on unwind. The parent must recover
   poisoned locks only after workers have terminated.
3. Bind the catalog to an immutable Http at construction and seal its endpoint
   map instead of passing Http per lookup and locking a dynamic map. This makes
   authentication-context ownership structural and avoids map-lock contention.
   Dynamic endpoint addition or mutable authentication would require redesign.

No deferred minor findings or unresolved review counterexamples remain.
