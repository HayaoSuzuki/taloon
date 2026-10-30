---
okf_version: "0.2"
---
# Development knowledge

Choose the topic relevant to your change, then inspect its cited sources.
Concepts distinguish approved designs from implemented behavior. Metadata and
link checks do not establish implementation correctness or human review.

- [Documentation ownership](documentation-policy.md): Audience boundaries,
  canonical topics, and evidence without duplicated current explanations.
- [Project baseline](project-baseline.md): Rust and licensing choices, current
  implementation, and dependency-inspection limits.
- [License inspection specification](license-inspection.md): Approved uv.lock
  coverage, policy, acquisition evidence, and CLI behavior; design approved,
  implementation evidence in the baseline.
- [Pure core and acquisition boundary](sans-io-design.md): Selected approach A,
  module ownership, API invariants, and implementation acceptance checks.
- [Formal models and generated behavioral tests](verification-strategy.md): Lean
  correspondence, property tests, fuzzing, and typed representation boundaries.

Use the [documentation procedure](../documentation.md) to maintain this bundle
and the [development guide](../index.md) to run checks.
