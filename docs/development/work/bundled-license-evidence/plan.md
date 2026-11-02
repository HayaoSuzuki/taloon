# Bundled license evidence implementation plan

> For agentic workers: implement through executing-plans or subagent-driven-development, with red/green tests and five review passes per stage.

**Goal:** Resolve absent declarations from verified bundled license text without network license databases or repeat archive downloads.

**Architecture:** Pure bounded archive readers and an exact template matcher live inside evidence. Acquisition retains ownership of network/cache decisions. Existing explicit declarations retain precedence, including invalid/repeated declarations. The owner selected recommended defaults and requested implementation through PR.

**Tech stack:** Rust, existing spdx canonical texts, serde, ZIP/tar; rstest/proptest and existing HTTP fixtures.

## Design and alternatives

Use local complete-text matching for an initial supported set (MIT, BSD-2-Clause,
BSD-3-Clause, ISC, Apache-2.0). Permit narrowly documented template copyright/holder
and whitespace variants; reject changed terms, unknown text, addenda and ambiguous
multi-license combinations. This narrower initial coverage avoids fuzzy scores
silently accepting modified obligations. All unrecognized cases retain Unknown
and hash-bound clarifications. AUTHORS is an attribution file, not a license
candidate; retain its provenance alongside the license files. Other unrecognized
declared files block inference. An explicit License-Expression or License always
prevents file fallback, so invalid declarations cannot be hidden by permissive files.

Read declared License-File paths inside the same verified distribution. If absent,
try conventional LICENSE/LICENCE/COPYING names only in its metadata namespace or
sdist root. Do not scan arbitrary package source or query GitHub/PyPI project JSON.
Archive hashes still require the full locked archive; HTTP ranges cannot prove its
SHA-256 without downloading all bytes, and add round trips. Keep the existing
small metadata fast path for valid declarations. Only absent declarations trigger
one archive fallback. Persist raw bounded observed files, not a matcher verdict,
and re-evaluate them under the current matcher on reuse.

Preserve existing record serialization/checksums when the optional file observation
is absent. Online old/sidecar cache entries needing files upgrade directly to an
archive without refetching index/sidecar metadata. Offline stays network-free.
Completed negative scans are cached to avoid repeat futile downloads. Do not reuse
file observations at sidecar strength. Limits: 32 files, 256 KiB each, 1 MiB total;
existing archive, metadata and tar-expansion limits remain. Cache bounds include
JSON expansion of the additional bytes. Keep archive and sidecar provenance distinct.

## Tasks

- [x] Pure matching: add named red/green cases and independent generated variants;
  implement conservative complete-template matching in evidence/text_match.rs.
- [x] Pure extraction: archive_files.rs reads only safe selected paths, rejects
  missing/ambiguous/nonregular entries and bounds before allocation; named tests
  cover wheel/sdist, path attacks and resource limits.
- [x] Evidence/cache: absent-field-only fallback, raw file snapshots, checksummed
  restoration, exact provenance and unknown behavior; public API regressions.
- [x] Acquisition: cache-first selective archive escalation; HTTP counter cases
  prove valid metadata retains its fast path, absent metadata fetches one archive,
  completed warm scans use no network, and offline never fetches.
- [x] Docs, five-pass reviews and strict local quality/full/generated/fuzz checks.

Delivery: create the feature PR and record hosted results separately on it.

## Review focus

1. Added or negated license conditions cannot disappear during normalization.
2. Missing/duplicate declared files and misleading roots cannot narrow scope.
3. Explicit invalid/repeated declarations cannot fall back to more permissive text.
4. Warm/negative/old-cache and offline states must avoid unnecessary requests.
5. File evidence must remain archive-verified and survive corruption/size checks.

Design assessment: 8/10 initially; evidence acquisition did not expose whether a
cached observation still needed license files. A single evidence-owned query and
optional raw file observation close that information leak without a new transport
layer. Final assessment requires integration tests and review.
