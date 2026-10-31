# Distribution acceptance review

Reviewed on 2026-11-01 against `587ee309..8be75b9` by a fresh read-only reviewer
using gpt-6-astra, with the approved design, implementation plan and review focus.
The reviewer found no Critical, Important or Minor code findings, independently
ran all 95 Python tests successfully, and left the checkout unchanged.

The review checked atomic reservations/lost responses, isolated same-wheel uv
execution, exact inventory/signature gating, original-byte recovery with separate
publication identity, and successful exact-source main checks rather than PR
checks. Hosted integration evidence was the outstanding acceptance condition.

## Observed hosted results

All checks at `8be75b9` passed in [PR #4](https://github.com/HayaoSuzuki/taloon/pull/4):

- [Release preview](https://github.com/HayaoSuzuki/taloon/actions/runs/41214479176):
  Windows x86_64, Linux manylinux2014 x86_64 and macOS arm64 wheel builds passed.
  Each job exercised the same wheel with Python 3.14 and 3.15, including actual
  uv installation, version/help, exits 0/1/2, paths and cold/warm evidence cache.
  Exact three-wheel assembly passed. Reservation, signing and index upload jobs
  were skipped on the PR event, as required.
- [CI](https://github.com/HayaoSuzuki/taloon/actions/runs/41214479117): Rust quality,
  tests, real TLS fixture, rustdoc, Lean proofs/corpus, bounded fuzzing and workflow
  lint passed on Linux.
- [Dependency audit](https://github.com/HayaoSuzuki/taloon/actions/runs/41214479189)
  and [documentation](https://github.com/HayaoSuzuki/taloon/actions/runs/41214479196)
  passed, including the expanded Python tooling suite.

Local final checks passed 95 Python tests, 77 normal Rust tests, all repository
hooks, strict MkDocs and document/OKF checks. A 256-example Hypothesis property
and real public reference signature positive/negative tests supplement fixed
tests. The release Lean lemmas are explicitly a decision model, not a proof of
Git, signatures, Python or network behavior.

GitHub environments `pypi` and `testpypi` were created through gh and verified to
allow only branch `main`, without a required human reviewer. No secret was added.
PyPI/TestPyPI account-side publisher registration, main signing and production
uploads remain separate prerequisites/observations; no upload was performed.

## Recorded rulings and boundaries

- Continue inline in the existing feature checkout as requested for the session;
  its branch isolates Git history, while edits remain visible in the shared
  directory. Cost: the checkout is not an independently isolated filesystem.
- Use Cargo-owned dynamic Python version metadata instead of duplicated version
  fields. Cost: packaging requires Cargo metadata and release edits must keep
  Cargo.lock consistent. Both cases have direct tests.
- Additional platforms, interpreters and sdist support are excluded by the
  approved inventory. Cost: unsupported users need an appropriate source build;
  published wheels intentionally do not cover them.
- Reject partial or mismatching publication rather than automatically repairing
  it. Cost: such recovery requires operator investigation; conflicting bytes are
  never silently accepted.
- PyPI account setup and uploads are outside pre-merge acceptance. Cost: merging
  without registered publishers fails at upload; verified GitHub assets support
  recovery after account setup. This does not grant permission to merge the PR.

No code findings were deferred. The design review observed seven of eight
software-design-philosophy diagnostics (approximately 8.75/10): clear module
ownership, simple interfaces, hidden policy/format knowledge, documented intent,
design review and readable boundaries. The design-effort percentage was not
instrumented; measuring that fraction in future work would close the remaining
evidence gap. It is not a runtime correctness claim.
