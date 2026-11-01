# Terminal text staged review

Base `efb0cff5b47bab14d494a17dc3a87118e514d21a`, macOS arm64, 2026-11-01.

## Design: five passes

1. Traced all safe() callers in check and describe. They consume Display values
   immediately; no owned String API or report representation needs changing.
2. Counted structural allocation sites: current flat_map creates a Vec for every
   scalar and then a String. Cow can borrow ordinary report fields unchanged.
3. Checked the exact existing escape predicate and independent numeric oracle.
   Preserve Unicode, quotes/backslashes, C0/C1, short escapes and bidi boundaries.
4. Compared one always-owned String with Cow's existing Borrowed/Owned Enum.
   Selected borrowed fast path; escaped text still requires allocation and an
   additional prefix scan. No bespoke type or public method is needed.
5. Considered JSON streaming and typed worker keys. Their parser compatibility
   and scheduling/order costs require separate evidence; selected the smaller
   rendering improvement under the owner's internal-improvement scope.

## Implementation: five passes

1. Checked the complete product diff and every immediate Display caller. Cow's
   borrowed lifetime remains within formatting expressions, including temporary
   joined strings; no report value or public API gains a borrowed lifetime.
2. Compared predicates exactly. The existing control/bidi policy is centralized,
   not expanded. Normal quotes/backslashes and ordinary Unicode remain unchanged.
3. Baseline semantic examples/property passed (16 cases). Adding six Borrowed
   requirements produced E0308 against the old String API; then all 22 cases
   passed. This RED checks a private ownership guarantee, not an old output bug.
4. Reviewed generated expectations: explicit numeric scalar ranges and short
   escapes never call production's predicate or escape_default. Fixed cases pin
   NUL, ESC, C1 and both bidi-range endpoints/adjacent nonmembers. New tests have
   AAA structure and one scenario/property, with independently named variants.
5. Inspected measured paths: no per-character Vec construction remains, unchanged
   inputs return Borrowed, escaped inputs allocate a growable output String.
   Kept that extra scan tradeoff explicit. Strict Clippy requested assert_ne!
   instead of an empty-string negation in the benchmark; corrected only the
   assertion style, then Clippy passed. No product defect was involved.

## Final verification: five passes

1. Stable suite passed 261 regular Rust tests, including check/describe process
   rendering and all existing semantic/formal adapters. Focused terminal cases
   passed 22; the borrowing type failure and GREEN are separately recorded.
2. Generated-seed and seed-12345 nightly suites each passed 261 regular tests.
   Proptest executes 256 numeric Unicode oracle cases. No timer threshold enters
   ordinary tests, and helper speed is not attributed to the complete command.
3. Existing Lean build and all three corpus freshness checks passed under the
   retained 20-second deadline. Metadata/inspection fuzz each completed 1,000 runs
   at max_len 4096. They cover existing core semantics, not terminal rendering;
   no new theorem or fuzz claim about Cow or escaping is asserted.
4. Python 134 tests, Ruff format/ALL lint, strict ty, Clippy/rustdoc and docs/OKF/
   strict user-site checks passed. No Python implementation changed, so Python
   mutation testing is inapplicable. No user workflow or compatibility contract
   changed; user documentation needs no update.
5. Inspect final diff/privacy/source hashes and run local hooks before commit;
   observe CI for the actual pushed SHA in the PR. A separate whole-PR reviewer
   checks code/design; final document/hash/check reconciliation stays explicit
   executor evidence, not a claim that an earlier reviewer saw future results.

Scoped design diagnostics: 8/8 (10/10), no failed row identified. Responsibilities,
interfaces, replaceability, intent comments, design review, hidden decisions,
understandable boundaries and deliberate design investment remain coherent.
The exact percentage of time and exhaustive correctness are not measured.
The [design](design.md) and [measurements](performance.md) record alternatives,
remaining costs and finite verification limits.

Fresh reviewer `terminal_text_final_review` (separate context, 2026-11-01)
reviewed base `9a8e2a1` through `efb0cff` plus the product/test changes. No
Critical/Important/Minor finding remains. All eight scoped diagnostics passed;
it independently ran a whitespace diff check. It did not inspect private data,
expand escape policy, adjudicate deferred JSON/worker-key work, rerun executor
suites or observe CI. The new performance record/final OKF attribution was not
yet available at its inspection. Executor reconciliation of complete logs and
source hashes supplies that later evidence; no reviewer approval is attributed
to unseen timing claims. Whole-command speed and peak RSS remain unestablished.
