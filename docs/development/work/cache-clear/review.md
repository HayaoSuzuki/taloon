# Cache clear verification and review

## Local verification

On 2026-11-01, the implementation based on `ec558b4` passed 99 normal Rust tests
on macOS arm64, formatting, strict Clippy/rustdoc, repository documentation/OKF
validation and the strict user-site build. No dependency or workflow changed.
The Linux-only invalid-UTF-8 filesystem case is not part of that local count.
Hosted checks and later independent review are recorded separately.

The first three CLI regressions failed with exit 2 instead of 0 before the
command existed. They then passed. The empty TALOON_CACHE_DIR regression
reproduced zero removals instead of one before normalization to dot; it passes
and preserves project inputs after the fix. Real archive and sidecar evidence
producers are exercised; a 256-case property generates all association/format
variants and checks that neighboring unknown names survive actual deletion.

## Five self-review passes

1. Deletion scope: only complete canonical v1 filenames and immediate regular
   files qualify. No recursive removal, prefix-only matching, content parsing,
   project discovery, credential read or HTTP initialization is used.
2. Cache identity: real archive/sidecar publications both clear. Corrupt records
   clear without trusting their contents; unknown schemas/temporary files remain.
   Empty overrides match existing writes, pinned by a reproduced regression.
3. Filesystem boundaries: matching directories and entry links remain; root
   links and files fail. Non-UTF-8 names are retained by checked conversion.
   The actual non-UTF-8 entry test is Linux-only: local fixture creation returned
   EPERM on macOS before the production path was entered.
4. Errors and concurrency: missing root and repeated clear succeed without
   creating the root. Vanished entries do not count. Other I/O failures return
   failure without claiming rollback. Active publishers may repopulate records;
   docs require stopping concurrent checks and claim no atomic clear.
5. Architecture/output: cache owns recognition/deletion; CLI shares directory
   resolution and renders only a count or safe diagnostic. No library I/O or
   new dependency. Check JSON behavior remains intact; cache clear uses text.

Software-design-philosophy assessment: 7/8 diagnostics supported, about 8.75/10.
Existing cache ownership hides naming, formats and filesystem decisions behind
clear(); no generic cache-management framework or new configuration was needed.
Comments state scope/concurrency rather than restating operations. Design effort
was not timed, so the 10–20% effort diagnostic is unverified; future time tracking
would substantiate that row. The behavior is covered by real filesystem/property
checks rather than adding a model that cannot establish filesystem ownership.

## Implementation rulings

1. Continue inline on a dedicated branch from merged main, respecting the
   existing session preference. This gives branch isolation without separate
   filesystem isolation from other local edits.
2. Exercise real sidecar publication through the binary cache adapter, avoiding
   network setup for process tests. CLI tests use real archive publication and
   a damaged sidecar record; sidecar publication is not observed through a live
   network process test.
3. Restrict invalid-UTF-8 filesystem entry creation to Linux, where hosted CI
   exercises it. macOS local verification cannot establish this input's behavior
   because fixture creation returned EPERM.
