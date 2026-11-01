# Verified saturating work accounting

The original three summary structs and their field order/types move unchanged
to the kernel, reexported from report. Actual merge bodies prove exact u64
saturation and monotonicity for all twenty combined counters, and preservation
of the receiver's elapsed_us. The other elapsed_us is intentionally not added.
No extra retained fields, temporary collections or allocations are introduced.

Twelve cases/properties pass, using independent u128 addition/clamping and
asymmetric elapsed boundaries. An isolated compiling elapsed-summing mutation
fails vc_merge'0. Exact source was restored, generated output removed and two
files proved again without cache. All existing call sites use these bodies.

An initial forty-attribute contract exceeded macro recursion depth. Grouping
exact/monotonic clauses into conjunctions preserved every field condition without
raising the compiler recursion limit. No solver/standard-library axiom was added.

Five reviews: (1) complete twenty-field coverage; (2) root reexports/actual call
connection; (3) identical storage and no copies; (4) independent arithmetic and
rejected elapsed mutation; (5) exact-source fresh proof and explicit clock,
transport and serialization boundaries. Duration conversion and real-clock
observation remain runtime-owned. Integrated results are recorded in PR #12.
