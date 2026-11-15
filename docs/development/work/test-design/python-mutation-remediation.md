# Investigation and remediation of the 23 open Python mutants

Verified 2026-11-15 in the working tree based on `73bfae0`.
This follows the [initial selected mutation run](python-mutations.md).
All **23 previously open candidate IDs** were investigated individually and
reverified. The original seven provisionally equivalent survivors were outside
this follow-up selection and were not reverified.

## Outcome

- **21 killed by behavioral assertions** after test improvements.
- **2 survived and were classified as equivalent under current consumers**.
- No timeout, error, inconclusive, not-run, memory or process-limit outcomes.
- Fresh baseline and complete verification succeeded; terminal `complete=true`,
  exit 0; run ID `a76cfadf-87b1-4b0f-801d-cea1bbc4f9b3`.
- Normal suite: **301 passed**, with **2,391/2,391 Python statements covered**.
  Ruff format/check, strict ty and documentation validation passed.
- Only eight Python test files changed. No production changes, new exclusions,
  Rust mutations or Rust builds were performed.

The two surviving changes are not counted as kills or silently removed from
the denominator. The native selected-run kill fraction is 21/23 (91.30%).
All 23 triage items are resolved: 21 missing expectations were repaired, and
two do not violate the current observable contract. No 100% mutation score,
whole-project score, or score combined with the earlier different-ID run is
claimed.

## Test improvements

Named cases distinguish malformed footnote definitions and exact path/message
diagnostics; compiler targets and wheel tags have explicit independent
expectations; generated compatible platform tags use a grammar independent of
the policy regex. Additional cases verify source-commit preflight, correct
index URL versions, bounded exponential backoff and attempt limits, missing
proof diagnostics, ancillary checksum handling, malformed workflow identities,
GitHub JSON arguments, case-insensitive missing releases and each smoke stage's
cache accounting.

Tests follow one scenario/property per function and AAA. Shared setup constructs
one protocol schedule; loops used for fixture construction do not execute
independent examples. Retry failures are finite even if attempt accounting is
mutated. Cross-review found no important issues or artificial mutation-only
expectations.

## Equivalent survivors and reconsideration

**Footnote expansion ordering** (`knowledge_references.py:113`): inserting each
output line at the front instead of appending reverses accumulated output
order. Expansion membership is controlled by unchanged `pending` and
`used_notes`; output `lines` does not affect expansion. Current navigation
consumers collect targets into sets before traversal. Definitions are resolved
before expansion. There is no supported ordering guarantee for navigation
yield order. Reconsider if a caller begins relying on expansion/yield order.

**Missing digest lookup** (`pypi_provenance.py:99`): subscript raises KeyError for
a missing digest; `get` yields None and shape validation raises TypeError.
Both reject the malformed inventory before accepting it or altering wheel
bytes. Valid inputs behave identically and current callers do not distinguish
these exception types. The new regression deliberately verifies refusal and
unchanged bytes while accepting either exception. Reconsider if exact
missing-field exception classification becomes a documented API contract.
This is contract equivalence, not literal traceback equivalence.

## Same-ID verification and disk limits

Used the same authenticated official v0.14.63 arm64 wheel as the initial run,
via `uv tool run --python 3.14 --from <wheel> hoimin`. Its SHA-256 remains
`3749dc199a2e775ac963f63aa3a4ae6e794dafba2692e09642507df56814e6d5`.
The earlier temporary plan had been cleaned up, so a new complete focused plan
was generated: 1,689 candidates, with the exact original 23 IDs selected.
Source files were unchanged, and all IDs matched the regenerated plan.

The selector remained `--source tools`, one worker, 8 GiB maximum workspace,
10 GiB minimum free space, macOS best-effort memory control, 60-second mutant
and 120-second baseline timeouts. This smaller follow-up plan used max-mutants
23 and a 1,800-second total budget, rather than the initial run's 100/3,600;
that budget change was another reason to regenerate the plan. Fingerprints
included `pyproject.toml` and `uv.lock`. The test argv remained the repository
`.venv/bin/python -m pytest -q -x --no-cov --randomly-seed=20261114`.
The existing debug binary was copied only for Python smoke tests.

Raw plan/events/wheel and isolated TMPDIR were held in one dedicated temporary
directory with EXIT cleanup. Peak hoimin-owned bytes:
**75.20 MiB**;
minimum available space:
**123.12 GiB**.
No disk-limit stop occurred; native execution cleanup was clean, and the
surrounding temporary directory was verified absent after completion.

## Individual outcomes and rationale

| Candidate ID | Target | Final outcome | Investigation and expectation |
| --- | --- | --- | --- |
| `m1_e21e9443ecb285ce28be61ccde09c1c7f3c297927ff2b926347dcefe457dcfd2` | `tools/check_docs.py:35` | killed | `tools/check_docs.py:35`, footnote-definition prefix `^\[\^` becomes empty Real behavioral gap; Verified in the same-ID run. A citation must have a matching Markdown footnote definition beginning at the line start. Added independently named missing-opening-marker, ordinary-reference, and inline-definition cases that retain the claim citation and require the exact missing-footnote diagnostic. The mutant's unanchored `evidence\]:` search would incorrectly accept these malformed definitions. |
| `m1_c773ebe2753ba80e194f2d0ce0d457a352822e11a36fc4e3c49d65e144dfe191` | `tools/check_docs.py:83` | killed | `tools/check_docs.py:83`, missing-entry diagnostic list becomes tuple Real public return-container contract gap; Verified in the same-ID run. `validate_okf` and `check_repository` declare `list[str]` and return mutable diagnostic lists on all paths. Strengthened the existing missing-entry test to compare against the exact one-element list. This enforces the public list result and useful diagnostic together, without introspecting private implementation details. |
| `m1_69835c01e2d6b781d8cf66c59c3c4212aa367d97b738043386053da126f2f495` | `tools/check_docs.py:89` | killed | `tools/check_docs.py:89`, `: ` diagnostic separator becomes empty Real observable diagnostic-format gap; Verified in the same-ID run. Added named missing-frontmatter and invalid-type cases comparing complete `path: message` diagnostics. The source path and explanation must remain separable in the checker/CLI's emitted diagnostics. The malformed-footnote cases also compare the complete path/message string. |
| `m1_fa482b596c3d3240b03c7eead63e416e32994683ada3e7600e4a95ee051dcd6a` | `tools/knowledge_references.py:113` | survived; contract-equivalent | `tools/knowledge_references.py:113`, `lines.append(line)` becomes `lines.insert(0, line)` Observationally equivalent for the current supported inventory/checker consumers. The mutation only reverses the accumulated output line order: `pending` and `used_notes`, which determine note expansion, are unchanged and never inspect `lines`. Consequently the same footnote/ordinary-line multiset is expanded, including cycles and repeated references. `markdown_targets(..., navigation=True)` scans this list, but `inventory` inserts its targets into `graph[page]`, a set. Reference definitions are fully resolved before expansion; duplicate-label suppression can change yield order, not the target set. Other callers use non-navigation scanning and do not call this function. Neither the module documentation nor documentation procedure promises navigation iterator order. No artificial order assertion was added. This classification is scoped to supported inventory/navigation membership behavior: direct helper-return or navigation-yield order is mechanically different and would require reconsideration if made an API guarantee. The existing independently generated transitive-closure property remains the relevant observable membership oracle. |
| `m1_547e56fd0add41822e29e189fcdb0972d9671f4586b1703990cfff559a2d8cd8` | `tools/pypi_provenance.py:99` | survived; contract-equivalent | Missing digests via subscript raises KeyError; get returns None and object_mapping raises TypeError. Both immediately refuse the malformed inventory; all valid inputs agree. Current callers do not catch either exception differently, and no documented exact exception-class/traceback contract exists for missing digest fields. Added test_index_inventory_refuses_missing_digest_record verifies refusal and unchanged wheel bytes while accepting either boundary failure. Pinning incidental KeyError solely to kill would overconstrain the contract. Contract-equivalent under current callers; not literally exception-equivalent. Survival confirmed. Reconsider if a caller depends on missing-field exception classification. |
| `m1_e6b0bb4bc14281c065ee2896fd28fa4ab27817cecf5fb8720005aa815e15b776` | `tools/pypi_provenance.py:263` | killed | Version in per-wheel index provenance URL must omit the release-tag v prefix. Added named PyPI/TestPyPI cases checking every independently constructed expected endpoint passed to verification, with valid complete inventory. Behavioral gap addressed; assertion kill confirmed, Verified in the same-ID run. |
| `m1_c87bc7af0287ed1f51605cdaf0bd9561a6140d2acb419cb76cd8c218a6a80ae5` | `tools/pypi_provenance.py:309` | killed | A service response omitting attestation_bundles should report the explicit missing-proof diagnostic, just like other invalid bundle shapes, instead of leaking an incidental missing-key lookup. Added missing-key response test requiring ValueError with Expected exactly one attestation bundle. Valid downloaded bytes isolate the proof boundary. Behavioral diagnostic gap addressed; kill confirmed, Verified in the same-ID run. |
| `m1_30d8d06dcbb6e4269dcc28bc55d196ddd09281fd6f7c080a7b89b66af7cff028` | `tools/pypi_provenance.py:349` | killed | Resetting attempts permits work after the 61-attempt ceiling. Added frozen-clock test supplying 61 missing-publication errors followed by success; original must raise the same HTTPError at attempt 61, with 60 sleeps. Later success bounds the test even under the broken counter, avoiding mutant hangs/timeouts. Retry-limit gap addressed; kill confirmed, Verified in the same-ID run. |
| `m1_8ca5764f2a1a0fd2dd02698b8bdc3f5cdd45c1da3edbc70ff49f814cc8958c24` | `tools/pypi_provenance.py:364` | killed | Changed exponent halves initial delay and shifts the backoff schedule. Named initial/growth/cap cases require exact independent delay sequences [10], [10,20], [10,20,40,60]. Clock and sleep are stubbed; no real wait. Backoff gap addressed; kill confirmed, Verified in the same-ID run. |
| `m1_94ee4f4691055575188ce699466f4a2964bbe0ad31824e38b26be39af0742a28` | `tools/pypi_release.py:135` | killed | Empty suffix includes checksummed nonwheel release assets in the wheel set and rejects valid releases. Named source-archive/attestation cases add legitimate ancillary assets and checksums; staging must preserve exactly the original wheels and bytes. Valid-input domain gap addressed; kill confirmed, Verified in the same-ID run. |
| `m1_44f89f994ead3595a2fdd32c971bfff68edd12aa23c2acc8a761ebdaa489c2d5` | `tools/release.py:88` | killed | Missing local commit must fail before remote reservation discovery/publication. Real Git fixture uses a syntactically valid absent object; test requires CalledProcessError before a stubbed remote-state lookup, then independently checks remote refs remain empty. Deletion instead reaches lookup and later push retries. Preflight/side-effect-order gap addressed; kill confirmed, Verified in the same-ID run. |
| `m1_e6c8591f483fe79f194826a11d78520607c9150ab666f6a30aa3bf099743b3da` | `tools/release_checks.py:38` | killed | `tools/release_checks.py:38`, `run.get("head_sha")` becomes `run["head_sha"]` Real malformed-service-record boundary gap; Verified in the same-ID run. A successful-looking run that cannot prove its workflow/source/trigger identity must not authorize publication. Added named missing-path, missing-head-sha, and missing-event cases through public `checks_passed`; all require false rather than an uncaught lookup exception. The missing-head-sha case exercises the exact survivor. |
| `m1_99b991459dd43f23ed9ca181e6d9ce29cf8335fce6d6f7fe8c79100e02f806a1` | `tools/release_policy.py:24` | killed | Erases Windows Rust target. Compiler invocation must select `x86_64-pc-windows-msvc`, independently of internal policy consistency. Named Windows compiler/installer identity case uses literal external identifiers. |
| `m1_724841045c971bf6aa21660bca93ed40fea180f9be2106087fbb8dafdd1500d9` | `tools/release_policy.py:24` | killed | Erases Windows build-wheel regex. A supported `win_amd64` wheel tag would no longer match. Named supported Windows build-tag fullmatch must succeed. |
| `m1_c3d4373d9dca3c269044890a9736541d260e9092d50a47702c2d6181c6423234` | `tools/release_policy.py:24` | killed | Erases Windows published-wheel tag. Published asset validation must require the actual installer tag. Named Windows compiler/installer identity case pins `win_amd64`. |
| `m1_b6e5069b2a24cdb39d9d349108ec88a05d832c3bc2a8097e2241ce911d3b54d0` | `tools/release_policy.py:29` | killed | Erases manylinux build-wheel regex. Compatible x86_64 manylinux builds would be rejected. Named exact stable tag and generated `manylinux_2_<version>_x86_64` cases use an independent tag grammar. |
| `m1_86017abacea662706cc858adc3a21351f9a7c0ad043390258e5e5672afad10e0` | `tools/release_policy.py:30` | killed | Erases published manylinux wheel tag. Exact published platform identity is a distribution contract. Named Linux identity case pins `manylinux_2_17_x86_64.manylinux2014_x86_64`. |
| `m1_8ff2a4c8145e30b7075e5c1fa1c35bb2ecc3636b2af3bef265c636bba2086c2c` | `tools/release_policy.py:34` | killed | Erases macOS compiler target. Arm64 release compilation must select `aarch64-apple-darwin`. Named macOS compiler/installer identity case. |
| `m1_875167eb29eb4435c2d74ffce58ca5e0be265943fd919a9d5a6a5c2ac736c368` | `tools/release_policy.py:35` | killed | Erases macOS build-wheel regex. Compatible arm64 macOS build tags must be recognized. Named stable macOS tag plus generated `macosx_<version>_0_arm64` grammar. |
| `m1_7f2f77287e3e9c4d6d7d6f4afe4bfda5781f5556543e70411f5f3ad7089d87ee` | `tools/release_policy.py:36` | killed | Erases published macOS wheel tag. Exact PyPI staging identity must retain `macosx_11_0_arm64`. Named macOS compiler/installer identity case. |
| `m1_b99e411602e190b4340ac2ec2c789e15cfb0690feaeef6772e44ed096c23c610` | `tools/release_prepare.py:34` | killed | Erases `--json` from gh release view. Without the JSON option, the CLI does not return the schema consumed by the release identity check. Existing recovery test now asserts the entire subprocess argv including repository, `--json`, and field list. |
| `m1_9401c390b38c8f6d5141662ce1bef65df590b6453e33ead315ea10d19cfe9c79` | `tools/release_prepare.py:50` | killed | Removes stderr case folding. A missing release with uppercase or titlecase wording incorrectly blocks a reserved new publication. Existing publication-selection test now has independently named lowercase/titlecase/uppercase diagnostics, requiring identical full publication output. |
| `m1_6c5b05671ccce2a1e13ee744d4fad4974d8c2080abdf2ea9680104d7d50aab32` | `tools/wheel_smoke.py:98` | killed | Deletes smoke cache-hit verification. Incorrect cold/warm/denied/missing-file cache counters would all be accepted. Four independently named process-protocol cases change only one stage's hit counter and require AssertionError carrying the exact offending report. Every other report field satisfies that stage, so rejection is attributable to cache accounting. |

Production behavior and design guarantees remain unchanged; no user guide or
OKF contract amendment is required. These results concern the selected Python
mutants and local environment, not unverified candidates or hosted CI.

