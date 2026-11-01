# Borrow unchanged terminal text

Base `efb0cff5b47bab14d494a17dc3a87118e514d21a`, 2026-11-01. Follow-up to the
owner's performance/type/structure audit, extending still-open PR #10.

## Decision

CLI safe() currently allocates a Vec for every character, then collects a new
String. Both check and describe use it repeatedly for displayed report fields.
Return private Cow<str> instead: borrow when no escaping is needed, otherwise
append escaped characters into one output String. A shared private predicate
owns the unchanged control/bidi policy. Cow's Borrowed/Owned states express the
ownership distinction; a new application Enum or public API adds no value.

Keep exact escaping, including Unicode, quotes/backslashes, control characters
and the existing bidi ranges. Preserve all text/JSON/report/evidence contracts.
The library remains I/O-free. No dependency changes or user instructions needed.

Alternatives: keep per-character vectors (simple expression, costly hidden
allocation), always build one String (removes character vectors but still copies
ordinary text), or borrow ordinary text and allocate only for escaping (selected).
The selected path adds a predicate scan and makes escaped inputs scan their
prefix twice; measurements must include escaped as well as ordinary inputs.
JSON streaming and worker-key redesign were inspected but deferred because their
compatibility/order cost exceeds this self-contained improvement.

## Verification

Pin existing output first using named AAA examples and a 256-case Unicode
property with a numeric-scalar escape oracle. Record identical ignored release
measurements before/after. Then add the borrowing contract test and observe the
old String return fail that type requirement; this is a new allocation contract,
not a claim that existing rendered output is wrong. Keep public process tests,
full stable/shuffled suites, existing Lean and fuzz, quality/docs checks, five
self-reviews per stage and fresh whole-PR review. Existing formal models do not
prove terminal escaping, borrowing or performance.

The inspected boundaries already satisfy eight software-design diagnostics;
this checklist does not measure allocator traffic. Retaining the same callers
while reducing hidden allocation improves implementation without expanding its
responsibility. Scoped score: 8/8 (10/10); no failed diagnostic row identified.
