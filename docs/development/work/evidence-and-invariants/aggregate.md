# Verified report traversal

The previous contract proved pairwise decisions and expressly excluded the
collection loop. The actual aggregation loop now belongs to the decision kernel.
A generic PackageOutcome retains the original named fields, while the root
PackageReport is a concrete type alias. No wrapper, Deref, flattening, cloned
payload or collected status vector is introduced; serde field-shape regression
passes. The existing package vector is borrowed and retained unchanged.

Quantified prefix invariants prove exact Incomplete/Violation/Compliant results,
including input errors and empty input. A decreasing remaining-length variant
proves termination and index bounds; early Incomplete return preserves the same
result as complete traversal. The caller's diagnostic-to-input_error and warning
classification, serialization and inspection remain tested boundaries.

Eight independently executed cases/property and five root report cases pass.
The isolated actual-source aggregate goals vc_len_PackageOutcome_P_S_A_D_L_N
and vc_aggregate_P prove without application axioms. Independent maximum-rank
properties exercise both kernel and real Report::new adapters.

Five reviews: (1) all precedence/empty/error cases; (2) actual direct call; (3)
borrowed named-field storage/no extra scan or allocation; (4) independent rank
oracle and loop invariants; (5) source equality, termination and caller boundaries.
Integrated proof/sensitivity results and exact source revision are recorded in
PR #12 rather than inferred from these isolated experiments.
