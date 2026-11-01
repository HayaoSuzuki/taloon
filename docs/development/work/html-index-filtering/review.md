# HTML index filtering staged review

Base `56fff43cdec130da66510a4f1bb5f135efbf9544`, macOS arm64, 2026-11-01.

## Design: five passes

1. Traced Http's bounded response into PackageIndex and HTML tokenization. The
   retained tag vector duplicates historical data even though output is scoped.
2. Checked base encounter order. A missing href does not select a base; the first
   href joins the response URL; earlier anchors cannot be rebound retroactively.
3. Checked failure semantics. A late unsupported version must invalidate a match;
   duplicate target links remain ambiguous and failures are absorbing.
4. Checked html5ever raw-text/RCDATA switches and local attribute-name collection.
   Preserve both, including modern-attribute presence overriding legacy values.
5. Compared direct private sink state with a generic event API and JSON streaming.
   Selected local interpretation; no caller protocol or new domain Enum is needed.

## Execution rulings

This is preservation refactoring of an already-green implementation. New fixed
and generated characterization tests are run against the old parser first; their
passing result is not a behavioral RED/defect claim. Initial missing test imports
are test setup errors, not production failures. The same ignored release harness
measures both implementations without a hard elapsed-time threshold.

## Implementation: five passes

1. Mapped every former post-tokenization branch to HtmlIndex::record_tag. Base,
   version, URL scope, fragment hash, metadata priority and Binding::Ambiguous
   semantics remain in one private protocol owner.
2. Inspected terminal-state transitions. Result discards partial output on error;
   no later token can restore success. Raw-text switching still occurs even
   after failure, and the tokenizer consumes the document through EOF.
3. Replaced base_seen plus cloned fallback URL with Option<explicit base> and a
   borrowed response URL. Both first-base and retroactive-base regressions pass.
4. Initial streaming still copied attributes into per-tag maps. Measurement was
   neutral, so replaced this with reverse borrowed local-name lookup. The prior
   map kept the last local-name value; html5ever's own duplicate handling remains
   intact. Duplicate-modern, uppercase and legacy-precedence cases passed before
   the borrowing change and afterward. Initial colon/equal test-fixture mistakes
   were corrected to the existing HTML hash syntax; not production bug claims.
5. Checked scope/storage: no historical tag vector, tag-name String or duplicate
   attribute map remains. Tokenizer/body and one current tag remain allocated.
   Five-sample timings improve modestly, not evidence for whole-command speed,
   allocator counts or peak RSS. No I/O, dependency or public API was added.

## Final verification: five passes

1. Stable Rust suite passed 239 regular tests. Focused index tests passed 26
   regular cases, including the unchanged strict 216-case Lean index adapter
   through JSON and HTML (432 lookups) and the new base-order property.
2. Generated-seed and seed-12345 nightly suites each passed 239 regular tests.
   Proptest checks 256 generated base/order/history cases; its expected absolute
   target comes from encounter order, not the parser's URL-joining helper.
3. Existing Lean build and all three corpus freshness checks passed within the
   unchanged 20-second command deadline. No theorem about HTML tokenization,
   arbitrary parsing or memory allocation is asserted. Metadata/inspection fuzz
   each completed 1,000 runs at max_len 4096; they check existing core paths,
   not this binary-private HTML sink. Existing index malformed-input properties
   separately exercise HTML/JSON in the regular suite.
4. Python 134 tests, Ruff format/ALL lint, strict ty, Clippy/rustdoc and docs/OKF/
   strict site checks passed. No Python implementation changed; Python mutation
   testing is inapplicable. User commands, cache schema and evidence rules remain
   unchanged, so user guidance needs no update.
5. Final diff, source hashes, privacy, local hooks and fresh whole-PR findings
   are checked before commit. GitHub checks are observed for the pushed SHA and
   recorded in the PR; local results do not imply remote outcomes.

Scoped software-design diagnostics remain 8/8 (10/10): coherent responsibilities,
deep interface, replaceable internals, intent comments, design review, hidden
protocol decisions, understandable boundaries and deliberate design investment.
No failed row is identified in the inspected scope. Exact time percentages and
universal correctness/performance are not established by this checklist.

The [design](design.md) explains why no generic API or additional domain Enum is
introduced. The [measurement record](performance.md) keeps samples and limits
separate from the structural storage argument.

Fresh whole-PR reviewer `html_index_final_review` (separate context, 2026-11-01)
inspected the branch from `9a8e2a1` through `56fff43` plus this working tree.
It found no Critical, Important or Minor issue. All eight scoped diagnostics
passed and no finding is deferred. It independently ran the whole-branch
diff whitespace check. Local html5ever 0.35.0 source
confirms tokenizer-level duplicate suppression before the sink; namespaces are
not adjusted without a tree builder. General browser/tree-builder semantics
(plaintext/noscript/foreign content) remain unchanged and are outside this
preservation claim. The reviewer did not independently rerun executor checks,
observe final CI, or establish peak RSS. The original model's finite/unbounded
limits and synthetic timing caveats remain explicit.

Independent correctness of URL normalization, SPDX parsing, cryptography and
html5ever generally was not established by this review; their relevant usage
and tokenizer behavior were inspected. Existing model and execution evidence
retain their narrower stated claims.
