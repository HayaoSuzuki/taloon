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

## Stage 4 complete inspections and reports

Reviewed on 2026-10-31 from base `44c32dc`.

1. **Specification pass:** traced each expected request through missing, failed,
   weak, permitted, and denied results. Configuration errors and target issues
   remain incomplete; clarification replaces a declaration without granting
   permission. Empty validated scope succeeds. Both original and interpreted
   declarations and override reasons remain reportable.
2. **Adversarial pass:** inspected duplicate/unknown results, shared bytes under
   separate requests/sources, and observation rebinding. Found cached evidence
   could change Wheel to Sdist without rechecking the format. A failing public
   evidence regression confirmed this; records now bind the format and the
   regression passes. Missing either shared-content association still fails.
3. **Design pass:** checked all eight diagnostics: information hiding, interface
   depth, responsibility, change locality, abstraction value, names, invariant
   representation, and comments. Inspection owns accounting; Evidence owns
   rebinding; Policy owns unused-entry analysis; Report owns exit precedence.
   Wire policy internals and acquisition implementation remain hidden. No open
   design row remains. Reports expose typed statuses/diagnostics and private
   run status, with counters independent of policy.
4. **Failure/proof pass:** checked UTC configuration validation, conservative
   unused warnings (all OR terms count), weak proof, and incomplete-over-warning
   precedence. Added a failing regression for failures with no proven origin;
   reports now explain absent paths without fabricating an edge. Lean proves
   missing-target rejection over arbitrary lists and exit precedence, and checks
   proof sufficiency. The finite generator detects deliberately broken sidecar,
   missing-result, and exit variants. These proofs do not cover Rust I/O.
5. **Verification pass:** 42 Rust tests passed, including seven inspection tests,
   six 256-case properties across stages, and all 477 strict Lean-generated
   cases (432 inspection plus the previous 45). The structured inspection fuzzer
   completed 1,000 executions without a crash. Strict Clippy/rustdoc, corpus
   freshness, documentation validation, and fuzz dependency checks passed.
   Lean builds stayed under the original 20-second deadline; no bounds were
   increased and no peak Lean memory measurement was available.

Ruling: duplicate result associations are rejected even when their observations
are identical. Acquisition must deduplicate internal work while returning one
association per request. Observation rebinding reparses a trusted record under
its expected identity/digest/format; IDs alone never constitute proof.


## Stage 5 local acquisition and recoverable cache

Reviewed on 2026-10-31 from base `15332ec`.

1. **Specification pass:** verified local paths are read without executing code,
   current bytes are hashed before every cache lookup, pinned mismatches fail,
   and a missing file cannot use an old observation. No policy/date/verdict is
   persisted. Sidecar-only records cannot satisfy stronger policy requirements.
2. **Adversarial pass:** traced truncated/incompatible JSON, interrupted temporary
   files, wrong identities, wrong formats, and alternate sources. Corrupt entries
   miss; global archive records still require digest/name/version/format matches;
   sidecars retain source association. The fuzz lock needed regeneration after
   tempfile became a shipping dependency; the locked license check detected it.
3. **Design pass:** checked the eight design diagnostics against the cache/local
   acquisition boundary. Cache lookup now receives the freshly observed digest,
   making the no-lock-hash local invariant explicit. Evidence retains proof
   validation ownership; cache owns paths/atomic publication; local acquisition
   owns byte reads and counters. No open design row remains in this slice.
4. **Failure/concurrency pass:** ran two concurrent writers and concurrent reads,
   checked temporary-file cleanup and single final publication, and tested an
   unavailable cache represented by a regular file. Successful in-memory
   evidence survives cache failures with warnings. Publication flushes/syncs a
   same-directory temporary file before atomic rename. This prevents publishing
   partial records; it does not promise survival across filesystem/power loss.
5. **Verification pass:** seven binary acquisition tests and 42 library/public
   tests passed, including a new 256-case persisted-record/mutation property.
   Strict Clippy and shipping/fuzz license/source checks passed after lock
   regeneration. Concurrent filesystem tests are operational evidence, separate
   from Lean's pure completeness/permission proofs. No network is used here.

Rulings: cache.get takes the current content digest explicitly so unpinned local
files cannot reuse a stale path-based record. Archive input is bounded to
256 MiB and oversized files produce an explicit acquisition error. The binary
entry point remains a deliberate exit-2 placeholder until stage 7; acquisition
is compiled/exercised through binary unit tests during this intermediate stage.


## Stage 6 HTTPS acquisition

Reviewed on 2026-10-31 from base `e86f30f`.

1. **Specification pass:** traced exact locked URL/digest matching through JSON
   and HTML Simple API current/legacy metadata advertisements. Unhashed,
   unavailable, corrupt, or ambiguous sidecars fall back to full verified
   archives. A stronger clarification fetches archive evidence online and fails
   incomplete offline when only sidecar evidence exists. No substitution,
   dependency resolution, installation, or inspected code execution occurs.
2. **Adversarial pass:** tested signed queries, percent encoding, fragments,
   relative/base URLs, script-contained pseudo-links, conflicting hashes,
   malformed/oversized responses, and one failure alongside a success. Found
   sidecar records could be rebound to a different URL with identical digest and
   source. A failing public regression confirmed this; sidecar provenance now
   binds the original request as well as its source. Archive observations remain
   reusable after identity/digest/format checks.
3. **Design pass:** checked the eight design diagnostics. Acquisition hides
   authentication, index formats, retries, concurrency, and persistence from the
   core. An opaque Acquirer hides its boxed private transport seam rather than
   exporting transport generics to the CLI. Shared-content jobs stay in one
   worker, but every request receives its own association. No open interface row
   remains. Evaluations and cache entries contain no policy verdict.
4. **Failure/security pass:** checked offline branches before any transport call,
   same-origin credentials, permanent credential removal after an origin change,
   HTTPS-only redirects, and bounded retries. An actual loopback test exercised
   invalid TLS and stalled handshakes using the real strict client. It passed
   separately because loopback sockets require execution outside the sandbox.
   Fixture transports separately cover redirects and authentication without
   claiming to prove the TLS stack. No credentials/query values appear in errors
   or cache provenance; request fingerprints preserve association without secrets.
5. **Verification pass:** 43 public/library tests and 16 binary tests passed;
   one ignored-by-default loopback test also passed explicitly. Eight properties
   across stages each run 256 cases. Strict Clippy/rustdoc, shipping/fuzz
   license/source checks and regenerated fuzz lock passed. Existing 477 strict
   Lean cases still match; network/cache concurrency is outside the pure proof.

Selected defaults: eight workers, 30-second request and 10-second connect
limits, up to three attempts for transport errors/429/5xx with 50/100 ms delay,
up to five redirects (six responses), 16 MiB index, 4 MiB metadata, and 256 MiB
archive responses. The client library's own retries are disabled so counters
reflect application attempts and redirect hops. Byte counters measure received
response bodies; cache hits and extraction attempts are observed at their owners.

Authentication uses host-specific machine/login/password entries from the
selected UTF-8 netrc file (1 MiB limit). Quoted values, comments, Unicode, and
account fields parse through shlex; account is ignored. Default credentials are
not sent, macros/unknown fields are rejected, and URL userinfo is stripped rather
than becoming Authorization. Environment path precedence belongs to CLI stage 7.

Dependency review selected reqwest 0.13.5 with rustls, html5ever 0.35.0 tokenizing,
and shlex 2.0.1. Inspected locked declarations and packaged license notices for
TLS dependencies. deny.toml grants exact-version ISC/BSD-3-Clause/CDLA permissions
for six affected TLS/certificate packages, retaining the global allowlist.
An older netrc parser had Unicode indexing limitations; another required nightly
and logged raw input. Neither is retained. The final small credential schema
uses the existing quoted-token lexer and emits only fixed safe errors.

## Stage 7 usable CLI and user guidance

Reviewed on 2026-10-31 from base `5a506d9`.

1. **Specification pass:** traced nearest-lock discovery, exact --project,
   required sibling configuration, full-scope inspection, text/JSON, UTC date,
   --offline, and --deny-warnings through real child processes. Startup failures
   use the same versioned report envelope. Report owns exit precedence; the CLI
   adds the actual process exit_code without recomputing license decisions.
2. **Adversarial pass:** tested invalid arguments/configuration, absent local
   artifacts, credential-bearing URLs, signed query redaction, and terminal
   control characters. Compared user guidance with archive interpretation and
   found non-symlink special ZIP members were accepted as metadata. A failing
   FIFO-mode regression confirmed the mismatch. Wheel metadata now rejects
   non-regular Unix member types; the regression passes.
3. **Design pass:** checked the eight diagnostics against the CLI/core boundary.
   Argument handlers own setup and rendering; Inspection, Policy, Evidence, and
   acquisition retain their existing responsibilities. Date/environment are
   obtained at runtime; no filesystem/clock/network calls enter the library.
   Typed clap subcommands/format values and the existing status/code enums avoid
   string dispatch for semantics. No unresolved design row remains.
4. **Process/operations pass:** sentinel uv/Python executables and executable
   setup.py in an sdist stayed unexecuted. Lock/config bytes stayed identical.
   Cache warmed through the CLI continued to re-evaluate a changed policy,
   without network or archive extraction. NETRC explicitly precedes the home
   fallback, a missing explicit path does not fall back online, and offline
   checks need no credentials. The hook descriptor and documented local hook
   preserve whole-lock coverage; runner integration belongs to stage 8.
5. **Verification pass:** 69 Rust tests passed: 17 binary, eight CLI, and 44
   other public tests. Eight prior properties each use 256 cases; a new 32-case
   child-process property independently checks report/exit consistency.
   Both documented policy snippets parse through the real Policy API. Strict
   Clippy, root/fuzz dependency checks, documentation validation, and strict
   MkDocs build passed. Local cargo install succeeded into an isolated prefix
   on Darwin arm64, Rust 1.99.1 and Apple clang 17.0.0. Initial release installation
   took 22.27 seconds; final source replacement took 6.04 seconds. No CMake,
   Python, uv, Lean, or fuzz tool is needed to run the installed checker.

Rulings: argument errors use a fixed safe message pointing to --help; input
reading bounds are explicit in the user guide. CLI JSON adds process exit_code
to the library's serialized Report. Publishing remains deferred. Tests of the
installed executable, hooks, CI wiring, and performance continue in stage 8;
these stage-7 results do not establish that final integration checkpoint.

## Stage 8 complete workflow and final verification

Reviewed on 2026-10-31 from base `a4914e4`. The active Codex implementation agent
performed the following five passes; the separate fresh-context review is
recorded in [the independent review](final-review.md).

1. **Specification trace:** checked every planned pipeline boundary against the
   implemented public library and CLI, including whole-lock dev/transitive/group/
   extra/platform inventory, every recorded artifact, exact policy/override
   scope, typed proof requirements, UTC expiry, and 2-over-1 exit precedence.
   Found text failures omitted individual unit permissions already available
   in JSON. A failing CLI regression now confirms the full AND/OR expression
   and each permitted/denied unit are explained. A separate failing regression
   confirmed repeated reasons lacked unique override identities; reports now
   identify each exception array entry and hash-bound clarification.
2. **Adversarial interactions:** found the blanket LicenseRef archive requirement
   overconstrained an ungranted custom OR branch. A public failing regression
   now permits the ordinary MIT alternative and classifies an ungranted custom
   expression as a violation. Expanded generated correspondence to cover this
   interaction. Found empty editable/virtual locations could disappear through
   workspace-root matching; a failing success-versus-exit-2 regression now fails
   input validation before membership classification.
3. **Design diagnostics:** checked all eight software-design-philosophy rows
   against the final module/API boundaries. The cache collision and source
   diagnostic fixes preserved caller simplicity. Metadata acquisition and cache
   bounds now derive from the core's one public metadata-size contract rather
   than independent constants. Current assessment is 8/8, 10/10 by that rubric,
   with no unresolved diagnostic row; this is a design assessment, not a
   correctness proof. Detailed evidence appears below.
4. **Crash/offline/credential/resource pass:** the independent reviewer reproduced
   same-content sidecars overwriting one weak key. A failing warm/offline test
   now retains two request associations and yields two cache hits with zero
   network. Regraded the missing-source diagnostic Important and verified safe
   structured/text source context. A supported 4 MiB metadata record expanded
   beyond the former 8 MiB JSON read limit; its failing roundtrip now succeeds
   with a derived six-fold escape bound plus envelope headroom. Buffered writes
   still flush and sync before atomic publication. Existing interruption,
   concurrency, local rehashing, strict HTTPS, redirect-auth isolation, and
   offline tests remain green. None of these operational tests are a Lean proof.
5. **Actual verification:** all 77 normal Rust tests passed, plus the separately
   invoked real-TLS/timeout fixture and opt-in release process benchmark. The
   nine properties comprise eight 256-case library/binary properties and one
   32-case CLI property. All 1,347 current Lean-generated cases matched public
   Rust behavior; kernel proofs and corpus freshness passed. All six fuzz
   targets completed 1,000 executions each without a crash (4,096-byte smoke
   limit). Strict formatting, Clippy, rustdoc, root/fuzz licenses and sources,
   28 Python documentation-tooling tests, OKF/link validation, strict MkDocs,
   workflow lint, and hook-manifest validation passed. RustSec database revision
   `2295466c4e9df26647fd67d8fa1c705ac2fc7f81` loaded 1,395 advisories; cargo-audit
   0.22.2 reported no findings for the 257-package root lock and 243-package fuzz
   lock. No advisory was suppressed. The local installed binary passed the
   documented hook/CI-equivalent process test with prek 0.5.7.

Final release measurements are in [the performance record](performance.md):
1,000 packages/10,000 distinct lock-pinned local archives, five warm process
runs of 0.608586, 0.553469, 0.562964, 0.575524, and 0.565200 seconds. Every warm
report had 10,000 hits and zero HTTP requests, downloaded bytes, and extractions.
The two-second goal was met locally; elapsed time is not a CI assertion.

### Final design diagnostic evidence

| Diagnostic | Assessment | Evidence |
| --- | --- | --- |
| Describe each module in one sentence | Pass | Module docs and the architecture ownership table identify each owner's knowledge |
| Interface simpler than implementation | Pass | Inspection exposes construction/requests/evaluation; archive/index/cache formats stay behind their owners |
| Change implementations without callers | Pass | Weak cache keys, buffered writes, ZIP handling, and transport layout changed without policy/CLI protocol redesign |
| Interface comments describe abstraction | Pass | Evidence documents proof/trust; Inspection documents exact result accounting; source summaries document safe display context |
| Discuss design in reviews | Pass | All eight stages reserve an actual design pass and record concrete changes |
| Hide important design decisions | Pass | Policy hides selectors/AST; evidence hides member/header interpretation; acquisition hides authentication/persistence/concurrency |
| Understand boundaries without implementation | Pass | Module docs, public typed interfaces, and the linked ownership decision state responsibilities and limits |
| Invest in design improvement | Pass, qualitative | Every stage includes a separate design pass and traced improvements; the wall-clock percentage was not instrumented |

The initial failed design interactions were corrected in their owning modules;
no new compatibility framework, package installation, backend execution, or
publishing work was introduced. Review exclusions were explicitly ruled on in
[the independent review](final-review.md). No established Minor finding remains
deferred. CI is configured with pinned tooling and the checked Lean asset digest;
GitHub-hosted execution remains unobserved until the branch is pushed.

### Recorded implementation rulings

These historical executor decisions record reasons and costs within the owner's
delegation. Current operational guidance remains in its existing user/developer
owners; interim-stage restrictions below describe their recorded checkpoints.

- Work in place on the existing feature branch impl-functions — preserve the owner's current-branch workflow and shared workspace — changes remain isolated from main but not from this checkout.
- use public integration tests instead of file-local tests — the oracle/fuzzer exercise the same API — task filters become integration targets.
- ArtifactRequest::new returns Result to validate remote URL/hash requirements — an infallible constructor admits invalid requests — existing clients must handle validation errors.
- share Error/Code with owner aliases — avoids forwarding wrappers while preserving typed diagnostic codes — callers distinguish categories rather than wrapper types.
- development fuzz license policy derives from shipping policy with exact libfuzzer-sys 0.4.13 NCSA permission and local fuzz package ELv2 — required by the explicitly requested fuzzing — review the exception before upgrades; shipping permission is unchanged.
- support only fixture-backed uv.lock version 1 revision 5 initially — explicit compatibility is safer than assuming historical wire formats — older locks fail with a format diagnostic.
- use unambiguous manifest requirements only as origin hints; unresolved hints never remove locked targets — avoid re-resolution or invented paths — some origin explanations may be unavailable.
- metadata limit 4 MiB, expanded tar limit 1 GiB, Stored/Deflate ZIP and gzip/tar only — bounded parsing and explicit incomplete outcomes — oversized or other archives remain unsupported.
- preserve metadata headers/full digest/header checksum without Description bodies — avoid repeated body retention — local cache remains trusted persistence rather than authenticated external proof.
- reject all duplicate result associations, even identical observations — a request must have exactly one producer result — runtimes must deduplicate internally before returning a batch.
- rebind every observation through Evidence.from_record and bind record format — IDs alone cannot authenticate proof — small repeated checksum/parsing cost.
- cache.get receives observed content hash — local files without pinned hashes must be rehashed — extra local reads remain on warm runs.
- buffer archives up to 256 MiB; atomic synced temporary cache publication without power-loss durability — bounded resource use and no partial published record — larger archives yield incomplete and cache may be lost after power failure.
- compile acquisition under binary unit tests until CLI stage — keep placeholder explicit without unused production entry points — end-to-end validation remains pending until stage 7.
- select netrc path environment precedence in CLI stage 7, pass the selected path into acquisition — environment belongs to runtime setup — precedence integration remains pending until stage 7.
- host-specific netrc quoted tokens via shlex; ignore default/account, reject macros/unknown fields, strip URL userinfo — avoid credential forwarding and unsafe parser dependencies — unsupported netrc extensions need configuration changes.
- opaque boxed transport and grouped content jobs across eight workers, three attempts (50/100 ms delays), five redirects, 30/10-second request/connect limits — bounded work and stable measured counters — worst-case failure latency can be long.
- exact-version ISC/BSD-3-Clause/CDLA permissions for six TLS/certificate dependencies after declaration/notice review — HTTPS needs these dependencies — upgrades require a new license review.
- bind cached sidecars to exact request plus source; verified archives stay content-reusable — index advertisements are artifact associations — fewer sidecar cache hits for same-content distinct URLs.
- JSON CLI envelope flattens Report and includes the actual exit_code — record warning escalation without duplicating verdict logic — library serde output has no process-specific exit field.
- fixed safe argument-error message with --help pointer, 64 MiB lock and 16 MiB pyproject input bounds — prevent locator disclosure and unbounded input buffering — less detailed argument errors and oversized documents yield exit 2.
- require archive proof for applicable custom grants/clarifications, not merely the occurrence of an unpermitted LicenseRef term — preserve explicit OR semantics and valid-expression violation status — ungranted custom units are denied rather than treated as missing evidence.
- re-grade the reviewer's missing-source Minor as Important — same-name/version unsupported records need an actionable redacted source — reports gain additive source fields and text context.
- include RequestId in sidecar cache filenames while archive keys remain content-reusable — exact associations must coexist — old weak entries become misses rather than being migrated.
- run pinned actionlint core and standalone ShellCheck over every Bash run block via one helper — combined subprocess integration stalled on macOS while both standalone checks passed — the helper rejects unsupported shells and requires the existing docs Python environment for YAML.
- derive cache-record bounds from the shared 4 MiB metadata contract (six-fold JSON expansion plus 64 KiB envelope headroom), and buffer writes before sync/rename — every supported observation must survive persistence — an encoded record may reach about 24 MiB, with additional parsing/checksum allocations.
- retain impl-functions with committed work, without merging/pushing/creating a PR — the owner requested current-branch commits and publishing is deferred — integration remains the owner's next decision. Superseded by the later explicit gh/PR handoff authorization below.
- accept the reviewer's five set-aside items only within their recorded scope — publishing/compatibility, URL-userinfo auth, older lock formats, more archive encodings are approved exclusions; final evidence updates are completed — widening any of these needs new implementation and verification.

- identify exceptions by their one-based policy array position and clarifications by content hash — distinguish repeated reasons and support audit lookup — reordering exception entries changes their reported IDs.
- the owner's later request authorizes pushing and creating a PR using gh after completion, superseding retain-only handoff — carry the implementation through the requested review artifact — merge and package publishing remain outside this handoff.
