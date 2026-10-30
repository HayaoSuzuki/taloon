# Initial design review

The reviewed draft captured the owner's accepted interview inputs before
written-design approval. The owner has since approved the design and added
[stage review requirements](implementation-approval.md). The snapshots below
refer to that earlier review; product implementation and performance measurements
have not started. Current design owners:

- [License inspection specification](../../knowledge/license-inspection.md).
- [Pure core and acquisition boundary](../../knowledge/sans-io-design.md).
- [Original specification inputs](specification-inputs.md).

## Builder review

On 2026-10-30 the Codex subagent `/root/design_builder_check` read the design as
a Rust implementer preparing a build plan. The review requested questions whose
answers would materially change the plan, excluded explicitly delegated choices,
and checked agreement with the interview inputs. Its result was
“nothing important left.” Follow-ups confirmed the explicit clarification of
invalid license expressions and the final snapshots below after moving evidence
to the content-specific `initial-design` directory at the owner's request.

| Source snapshot | Working-tree SHA-256 |
| --- | --- |
| `license-inspection.md` | `9803665d5addc0a2253ebbced9fe14bb7b0b5731966119bd31327908dac3256b` |
| `sans-io-design.md` | `6de0ccec556fe790a1a98c067ce037824ccd97981d8bbecb767d82d40d65e263` |
| `specification-inputs.md` | `67d6a5e887ff58b121639ec7426339459ed84640cf7da8c3d02f5de3f13d2930` |

These were uncommitted drafts on base revision
`d5985ba2b65571147fbbeb88736e6420c852f86a`. The review is an agent's planning
assessment, not owner approval or proof of product behavior.

## Implementer choices to inspect during approval

The owner delegated internal details in Q28. The draft makes these choices
explicit:

- Local archives without a recorded lock digest require re-hashing on each
  run. Reports distinguish observed local content from lock-pinned content.
- Hash-scoped custom LicenseRef permission requires archive-content verification,
  like manual clarification.
- Unused warnings use conservative syntactic/selector matching and are
  suppressed when evidence is incomplete.
- The proposed configuration schema, nearest-lock discovery behavior,
  versioned JSON report, duplicate-result rejection, and cache recovery rules
  remain inspectable in the current design owners.

## Design diagnostics

The pre-documentation design scored 6/8 diagnostics, or 7.5/10, under the
software-design-philosophy skill. Interface contracts and design-review criteria
were the two incomplete items. The final draft supplies both, giving the
written proposal an assessed 8/8, or 10/10. This is a design assessment; assess
the implementation separately when it exists.

The eight checks concern one-sentence responsibilities, deep interfaces,
replaceable implementations, abstraction comments, design review, information
hiding, understandable boundaries, and deliberate design improvement. No
diagnostic remains open at the written-design level. Implementation reviews must
look for actual leakage, shallow forwarding, and contracts callers cannot use.

The prose review scored directness 9, rhythm 8, reader trust 9, authenticity 8,
and density 8, totaling 42/50 under stop-slop. The draft preserves explicit
conditions because they determine inspection success and evidence strength.

## Documentation validation

Checks ran on macOS with uv 0.13.6, Python 3.14.9, and pytest 9.2.0:

| Command | Observed result |
| --- | --- |
| `uv run --frozen --no-sync python -m tools.check_docs` | Passed OKF structure, source-footnote association, navigation, and local links. |
| `uv run --frozen --no-sync pytest` | 28 documentation-tooling tests passed. |
| `uv run --frozen --no-sync mkdocs build --strict` | Exit 0; user-site build passed. Material printed its general MkDocs 2 migration notice. |
| `git diff --check` | Exit 0; no whitespace errors in tracked changes. |

These checks do not validate product semantics or the two-second performance
target. The user guide remains unchanged because product behavior is still
unimplemented. No Rust code, dependencies, or runtime configuration changed.
