# Python test design review

Reviewed all eleven `tests/test_*.py` files against the development guide's
[test design requirements](../../index.md#test-design). The inspected base was
`f8e312706625fc3c29fd09e733877e08fb6c0152`; the results below describe the
uncommitted working tree on 2026-11-14, macOS arm64, Python 3.14.10.
Pre-existing changes in `test_knowledge_references.py` and `test_wheel_smoke.py`
were retained. No production files were changed by this Python review.

The review inspected scenarios and assertions, then used AST inspection to
locate unnamed parameter groups and conditional test bodies. AST inspection was
a navigation aid, not the acceptance criterion. Setup loops and callback control
flow were reviewed for their purpose instead of automatically splitting them.
The final files contain 151 test functions, 45 explicitly named pytest parameter
groups, and eight Hypothesis properties. The collected suite has 269 cases.

## Per-file findings and changes

| File | Findings and action | Generated domain or reason for fixed cases |
| --- | --- | --- |
| `test_check_docs.py` | Named malformed concept/source cases. Split missing source from unreachable navigation and successful CLI output from failed CLI output. Each CLI context now has an explicit expected status and stream contents. Existing multi-diagnostic cases remain single aggregation scenarios. | YAML failure categories and CLI outcomes are explicit boundary regressions. Generated document-graph semantics belong to the inventory property below. |
| `test_distribution.py` | Named each packaging metadata field. Replaced the in-test section-selection branch with explicit section paths and a typed lookup helper. Kept workflow/platform agreement as one contract. | The shipping metadata and platform matrix are finite declarations. Generating arbitrary declarations would not test the checked-in packaging promise. |
| `test_json_data.py` | Named invalid container/key shapes. Broadened generated valid containers from integer-only values to recursive JSON values, preserving identity as the independent invariant. | Nested arrays/objects, null, booleans, integers and strings; at most 32 recursive leaves and eight children per recursive container. No production parser or copied validator computes expected identity. |
| `test_knowledge_references.py` | Preserved existing independent record-category cases and graph property. Named CLI, non-filesystem target and fence cases. Replaced conditional CLI fixture content with explicit content/status parameters. Marked the missing-entry exception operation. | The existing five-node directed-graph domain includes disconnected nodes, self-links and cycles. Floyd-Warshall closure independently predicts navigation reachability; graph construction and closure loops are setup. |
| `test_pypi_provenance.py` | Named every fixed group. Moved predicate, inventory and untrusted-evidence construction into setup helpers. Split missing-index inspection from verification refusal, CLI inspection output from verification, and successful staging from rejected staging. A separate two-wheel scenario checks the complete bytes/proof inventory and asserts that both build and signature verifiers receive each real fixture filename. A second-wheel signature refusal regression checks that no output is staged after the first wheel passes. Replaced the mixed generated mutation/branch test with separate exact-byte acceptance and appended-byte rejection properties. | Arbitrary wheel bytes up to 256 bytes and nonempty appended bytes up to 64 bytes, 128 examples per property; independent `hashlib.sha256` computes the original proof. Fixed signature, publisher, subject and propagation cases remain readable regressions. Real offline signature checks remain alongside tests that stub cryptographic/service boundaries. |
| `test_pypi_release.py` | Named inventory, structure, tag and metadata groups. Moved mutation selection into fixture builders; metadata field/value pairs are explicit. Every rejection still checks that staging output is absent when applicable. | Fixed archive faults represent different format/trust requirements. Generated content integrity is covered at the provenance inventory boundary; malformed archive categories remain named examples. |
| `test_release.py` | Named collision, version, identity and manifest cases. Corrected a test name that claimed untested dynamic metadata preservation. Kept manifest/lock updates as one atomic version-update scenario. Isolated internal manifest representation handling in a helper. | Real Git reservations and competing checkouts remain scenario tests; the reusable version-selection algorithm is generalized in `test_release_properties.py`. Callback branches simulate push failure/recovery; archive-building loops construct one complete platform inventory. |
| `test_release_checks.py` | Named foreign source/event/status/path, counter and terminal outcome variants. Split retry selection from pending-retry authorization. Added a generated latest-run/attempt ordering property with a known winner per workflow. | Three workflows, counters from 1 through 10000, arbitrary winner success flags, and permutations of nine records. Each winner beats both a lower attempt at its run number and an older run with a much larger attempt. Expected authorization is the conjunction of the three independently generated winner flags. |
| `test_release_prepare.py` | Named invalid event identities, lookup failures and invalid tag cases. Existing preview, recovery and publication scenarios already have separate setup, invocation and observable output/refusal checks. | Event categories and externally visible CLI output are fixed protocol boundaries; arbitrary GitHub events are not an implemented input domain. |
| `test_release_properties.py` | Replaced the conditional freshness assertion with exact minimization over admissible candidates. Expected output is calculated before the production call. Kept fixed boundary examples in the release tests. | Numeric triples from 0 through 1000 plus explicit boundaries, histories up to 32 records, 256 examples per property. The oracle enumerates the floor and historical patch successors and minimizes candidates satisfying all freshness constraints; it does not call production parsing/selection or repeat its maximum-history selection. |
| `test_wheel_smoke.py` | Preserved existing binary adapter changes. Named failed-verification cases and supplied explicit expected exception arguments instead of branching on detail in the assertion. | Optimized execution, missing installer/wheel, CLI arguments and the real-binary smoke protocol are distinct concrete integration contexts. The binary adapter exercises the smoke protocol, not an actual uv wheel installation. |

## Deliberate retained structures

Fixture builders may branch on an explicit named fault to construct typed test
inputs. This keeps each collected case independent while separating setup from
the tested operation. Nested Git/service callbacks branch to emulate one failure
schedule, not to choose an assertion. Loops build archive contents, repository
files, graph expectations and generated workflow histories; they do not run
independent test cases inside a function.

Multiple assertions remain when they describe one result: atomic refusal without
output, original bytes plus their proofs, complete diagnostics, or one concurrent
reservation outcome. The smoke protocol is itself the production operation under
test; its multiple CLI transitions form that integration scenario. Network/service
stubs establish boundary behavior, not live GitHub/PyPI publication. No new
mutation score or exhaustive guarantee is claimed. No new hoimin plan/verify was
run; production code was unchanged and mutation verification is separate from
this test-structure audit. The existing historical mutation report remains
evidence only for its recorded revision and environment.

## Verification

All repository Python quality gates passed:

```sh
UV_CACHE_DIR=/private/tmp/taloon-python-audit-uv uv run --frozen --no-sync ruff format --check .
UV_CACHE_DIR=/private/tmp/taloon-python-audit-uv uv run --frozen --no-sync ruff check --no-fix .
UV_CACHE_DIR=/private/tmp/taloon-python-audit-uv uv run --frozen --no-sync ty check
UV_CACHE_DIR=/private/tmp/taloon-python-audit-uv uv run --frozen --no-sync pytest --cov --cov-report=term-missing
```

The cache override avoids writes to the sandbox-restricted default uv cache.
Final pytest seed: `3233292125`; **269 passed**, **2264/2264 maintained Python
statements covered (100%)**, including child Python processes. Coverage does not
establish assertion strength, all asynchronous schedules, or other platforms.
The pre-existing debug binary was used for the adapter smoke cases. No commit was
created. User behavior, distribution decisions and contracts were unchanged, so
no user-guide or OKF contract amendment was needed for these Python edits.
