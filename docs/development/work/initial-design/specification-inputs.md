# Initial design specification inputs

This record captures decisions from the repository owner's design conversation
on 2026-10-30. The owner selected the recommendations below, with the recorded
changes. Approval of these inputs does not constitute approval of the resulting
written design or evidence of implementation. The conversation used the
`grilling` and `software-design-philosophy` skills.

The original [Issue 1](https://github.com/HayaoSuzuki/taloon/issues/1), inspected
on 2026-10-30, proposed a fast license checker for uv-resolved Python
dependencies. Its design candidates were open. The issue had no comments at
inspection time. This conversation resolved candidates for taloon's initial
implementation.

## Owner decisions

| Question | Recorded decision |
| --- | --- |
| Q1 | Implement in Rust; inspect Python dependencies in `uv.lock`. |
| Q2 | Follow cargo-deny's license-checking use case; consider no configuration, output, or other compatibility with cargo-deny. Use specifications suitable for Python metadata and uv.lock. Poetry support stays outside this version. |
| Q3 | Separate a pure processing core from file, network, clock, environment, and presentation operations. Decide whether an effect state machine is needed after examining acquisition needs. |
| Q4 | Success requires checking the entire selected scope and confirming policy compliance. Distinguish a violation from inability to check. |
| Q5 | Check all wheels and sdists recorded for every external dependency in the lock, across environments; do not discover additional published artifacts. |
| Q6 | Check declared license information. Do not promise discovery of undeclared licenses in bundled components. |
| Q7 | Do not install, build, or execute packages or build backends; do not resolve build dependencies absent from the lock. Do not claim to check future built wheels. |
| Q8 | Parse uv.lock without invoking uv at runtime. Isolate format knowledge; reject unsupported formats. Study uv code without depending on its internal crates in the initial version. |
| Q9 | Exclude the project itself and workspace members themselves; check their external dependencies. Do not exclude packages just because they are private. |
| Q10 | Support registry, direct URL, and local-file wheel/sdist artifacts. Report external Git and directory dependencies as unsupported, incomplete checks. |
| Q11 | Prefer License-Expression. If absent, accept legacy License only when it parses as a valid SPDX expression. Do not infer expressions from classifiers, prose, fuzzy names, or license text. An invalid License-Expression is an error rather than a reason to fall back. |
| Q12 | The owner changed the recommended dedicated taloon.toml to pyproject.toml. Use tool.taloon; missing policy is an error. An explicit empty allowlist permits nothing. |
| Q13 | Evaluate AND, OR, and WITH as specified in the conversation. Treat WITH combinations and distinct GNU identifiers as distinct permission units; do not infer license compatibility. |
| Q14 | Additive exceptions require normalized package name, exact version, source, and reason. Do not bypass missing evidence or unsupported sources. |
| Q15 | Manual clarification specifies an artifact hash, an SPDX expression, and reason. Retain original evidence and evaluate the clarified expression under the same policy. Require archive-hash verification for clarification. |
| Q16 | Permit custom LicenseRef identifiers only in scoped exceptions requiring package name, exact version, source, and artifact hash. Forbid them in the global allowlist. |
| Q17 | Require reasons for exceptions and clarifications; expiry is optional and inclusive by UTC date. Expired entries are errors. Warn about unused entries and global permissions only when non-use can be established. Pass the evaluation date into the core. |
| Q18 | Cache acquired metadata and provenance under artifact content identity. Re-evaluate policy on every run. Do not depend on uv's internal cache format. |
| Q19 | Default execution fetches missing evidence. Offline execution makes no network requests and requires enough cached or local evidence for the full scope. |
| Q20 | Prefer standard per-artifact metadata when the index archive hash agrees with the lock and a metadata hash can be verified. Otherwise acquire the archive and verify its lock hash. Trust the index's metadata-to-archive association without claiming archive-hash verification. Manual clarification requires archive verification. |
| Q21 | Treat uv.lock as authoritative. Leave freshness checks against dependency declarations and re-resolution to uv. |
| Q22 | Use taloon check, upward discovery, and --project. Apply one policy beside one lock across the workspace. Do not merge member or user-wide policies. |
| Q23 | Provide text and JSON. Exit 0 for complete compliance, 1 for a complete check with violations, and 2 for incomplete/invalid checks. Incompleteness takes precedence. Aggregate recoverable failures. Include provenance and shortest dependency paths by entry point. Add --deny-warnings. |
| Q24 | Initially support credential-free HTTPS and HTTP authentication through .netrc or NETRC. Defer uv stores, keyrings, and cloud authentication. Redact credentials and do not forward them across host changes. |
| Q25 | Use a two-second target for a fixed 1,000-dependency, 10,000-artifact warm-cache workload. The owner questioned needing a threshold, then accepted two seconds. The subsequent recap made this a measured performance goal rather than a wall-clock CI failure condition. No performance result exists yet. |
| Q26 | Select approach A: the core produces artifact/evidence requests, acquisition returns results, and the core aggregates and evaluates. Do not build an acquisition effect state machine initially. |
| Q27 | Provide a local pre-commit hook invoking an already-built taloon check, without filenames, on each commit. Use the same command and policy in CI. Defer binary publishing and release automation. |
| Q28 | Delegate crate selection, internal types/files, concurrency/timeouts/retries, cache format/concurrency/recovery, tested lock format range, ordering, JSON shape, and configuration key names to the implementer. Record reasons and verification. Consult the owner before reducing scope, weakening success, or adding runtime prerequisites. |

## Source observations

- At repository revision `d5985ba2b65571147fbbeb88736e6420c852f86a`,
  `src/lib.rs` contains crate documentation only. There is no product behavior
  or Rust behavioral test suite.
- [uv lockfiles](https://docs.astral.sh/uv/concepts/projects/layout/#the-lockfile)
  cover multiple environments. uv recommends
  [workspace metadata](https://docs.astral.sh/uv/reference/internals/metadata/)
  over direct lock parsing; the metadata JSON schema is also still preview.
- uv identifies explicit workspace members from its lock manifest and an
  implicit root from an editable/virtual `.` source. A directory source alone
  is not a root. These observations concern uv revision
  `d041e5700a7bb39bff687ebbba4f6f6f2bda5cc8`, not every historical lock:
  [workspace indexing](https://github.com/astral-sh/uv/blob/d041e5700a7bb39bff687ebbba4f6f6f2bda5cc8/crates/uv-lock/src/lock/mod.rs#L2724),
  [root definition](https://github.com/astral-sh/uv/blob/d041e5700a7bb39bff687ebbba4f6f6f2bda5cc8/crates/uv-lock/src/lock/mod.rs#L8120).
- [Core Metadata](https://packaging.python.org/en/latest/specifications/core-metadata/#license-expression)
  scopes License-Expression to its containing distribution. An sdist's
  declaration does not establish the declaration of arbitrary future wheels.
- [PEP 658](https://peps.python.org/pep-0658/#specification) and
  [PEP 714](https://peps.python.org/pep-0714/) specify per-artifact metadata.
  A metadata digest authenticates metadata bytes, not the full archive digest.
- [uv HTTP authentication](https://docs.astral.sh/uv/concepts/authentication/http/)
  covers more credential mechanisms than this version will support.

## Status

The owner accepted all decisions above. The canonical draft design belongs in
the development knowledge bundle. Its approval, implementation, product tests,
and performance measurements remain separate steps.
