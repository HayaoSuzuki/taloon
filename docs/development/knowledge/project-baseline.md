---
type: Contract
title: Project baseline
description: Rust package, current policy implementation, and remaining pipeline boundaries.
status: draft
sources:
- id: cargo
  resource: ../../../Cargo.toml
  working_tree: modified
  sha256: 52d917784ec7fb6b45b41062aca887b08568a2bb28cef0f49cdcf2a4a293bbc2
- id: toolchain
  resource: ../../../rust-toolchain.toml
  working_tree: untracked
  sha256: f715f440167e20fd8ea9afa0498c3d1ab80174cfcb8cbc895f106a34d907a1e1
- id: license
  resource: ../../../LICENSE
  working_tree: untracked
  sha256: 48255018b41fc0e965b1115af7e6779bc218bb8a6747d561da800d5022622aa2
- id: scope
  resource: ../../../AGENTS.md
  working_tree: modified
  sha256: dfcbbe67f52b344e1c87ef6f14f3db2b2c7b6ca755b81a95c20d9bc2643d4bb5
- id: crate
  resource: ../../../src/lib.rs
  working_tree: modified
  sha256: 2148d4779ea1155e61bdd50b3f78c62d5c96be0ca8eeb35c9748ea9ae97cbf5c
- id: design-inputs
  resource: ../work/initial-design/specification-inputs.md
  working_tree: untracked
  sha256: 67d6a5e887ff58b121639ec7426339459ed84640cf7da8c3d02f5de3f13d2930
- id: design-approval
  resource: ../work/initial-design/implementation-approval.md
  working_tree: untracked
  sha256: 1593dc51b5663e513152059e50b39795063a696bf3b6b417c9a5c8fae959834c
- id: runtime
  resource: ../../../src/cli.rs
  working_tree: untracked
  sha256: 893b9331280b6db0987a9ae32b5203ea1e08635f8c85b19f340b5d088532f64c
---
# Current baseline

taloon uses Rust edition 2024 with Rust 1.99.1 pinned for development. Workspace
Clippy settings deny `all` and `pedantic` lints. The sole package has publishing
disabled. It now depends on Rust crates for validated Python identities, SPDX
expressions, TOML, and supporting data types.[^cargo][^toolchain]

The repository owner selected Elastic-2.0, matching hoimin. Cargo records that
license, and LICENSE contains the license text.[^cargo][^license]

# Implementation boundary

The pure library implements validated identities, Python SPDX policy, complete
uv.lock version-1/revision-5 targets and annotated origins, in-memory artifact
metadata evidence, and coverage-owning inspection/report aggregation.
The binary implements local/HTTPS acquisition, evidence caching, offline checks,
and `taloon check`. [User guidance](../../user/index.md) owns operational details
and tested prerequisites.[^crate][^runtime]

[Stage reviews](../work/initial-design/implementation-reviews.md) record the actual
tests, Lean proof/corpus correspondence, properties, fuzzing, and at least five
self-reviews per completed stage. CLI process checks and local installation have
passed on macOS arm64. Final hook/CI integration, process performance measurement,
and independent whole-change review belong to stage 8 and remain pending.
These results establish their recorded scope, not legal completeness of upstream
declarations or arbitrary Rust/I/O correspondence to Lean.[^runtime]

The [inspection specification](license-inspection.md) and
[pure-core design](sans-io-design.md) record the approved contract. Publishing,
release automation, and deployment remain deferred.[^scope][^design-inputs][^design-approval]

# Recheck conditions

Update this baseline when the compiler, license, package layout, dependency inspection design,
or publishing policy changes. Add independently retrievable decisions for
metadata acquisition and policy contracts as they become concrete. Do not infer
runtime guarantees from this scaffold or from a successful empty test suite.[^cargo][^crate]

[^cargo]: [Cargo metadata and lint policy](../../../Cargo.toml).
[^toolchain]: [Pinned Rust toolchain](../../../rust-toolchain.toml).
[^license]: [Elastic License 2.0](../../../LICENSE).
[^scope]: [Project scope](../../../AGENTS.md).
[^crate]: [Initial library](../../../src/lib.rs).
[^design-inputs]: [Owner decisions for the initial implementation](../work/initial-design/specification-inputs.md).
[^design-approval]: [Written-design approval and review requirement](../work/initial-design/implementation-approval.md).

[^runtime]: [Runtime setup and CLI](../../../src/cli.rs).
