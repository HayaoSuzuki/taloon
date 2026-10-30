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
executor ledger. At that checkpoint, product pipeline stages 2–8 and their reviews remained pending.


## Stage 2 locked inventory and conditional origins

Reviewed on 2026-10-31 from base `cf7ffcf`. The supported wire schema is
version 1 revision 5; other versions/revisions fail explicitly. Tests use
synthetic examples and this repository's documentation lock.

1. **Specification pass:** checked eight external identities/nine artifacts,
   dev/transitive/extra/platform branches, same-name versions/sources, local
   archives, unsupported Git/directories, and explicit/implicit workspace roots.
   Same-name registry packages remain external. No current-platform selection
   narrows the inventory.
2. **Adversarial pass:** verified ambiguous/unresolved edges and duplicate
   identities cannot invent associations. Found plain and extra-enabled visits
   to the same package could collapse. Added a failing regression and made the
   visited state include selected extras; the regression then passed.
3. **Design pass:** kept wire structs private and separated record validation,
   target construction, and annotated traversal within resolution. Added typed
   GroupKind rather than group/extra string dispatch. PEP 440 constraint meaning
   remains in the model owner. Three public operations expose no TOML nodes.
   Assessment: 8/8 diagnostics, 10/10 for this slice; no open row.
4. **Failure/context pass:** found cycles could activate a workspace member's
   development group during ordinary dependency traversal. A failing regression
   demonstrated the extra origin; development groups now remain entry points,
   while optional dependencies require selected extras. Added separate red/green
   regressions for workspace-only group origins and package resolution markers.
   Ambiguous manifest requirement hints yield no invented origin and cannot
   remove targets. The original marker fixture also needed its tied shortest
   route to carry the condition it asserted; the traversal's tie-break was valid.
5. **Verification pass:** all 22 Rust tests passed, including 11 resolution
   tests and three 256-case property tests across the library. The uv_lock fuzz
   target exercises inventory and paths from valid cyclic/extra seeds and
   arbitrary bytes; its final bounded 1,000-run campaign completed without a
   crash. Strict Clippy passed. Earlier stage-1 Lean policy expectations remain
   unchanged; graph/coverage model correspondence belongs to stage 4.

Ruling: manifest dependency-group requirements serve only as unambiguous
locked-origin hints, using existing locked versions and constraints. They do
not re-resolve dependencies. Ambiguous hints leave a missing-path explanation;
all locked targets still undergo policy checking. The risk is an unavailable
origin explanation, not permission based on missing evidence.


## Stage 3 distribution and metadata evidence

Reviewed on 2026-10-31 from base `0c2e49e`. These checks cover in-memory
interpretation; acquisition and the executable are still pending.

1. **Specification pass:** traced declaration precedence, exact identities,
   archive SHA-256, sidecar association/digest, and proof requirements through
   public constructors. Invalid declarations remain observations that a later
   clarification may replace; invalid identities and archives remain errors.
2. **Adversarial pass:** inspected folded/repeated fields, invalid UTF-8,
   historical/future metadata versions, duplicate ZIP paths, and source changes.
   Added regressions rejecting historical 1.3/2.0 and exact duplicate members,
   including ZIP readers that deduplicate their member index. No classifier or
   prose fallback grants permission.
3. **Design pass:** evidence hides ZIP/tar/header details and private proof
   state behind constructors. Records preserve full metadata digests and header
   bytes but discard Description bodies; the body-size regression confirms
   persisted record size remains constant. Eight design diagnostics were checked:
   information hiding, interface depth, responsibility, change locality,
   abstraction value, naming, invariants, and explanatory comments. No unresolved
   interface row remains in this slice.
4. **Failure/resource pass:** checked hash-before-parse, unique regular metadata
   members, gzip trailer validation, and cache corruption/source checks. Metadata
   is bounded to 4 MiB and expanded tar input to 1 GiB; exceeding a limit is an
   explicit error, never a skipped target. ZIP supports Stored/Deflate and source
   archives support gzip/tar. Archive input is currently buffered by callers.
   Cached records are trusted local observations with corruption checks, not
   signed claims from arbitrary third parties. A sidecar cannot gain strength
   through a record roundtrip.
5. **Verification pass:** all 34 Rust tests passed, including 12 evidence tests
   and five 256-case properties across stages. Metadata, wheel, and sdist fuzz
   campaigns each completed 1,000 runs without a crash. Archive targets combine
   arbitrary bytes with structure-aware repacking so valid checksums do not
   prevent header-parser coverage. Strict Clippy, rustdoc, and both shipping and
   fuzz dependency license/source checks passed. Lean policy correspondence is
   unchanged; complete-inspection correspondence belongs to stage 4.

Ruling: cache only original headers plus full-observation digests and a separate
header checksum. This avoids retaining per-artifact Description bodies while
preserving declarations and provenance. It assumes the local cache is trusted;
its checksums detect corruption and do not authenticate third-party records.
