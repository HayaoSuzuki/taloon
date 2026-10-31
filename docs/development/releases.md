# Building and publishing Python wheels

The [distribution decision](knowledge/python-distribution.md) owns release
semantics and trust boundaries. Shipping code remains Rust; maturin's `bin`
bindings package its executable, with no Python runtime package dependencies.
Supported installers use Python 3.14 or 3.15. Each platform wheel is built once
and exercised by both versions. Linux owns exhaustive Rust/property/Lean/fuzz
suites; platform smoke checks own uv startup, exits, paths, and cache integration.

## One-time publisher configuration

Before merging the first release workflow, register a pending publisher for
`taloon` on both [PyPI](https://pypi.org/manage/account/publishing/) and
[TestPyPI](https://test.pypi.org/manage/account/publishing/):

| Field | Value |
| --- | --- |
| PyPI project name | `taloon` |
| Owner | `HayaoSuzuki` |
| Repository | `taloon` |
| Workflow filename | `release.yml` |
| Environment | `pypi` on PyPI; `testpypi` on TestPyPI |

If the project already exists under your account, add the same publisher to
that project instead. Public JSON HTTP 404 does not reserve a name or establish
that registration is available. This account-side step cannot be completed with
GitHub credentials. Follow [PyPI's setup procedure](https://docs.pypi.org/trusted-publishers/creating-a-project-through-oidc/).

Create GitHub environments `pypi` and `testpypi`, allowing deployment only from
branch `main`. Do not require a reviewer if publication must run automatically.
No PyPI API token or secret is needed. Keep the repository's default workflow
token read-only; only individual reservation/signing/upload jobs request writes.
Required main checks also need to remain enabled. Branch protection and approval
policies for merging are separate repository administration.

## Normal publication

The [release workflow](../../.github/workflows/release.yml) previews all three
wheels on PRs without publishing. A main push reserves a stable tag and ordering
ref atomically, builds/smokes the wheels, and requires successful CI,
Documentation and Dependency audit runs for that exact main push.

The first version is the committed Cargo workspace version (initially 0.1.0).
Later reservations use the greater of the manifest floor and latest stable
version with its patch increased by one. Raising the committed floor selects a
new minor or major line. Re-executing a reserved commit reuses its tag; stale
unreserved sources are skipped. Version edits affect only the build workspace;
the tag continues to identify the original source commit.

After validating the exact inventory and each wheel's metadata/license, the
workflow signs per-wheel GitHub build provenance and retains the wheels and
SHA256SUMS as a GitHub Release. It converts the original signatures, uploads
to TestPyPI, verifies downloaded bytes and both build/publish predicates, then
uploads the same wheels to PyPI and verifies again. Index visibility waits are
bounded at ten minutes. A partial/mismatching or yanked inventory stops the run.
It never treats an upload conflict as proof of a matching publication.

## Local reproduction

Use the repository's pinned Rust compiler, native C toolchain, Python 3.14 and uv:

```sh
uv sync --frozen --only-group docs --only-group release --no-install-project
uvx --from maturin==1.15.0 maturin build --release --locked --out wheelhouse
python -m tools.wheel_smoke wheelhouse/taloon-VERSION-PLATFORM.whl --python 3.14
python -m tools.wheel_smoke wheelhouse/taloon-VERSION-PLATFORM.whl --python 3.15
uv run --frozen --no-sync pytest
tools/check_workflows.sh
```

Use the actual wheel filename. Linux release builds use the digest-pinned
manylinux2014 container in the workflow; a host Linux build is not evidence of
manylinux compatibility. Windows MSVC x86_64 and macOS 11 arm64 use native
builders. Additional architectures/libcs and sdist publication are outside this
release inventory. crates.io publishing remains disabled.

## Recovery and provenance verification

Fix the external cause (for example a pending publisher registration), then
dispatch from main with the reserved stable tag:

```sh
gh workflow run release.yml --repo HayaoSuzuki/taloon --ref main -f pypi_tag=v0.1.0
```

Recovery requires an existing published stable GitHub Release, an original
source tag that is an ancestor of current main, successful main checks for that
source, exact artifact inventory, and valid original per-wheel build signatures.
It downloads and revalidates those assets; it does not rebuild or replace them.
Release assets are storage, not an immutability guarantee. Signature verification
rejects substituted bytes and repository/workflow/source disagreement. The
original source/workflow digest and the current main recovery publisher identity
are deliberately different checks.

A failure before a GitHub Release exists can be retried by rerunning the original
main workflow. A partially uploaded GitHub Release is not repaired automatically;
inspect it before taking an explicit corrective action. Concurrent publication
attempts can fail on an upload conflict; rerun recovery to verify an existing
complete publication. A failed or missing quality check must be resolved through
its original main workflow, not replaced by PR check results.

For independent index verification, stage the trusted original wheels in `dist`:

```sh
uv run --frozen --no-sync python -m tools.pypi_provenance verify \
  --index pypi --tag v0.1.0 --commit FULL_ORIGINAL_COMMIT_SHA --directory dist
```

This performs cryptographic verification and byte/inventory comparison. Neither
attestations nor the small Lean reservation model establish that all upstream
license declarations are legally complete.
