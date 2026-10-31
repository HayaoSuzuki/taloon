---
type: Decision
title: Python binary distribution and verified publication
description: Package Rust executables with maturin and publish original verified wheels through OIDC after main checks.
status: accepted
sources:
- id: design
  resource: ../work/pypi-distribution/design.md
  working_tree: modified
  sha256: a60859db875774bc3fc7b558161be540628ff0a644f9d755a5b02aa44360fe90
- id: reference
  resource: https://github.com/HayaoSuzuki/hoimin/blob/39d35f3980c428ab15258b519a233b86f1551ffa/.github/workflows/release.yml
- id: workflow
  resource: ../../../.github/workflows/release.yml
  working_tree: untracked
  sha256: b6a8ff3295f64388a1c99f7d8540572b1d00424db86cb1c31e16fd85692b4a0c
- id: evidence
  resource: ../work/pypi-distribution/reviews.md
  working_tree: modified
  sha256: af61786e4fcfcf58950311a23e8a18f77812f4eb61676a8879b87867fcc32ab6
---
# Decision

The owner approved Python binary distribution, automatic patch numbering after
main merges, hoimin's origin proofs, Python 3.14/3.15 only, and Linux-focused
exhaustive testing with focused integration checks on each wheel platform.
This supersedes the initial deferred-publication scope.[^design]

Use maturin bin bindings and Cargo-owned dynamic Python version metadata. Ship
one wheel for each of Windows x86_64/MSVC, Linux manylinux2014 x86_64, and macOS
11 arm64. Python API bindings, wrapper processes, extra platforms, sdist and
crates.io publication add no benefit to the agreed CLI distribution scope.
Neither user Python dependencies nor Python ABI-specific wheel builds are needed.
Keep repository tooling dependencies outside the shipped package.[^design][^reference]

# Identity and trust boundaries

Use main pushes, with successful quality workflows for the same commit, as the
publication event. PRs build and exercise wheels with read-only credentials.
Atomic tag/state-ref reservations preserve commit/version identity across retries
and competing runs. A release tag identifies original source; temporary version
edits used for packaging are not claimed committed.[^workflow]

Validate exact wheel inventory, metadata, license and checksums before signing.
Retain per-wheel GitHub SLSA provenance; publish those original observations with
PyPI publish attestations through TestPyPI and PyPI. Verify downloaded bytes,
publisher, both predicates, original build source/workflow digest and main
publisher ref. Tokenless OIDC upload jobs execute no checked-out project code.
Recovery uses verified original stored assets, never a rebuild or silent
skip-existing. Storage can be replaced; signatures provide the integrity check,
not a claim that GitHub Release assets are immutable.[^workflow][^reference]

# Operations, evidence and reconsideration

The [release procedure](../releases.md) owns setup commands and account-side
PyPI/TestPyPI publisher prerequisites. [Stage evidence](../work/pypi-distribution/reviews.md)
distinguishes observed local/hosted checks from production publication. Configured
automation is not evidence that an upload or account registration occurred.[^evidence]

Revisit the inventory when supporting another architecture/libc, Python version,
sdist or Python API. Revisit identity checks when changing trigger, workflow,
repository, signing provider or recovery source. Keep vulnerability/license gates
and exact-source checks when changing CI organization.[^design][^workflow]

[^design]: [Approved owner decisions](../work/pypi-distribution/design.md).
[^reference]: [Inspected hoimin workflow](https://github.com/HayaoSuzuki/hoimin/blob/39d35f3980c428ab15258b519a233b86f1551ffa/.github/workflows/release.yml).
[^workflow]: [Release implementation](../../../.github/workflows/release.yml).
[^evidence]: [Recorded checks and review passes](../work/pypi-distribution/reviews.md).
