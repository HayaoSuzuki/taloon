# Third-party refresh (2026-11-24)

The requested policy is the newest compatible stable release, preferring an
upstream LTS when available. Resolve versions from official registries and
release tags; keep Actions and container images pinned by immutable identities.
Rust has no upstream LTS channel; development now uses stable 1.100.0.
Python remains on the supported 3.14 line; CI resolves its current patch.

The failing dependency-audit job reported RUSTSEC-2026-0592 in url 2.5.8.
Both shipping and fuzz lockfiles now select 2.5.9. Rust dependencies include
html5ever 0.40.1, sha2 0.11.1 and rstest 0.29.0. html5ever now provides both
byte and string AsRef implementations; explicit string dereferencing preserves
the existing tokenizer comparisons without changing HTML semantics.

Actions releases were checked against each repository's latest release.
setup-uv changed; attest already used the v4.2.2 SHA and its comment now names
that exact release. Other referenced releases were current.
The maturin 1.16.0 image digest was obtained from the official OCI index.
Python dependency pins and tool pins were checked against PyPI and crates.io.

## Coordinated Creusot environment migration

Creusot and creusot-std now use 0.15.0, with Z3 5.3.2 and the matching hoimin
recipe at 242ed683577179e641b80c82f2b2a0cd40c9bb61. The old 0.14.0 image cannot
satisfy the new builder identity. The committed pin explicitly selects bootstrap.
After merge the trusted-main environment publisher builds and attests the new
image; its actual digest can then be adopted in a follow-up. No PR-produced
image is represented as trusted-main provenance.

## Validation

Rust stable workspace tests and strict Clippy passed with 1.100.0.
Python: 301 tests passed; Ruff formatting/lint and ty passed.
Latest cargo-audit 0.23.0 reported no vulnerabilities in the shipping lockfile.
Workflow shell checks and zizmor 1.33.1 passed. Documentation references and
strict user-site build passed. Local execution is macOS arm64; no hosted CI
run or publication is claimed.

Lean 4.36.0 rebuilt all 30 targets and all three corpus freshness checks passed.
The pinned nightly shuffle suite passed. Rust formatting and rustdoc passed.
cargo-deny 0.22.0 accepted licenses/sources for shipping and fuzz graphs;
cargo-audit 0.23.0 found no vulnerabilities in either graph.

The 0.15 Seq API renames the pure append operation to `snoc`; all three
logical expressions use it. No postcondition, invariant, proof inventory or
negative witness was removed. Exact-version LGPL exceptions were updated only
for the verification-only 0.15 crates, whose declarations retain the same license.

Local proof verification initially uses the existing `hoimin-creusot-015-arm64`
image. Its manifest differs from the pinned recipe only in apt snapshot
(20261018 rather than 20261117); all tool artifacts, versions, receipts and
solver/config hashes match. This is separate from the new recipe build check.

All 14 compiling negative witnesses were rejected by Creusot 0.15.0.
The Seq migration uses the pure `snoc` operation, matching the new Vec push
contract. A trial concat/singleton expression left one normalization VC unsolved;
using the named pure operation restored the existing proof without weakening it.

Creusot 0.15.0 fresh verification passed all 21 obligation files on each of
x86_64-unknown-linux-gnu, aarch64-apple-darwin and x86_64-pc-windows-msvc.
The verifier script exited successfully. All inventory/goal checks and input
fingerprint checks stayed enabled.

The new pinned ARM64 recipe image build completed successfully. CI exposed a
stale revision literal in the builder regression; it now expects the new pin.
All four orchestration suites were rerun with independently checked exit codes.

maturin 1.16 emits the same two manylinux compatibility tags in reverse order.
The exact published identity and attestation paths now match that emitted name.
The fixed platform regression failed before the policy update and passed after.

The [image adoption record](image-adoption.md) completes the trusted-main
publication/digest transition and records the required-check context alignment.
