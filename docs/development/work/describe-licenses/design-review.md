# Describe design review

Design-only record on 2026-11-01, based on repository `4c481d4`. No product code,
new CLI or executable behavior existed at that design-review checkpoint.

## Five self-review passes

1. Owner decisions: uv.lock only; display without permission policy; all matching
   identities; external normal/all-extra/all-environment closure without dev;
   project-name AllExternal mode and own/workspace-license exclusion; distinct
   artifact declarations/unknowns. Workspace-member and name-collision behavior
   is explicitly labeled an implementer proposal, not an owner quote.
2. Coverage: AllExternal uses the same full inventory as check, including
   unreachable and unsupported records. External closure includes all optional
   groups at every visited node and retains workspace routing nodes. Cycles and
   diamonds deduplicate inventory without erasing graph relationships.
3. Evidence: request only selected artifacts; retain exact identity and proof,
   complete outcome accounting and sidecar strength; use original declarations
   without clarifications/permission interpretation. Missing and invalid licenses
   stay unknown, and distinct artifact expressions are not joined into one SPDX
   expression. Exact-text grouping avoids unrequested algebraic equivalence.
4. CLI/output: uv.lock-only input, shared location/auth/cache/progress behavior,
   no required pyproject policy or evaluation date. Separate describe status and
   schema, one JSON object including startup errors, deterministic text and
   source redaction. Exit 0 complete, 2 incomplete; no violation exit.
5. Ownership and bounds: graph selection stays with resolution; description
   owns declarations/completeness; acquisition stays shared. Typed scope/results
   avoid mode Booleans in permission evaluation. Flat inventory plus annotated
   graph avoids exponential recursive rendering. Planned model/property/fuzz
   correspondence has explicit limits; no runtime dependencies are planned.

A fresh read-only Rust builder (`describe_design_builder_check`) inspected the
design and relevant code and reported “nothing important left.” This is a
planning-readiness review, not a code review or implementation verification.
No unresolved builder question or deferred finding remains at this stage.
Repository documentation/OKF validation passed locally. Behavior tests, formal
model changes, lint and hosted checks remain implementation-stage work.
