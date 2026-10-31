# PyPI distribution implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan inline, task by task.

**Goal:** Publish verified Rust binary wheels through TestPyPI to PyPI after main merges.

**Architecture:** Maturin packages the existing binary. Release tooling owns
versions, exact inventories, signatures, and index verification; the workflow
owns permissions and execution order. Normal exhaustive suites stay on Linux.

**Tech Stack:** Rust, maturin 1.15.0, Python 3.14/3.15, uv, GitHub Actions,
OIDC Trusted Publishing, pypi-attestations, pinned Lean.

**Spec:** [Approved design](design.md).

## Global constraints

- Python requirement is `>=3.14,<3.16`; no runtime Python dependencies.
- Wheels: Windows x86_64, manylinux2014 x86_64, macOS 11 arm64, no sdist.
- PRs have no write/OIDC publishing permissions. Main releases require the exact
  main commit's CI, Documentation and Dependency audit success.
- Preserve original built bytes and source signatures on recovery. Never hide
  partial/mismatching publication with skip-existing.
- Five self-review passes per stage, real red/green behavior tests, one fresh
  final review. GitHub operations use gh. No merge or premature upload.

## Review focus

- Concurrent version reservations and lost responses cannot reuse a version for
  different commits (Task 2 local bare-remote tests).
- One Python installation cannot mask an unusable wheel on the other supported
  interpreter (Task 1 isolated uv invocations for both versions).
- Wrong or partial wheel inventories cannot be uploaded (Task 2 validation tests).
- Recovery distinguishes original build identity from current publish identity
  and rejects tampering (Task 3 negative provenance and index tests).
- Pending or successful PR CI cannot substitute for the released main source's
  successful checks (Task 3 check selection tests).

### Task 1: Binary packaging and installed-wheel integration

**Files:** `pyproject.toml`, `uv.lock`, `tools/wheel_smoke.py`,
`tests/test_distribution.py`, user installation guidance.

**Interfaces:** Maturin bin wheel containing `taloon`; `python -m tools.wheel_smoke
WHEEL --python VERSION` verifies actual uv installation and exit/report/cache behavior.

- [x] Write metadata/support-range assertions and smoke scenarios for version,
  help, local MIT allow/deny and missing archive, cold/warm cache.
- [x] Observe missing packaging/smoke behavior fail.
- [x] Configure maturin binary bindings with dynamic Cargo version; implement
  standalone stdlib smoke runner without invoking Python packages being inspected.
- [x] Build a local release wheel; run smoke on Python 3.14 and 3.15; run Python
  tooling tests and docs checks. Expected: both invoke the same wheel successfully.
- [x] Review five times, record evidence, and commit this independently usable stage.

### Task 2: Version reservations and artifact ownership

**Files:** `tools/release.py`, `tools/release_policy.py`, `tools/pypi_release.py`,
`tests/test_release.py`, `tests/test_pypi_release.py`.

**Interfaces:** `reserve_tag(root: Path, commit: str) -> str | None`;
`set_version(root: Path, version: str) -> None`; CLI tag/set-version/assemble;
`validate_wheel(path: Path, version: str) -> None` and
`prepare_wheels(tag, release_json, assets, output)` consume Task 1's wheel format.

- [x] Write failing local-remote reservation tests for first/repeated/new/stale
  commits, competing reservations and lost responses; malformed versions fail.
- [x] Implement atomic reservation, consistent manifest/lock rewriting, and
  exact three-wheel inventory/checksum validation adapted from attributed hoimin.
- [x] Write and pass failure tests for changed licenses/metadata/tags, duplicates,
  missing/extra files, substituted bytes, and existing output directories.
- [x] Add independent generated version/order cases and a small Lean reservation
  model for no overwrite/stale rejection; document its correspondence limits.
- [x] Run stage tests, review five times, record evidence, and commit.

### Task 3: Provenance, release workflow and operations

**Files:** `tools/pypi_provenance.py`, `tools/release_checks.py`, tests for both;
`.github/workflows/release.yml`, development release procedure, OKF distribution
decision/baseline, AGENTS, user guidance, stage review record.

**Interfaces:** Attributed hoimin stage/inspect/verify provenance CLI consumes
Task 2's release assets; check gate consumes gh Actions JSON for exact commit and
push event; workflow consumes validated artifact/version outputs only.

- [ ] Write failing gate selection and provenance rejection/propagation tests.
- [ ] Adapt original signature conversion, exact downloaded byte/publisher/
  predicate verification and bounded propagation handling; keep index policy
  separate from version/inventory ownership.
- [ ] Build preview wheels in three native/manylinux jobs, smoke both supported
  Python versions, validate inventory; main-only gate/sign/release/store followed
  by TestPyPI inspect/upload/verify and PyPI inspect/upload/verify.
- [ ] Add manual main recovery of original signed release assets; no rebuild or
  replacing existing assets. Upload jobs check out no code.
- [ ] Document exact PyPI pending publisher fields and GitHub environments;
  validate syntax, workflow security, docs, all relevant tests, and hosted PR
  previews. Expected: PR cannot publish; all three wheel smoke jobs pass.
- [ ] Review five times, request a fresh final review, fix reproduced important
  findings, commit/push/create PR. Report external PyPI setup still required.
