---
type: Contract
title: Project baseline
description: Rust package, current policy implementation, and remaining pipeline boundaries.
status: draft
sources:
- id: cargo
  resource: ../../../Cargo.toml
  working_tree: modified
  sha256: c27127e1f85dd6f1ddf02ffbf6926cde044fe806b92fd30207efb941bc3b00a8
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
  working_tree: untracked
  sha256: afcfe774b5f3fe1abe4fd5cddc8286b73a596ef8cd78e692c4a96234a2bcabf7
- id: crate
  resource: ../../../src/lib.rs
  working_tree: modified
  sha256: 4d66e25ce7339502bbee9dad0b814191bc8b20e2003768fa465012da0a80fb16
- id: design-inputs
  resource: ../work/initial-design/specification-inputs.md
  working_tree: untracked
  sha256: 67d6a5e887ff58b121639ec7426339459ed84640cf7da8c3d02f5de3f13d2930
- id: design-approval
  resource: ../work/initial-design/implementation-approval.md
  working_tree: untracked
  sha256: 1593dc51b5663e513152059e50b39795063a696bf3b6b417c9a5c8fae959834c
---
# Current baseline

taloon uses Rust edition 2024 with Rust 1.99.1 pinned for development. Workspace
Clippy settings deny `all` and `pedantic` lints. The sole package has publishing
disabled. It now depends on Rust crates for validated Python identities, SPDX
expressions, TOML, and supporting data types.[^cargo][^toolchain]

The repository owner selected Elastic-2.0, matching hoimin. Cargo records that
license, and LICENSE contains the license text.[^cargo][^license]

# Implementation boundary

The library implements validated identities and pure policy evaluation.
[Stage 1 reviews](../work/initial-design/implementation-reviews.md) record the tests,
initial Lean proofs/correspondence, properties, and fuzzing evidence. A usable CLI,
complete lock parsing, artifact acquisition, and caching remain unimplemented.
Rust is the implementation language. The
[license inspection specification](license-inspection.md) and
[pure-core design](sans-io-design.md) now record the owner's approved design.
The owner requires at least five self-reviews per implementation stage.
The approved design does not establish complete
metadata acquisition, artifact coverage, or caching.
Publishing and deployment remain deferred.[^crate][^scope][^design-inputs][^design-approval]
The [design review record](../work/initial-design/design-review.md) distinguishes
document review and validation from product verification.

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
