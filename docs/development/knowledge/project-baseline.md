---
type: Contract
title: Project baseline
description: Implemented Rust checker, verified local evidence, and operational boundaries.
status: verified
verified_by: Codex implementation agent
verified_at: "2026-10-31"
sources:
- id: cargo
  resource: ../../../Cargo.toml
  revision: a4914e4e9967c7e5282e29f1b994a5b5962e1a2e
- id: toolchain
  resource: ../../../rust-toolchain.toml
  revision: a4914e4e9967c7e5282e29f1b994a5b5962e1a2e
- id: license
  resource: ../../../LICENSE
  revision: a4914e4e9967c7e5282e29f1b994a5b5962e1a2e
- id: scope
  resource: ../../../AGENTS.md
  revision: a4914e4e9967c7e5282e29f1b994a5b5962e1a2e
- id: crate
  resource: ../../../src/lib.rs
  revision: a4914e4e9967c7e5282e29f1b994a5b5962e1a2e
- id: design-inputs
  resource: ../work/initial-design/specification-inputs.md
  revision: a4914e4e9967c7e5282e29f1b994a5b5962e1a2e
- id: design-approval
  resource: ../work/initial-design/implementation-approval.md
  working_tree: modified
  sha256: 1db5df1cd88ff30c390fb2d2960c6406cec472a71b1f0e22bf8670badbc55ada
- id: runtime
  resource: ../../../src/cli.rs
  working_tree: modified
  sha256: fe07012d6c70e4c62189d30cdbb02d9bc7378be2bfae7d55ab3432cd1a7ee79b
- id: inspection
  resource: ../../../src/inspection/mod.rs
  working_tree: modified
  sha256: 974c6f80134a37a4a7c93c3ae4ba1c85d7aeaf2030cfd3ab3d45d9e1c7be04ec
- id: cache
  resource: ../../../src/acquisition/cache.rs
  working_tree: modified
  sha256: c749e57ae04b1ab23b4a21a87f896bae124eb33afa76f370525b0ba772bf6004
- id: reviews
  resource: ../work/initial-design/implementation-reviews.md
  working_tree: modified
  sha256: faafe4bcb0d755121e39fd148f766493d03d2acbc9091d8692feaadfee04420b
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
and tested prerequisites.[^crate][^runtime][^inspection][^cache]

[Stage reviews](../work/initial-design/implementation-reviews.md) record five
self-review passes for each of the eight completed stages, actual tests, Lean
proof/corpus correspondence, properties, fuzzing, and fixes. The fresh independent
[whole-change review](../work/initial-design/final-review.md) found two issues,
both corrected and regression-tested. The
[process performance record](../work/initial-design/performance.md) records the
locally met warm-cache goal and its fixture/environment limits.[^reviews]

Local verification used macOS arm64. Ubuntu CI is configured; no GitHub-hosted
execution is claimed before push. These results establish their recorded scope,
not legal completeness of upstream declarations or arbitrary Rust/I/O
correspondence to Lean.[^reviews]

The [inspection specification](license-inspection.md) and
[pure-core design](sans-io-design.md) record the approved contract. Publishing,
release automation, and deployment remain deferred.[^scope][^design-inputs][^design-approval]

# Recheck conditions

Update this baseline when the compiler, license, package layout, dependency inspection design,
or publishing policy changes. Add independently retrievable decisions for
metadata acquisition and policy contracts as they become concrete. Do not infer
runtime guarantees beyond the recorded checks and supported input ranges.[^cargo][^crate]

[^cargo]: [Cargo metadata and lint policy](../../../Cargo.toml).
[^toolchain]: [Pinned Rust toolchain](../../../rust-toolchain.toml).
[^license]: [Elastic License 2.0](../../../LICENSE).
[^scope]: [Project scope](../../../AGENTS.md).
[^crate]: [Initial library](../../../src/lib.rs).
[^design-inputs]: [Owner decisions for the initial implementation](../work/initial-design/specification-inputs.md).
[^design-approval]: [Written-design approval and review requirement](../work/initial-design/implementation-approval.md).

[^runtime]: [Runtime setup and CLI](../../../src/cli.rs).

[^inspection]: [Coverage-owning inspection](../../../src/inspection/mod.rs).
[^cache]: [Evidence persistence](../../../src/acquisition/cache.rs).
[^reviews]: [Actual stage verification and decisions](../work/initial-design/implementation-reviews.md).
