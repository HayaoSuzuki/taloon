# PyPI distribution design

Status: proposed for owner approval; independent builder check completed.
Prepared against taloon `587ee309` and
hoimin `39d35f3980c428ab15258b519a233b86f1551ffa` on 2026-10-31.

## Intent and owner decisions

Distribute the existing Rust CLI as the Python package `taloon`, runnable with
`uv tool run taloon check` or installable with `uv tool install taloon`.
The owner requested hoimin's packaging and origin proofs, automatic publication
after merging into main, and automatic patch numbering starting at 0.1.0.
This supersedes the initial implementation's deferred-publishing scope.

## Approach

Use hoimin's maturin binary-wheel approach (`bindings = "bin"`). It packages the
existing executable without Python bindings or runtime Python dependencies.
The requested reference already selects this approach; adding a Python API or a
Python wrapper would introduce behavior unrelated to command distribution.
Retain `publish = false` for crates.io: Python distribution does not require
publishing the Rust crate.

Match hoimin's three release targets: Windows x86_64/MSVC, Linux x86_64 GNU
manylinux2014, and macOS arm64 with a macOS 11 wheel floor. Build Linux inside
the checksum-pinned manylinux maturin image, not on the Ubuntu host. Use pinned
tool versions and action SHAs from the inspected reference, verifying them before
use. Initially publish these three wheels only, matching hoimin's PyPI inventory.
No sdist fallback or additional CPU/libc target is promised by this change.

The binary wheel is independent of Python's ABI. The owner explicitly restricted
supported installation interpreters to Python 3.14 and 3.15. Set package metadata
to `requires-python = ">=3.14,<3.16"` and smoke-test uv invocation with both
versions on each release target. Older interpreters and Python 3.16 or later
remain outside the declared range. Repository documentation tooling continues
to use Python 3.14. Keep its dependency group separate from package runtime
dependencies.

## Versions and event identity

PRs build and smoke-test preview wheels without publishing or write credentials.
Use `push` on main for releases, so workflow identity and built source both
refer to the same main commit. Do not use `pull_request_target` for publication.
Direct main pushes follow the same rule; merge protection remains repository
administration rather than a release heuristic.

Use stable `vMAJOR.MINOR.PATCH` tags. Cargo's committed workspace version is a
minimum release version. First reservation is 0.1.0; subsequent reservations
increment the latest patch unless the committed minimum is higher. Re-executing
the same commit reuses its reservation. Adapt hoimin's atomic tag/state-ref
reservation and ancestry checks, preserving immutable tags and rejecting stale
or unrelated source ordering. Test reservations with local bare Git remotes,
including collisions/retries, response loss, and an older run overtaken by main.
An overtaken older run must be explicitly skipped rather than claim publication.

Set the selected version only in the build workspace, consistently across Cargo,
Python metadata, and lockfiles. Never rewrite main just to bump a release.
Stable tags identify source; signed build evidence also identifies the workflow
and artifact digests, rather than implying that generated version edits were
committed. An explicit manual recovery dispatch from main selects an existing
main release tag and its original validated bytes; it must not rebuild different
bytes under an already-published version or silently skip mismatched uploads.
Verify original build signatures against the original source/workflow commit;
verify the recovery publish signature against its current main workflow identity.
These identities can differ legitimately and must be reported separately.

## Build, validate, prove, publish

1. Build three wheels from one selected commit/version with locked dependencies.
   Validate exact wheel inventory, package/version, Python requirement, license
   expression/text, wheel tags, and packaged executable. Run the actual installed
   wheel through `uv tool run --from <wheel> taloon`: version/help and synthetic
   offline allowed/denied/incomplete cases, preserving exits 0/1/2 and JSON.
2. Require successful CI, Documentation, and Dependency audit workflows for the
   exact main commit before publication, including Rust tests, Lean and fuzzing.
   Bound the wait and fail on missing/failed checks. Recovery verifies that the
   original source passed these checks too; PR previews cannot satisfy this gate.
3. Assemble validated artifacts and SHA-256 inventory. On main, create
   GitHub SLSA build provenance for each wheel; verify repository, workflow,
   source commit, signer commit, subject, digest, and hosted-runner identity.
   Convert these original signatures to PyPI-compatible adjacent attestations.
4. Publish those bytes to TestPyPI through OIDC Trusted Publishing, with PyPI's
   publish attestation enabled. Verify downloaded file inventory/digests and both
   SLSA and publish signatures, with bounded propagation retries.
5. Publish the identical verified wheels to PyPI and perform the same checks.
   Existing complete matching publications may be verified and reused. Partial,
   yanked, wrong-byte, wrong-source, or wrong-signer publications stop recovery;
   do not use `skip-existing` to hide disagreement.

Publishing jobs consume validated artifacts and do not execute checked-out
project code. Separate build, signing, and uploading permissions. No API token
is stored; OIDC rights exist only where signing/publishing needs them. Upload
only wheels and their supported adjacent attestation files, not arbitrary build
outputs. Keep evidence needed for recovery as GitHub release assets. Their
storage is not claimed immutable: recovery revalidates the original signed
digests and source identity and rejects replaced files. Never overwrite an
existing release's assets to repair a verification failure.

## External configuration

PyPI and TestPyPI each require a normal or pending Trusted Publisher for project
`taloon`, owner `HayaoSuzuki`, repository `taloon`, workflow `release.yml`, and
environment `pypi` or `testpypi`, respectively. Check project-name availability;
do not claim registration merely because the public JSON endpoint is absent.
Both public project endpoints returned HTTP 404 during read-only investigation
on 2026-10-31; this does not establish ownership or registrability.
GitHub environments must allow main's release jobs without mandatory human
approval if fully automatic publication is desired. Document exact setup and
verification commands. Use gh for all GitHub inspection and changes. PyPI
account-side configuration requires the owner's authenticated session.

## Implementation and acceptance

Implement in a new branch/PR, leaving the merged license-checking change intact.
Stages: binary packaging and installed-wheel checks; reservation/recovery and
artifact validation; publishing/provenance workflow and operational guidance.
Perform at least five self-review passes per stage. Add meaningful property and
failure tests around ordering, exact inventories, version rewriting, and trust
rejection. Assess whether the release state machine benefits from a bounded
Lean oracle, without claiming a proof covers network or signature libraries.

Update the existing user installation guide, developer release procedure, OKF
distribution decision and baseline, and AGENTS scope. Distinguish observed local
and hosted preview checks from a production upload. Publishing occurs only from
the merged main workflow; do not merge the PR or perform an early production
upload as part of preparing this change.

## Sources

- Local reference: hoimin's `pyproject.toml`, `.github/workflows/release.yml`, and
  `tools/release.py`, `release_policy.py`, `pypi_release.py`, and
  `pypi_provenance.py` at the revision above. Reused code retains attribution and
  its Elastic-2.0 terms.
- [maturin binary bindings](https://www.maturin.rs/bindings).
- [PyPI Trusted Publishing](https://docs.pypi.org/trusted-publishers/using-a-publisher/).
- [PyPI attestations](https://docs.pypi.org/attestations/producing-attestations/).
