# Initial implementation stage reviews

Each stage requires at least five actual self-reviews. The active Codex agent
performed the passes below. Unlisted stages remain pending. Base documentation
commit: `e630aa0604cdc7c9fd2d997041ff96f3f1837337` on `impl-functions`.

## Stage 1 identities and policy

Reviewed the working tree on 2026-10-31 against the approved contract. The stage
adds validated package/version/date/hash identities, typed source/artifact/
permission/diagnostic domains, policy parsing/evaluation, and the initial
Lean/property/fuzz slice. There is no usable CLI or artifact acquisition yet.

1. **Specification pass:** traced AND/OR/WITH, exact package/source/version
   exceptions, hash-scoped LicenseRef permission, clarification, and inclusive
   expiry to the policy API and tests. Expanded generated expression coverage
   from four Boolean combinations to bounded recursive trees. Verified the
   identifier database is SPDX 3.29.0 (spdx 0.13.6); deprecated identifiers remain
   distinct and imprecise aliases remain disabled.
2. **Adversarial-input pass:** found malformed URLs used a raw-string fallback
   that could expose userinfo/query secrets. Added a failing fixture and changed
   the fallback to a redacted invalid-locator marker. The regression passed after
   the single fix; ordinary percent-encoded paths retain diagnostic context.
3. **Design pass:** found public locator enum construction could bypass its
   convenience constructors. A failing test demonstrated that ArtifactRequest
   accepted a malformed remote locator. Its validating constructor now checks
   locator variants and rejects invalid remote URLs/empty local paths. Kept
   policy storage private and changed override kinds to an enum. Assessment:
   8/8 design diagnostics, 10/10 for this slice; no open row. Later stages must
   maintain these boundaries.
4. **Failure/proof pass:** confirmed expiry validates unmatched entries, custom
   permission needs the exact content hash and archive-level evidence, and
   clarifications do not grant permission. Lean first rejected a false positive
   example with the intentionally incomplete evaluator. After implementation,
   its monotonicity/expiry proofs and broken AND/OR/expiry witnesses passed.
   The adapter rejects empty/malformed corpora as infrastructure failures.
5. **Verification pass:** inspected actual targeted red/green results, the full
   Rust suite, strict Clippy, rustdoc, Lean build/corpus freshness, 45 strict
   Rust/Lean cases, 256-case property tests, and 1,000-run fuzz smoke campaigns.
   The final recorded fuzz campaign used seed 788571319, max_len 4,096, and
   observed 61 MB RSS with no crash. Generated non-crash mutations live in the
   ignored executor workspace; named seeds remain versioned.

Root dependency license/source checks passed. The fuzz workspace initially
failed because libfuzzer-sys 0.4.13 declares `(MIT OR Apache-2.0) AND NCSA`.
`tools/check_fuzz_licenses.sh` derives a temporary policy from the root policy,
adding exact-version exceptions for the local taloon-fuzz package and NCSA for
libfuzzer-sys only. The fuzz graph then passed; the shipping allowlist was not
widened. This is a development-only dependency policy decision.

The latest full Rust suite had 11 passing tests (one adapter exercises 45 corpus
rows). The two properties use 256 generated cases each. Lean build completed in
4.204 seconds after adding the final sensitivity witness; the earlier initial
build was 4.792 seconds. No bound was raised and no peak Lean memory reading was
available. These are local observations, not performance-target evidence.

Rulings within delegated implementation scope:

- Share one typed Error/Code carrier with owner-specific aliases rather than
  several forwarding error wrappers; callers still distinguish categories.
- ArtifactRequest construction returns Result to enforce remote digest/locator
  invariants. The previous plan's infallible constructor cannot encode them.
- Initial tests live under tests/ rather than file-local test modules, so they
  exercise the public embedding API and the same interfaces as the oracle/fuzzer.

The authoritative completion command and commit range are also recorded in the
executor ledger. Product pipeline stages 2–8 and their reviews remain pending.
