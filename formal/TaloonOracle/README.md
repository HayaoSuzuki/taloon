# Taloon semantic oracle

The separate [release decision model](TaloonOracle/Release.lean) proves existing
reservations are reused, stale unreserved sources are rejected, and fresh
reservations increase the version ordinal and respect its floor. Its premises
assume correct ancestry observations and atomic compare-and-swap. It does not
prove Git/network behavior or the Python implementation; local bare-remote and
generated version tests cover that correspondence separately. The ordinal model
omits the first empty-history reservation; that case is an explicit Python test.

This dependency-free Lean project follows the model/proofs/cases/corpus
organization inspected in hoimin at revision
`542ec59c002d72fdff0137aa90113ffa59d70844`. The taloon model is new code;
hoimin's mutation-score semantics are not part of this checker.

Lean v4.32.2 proves properties of the [semantic model](TaloonOracle/Model.lean).
The [proofs](TaloonOracle/Proofs.lean) establish permission monotonicity for
arbitrary expression trees and pointwise permission extensions, inclusive
expiry, rejection of past expiry, archive/sidecar strength, missing/insufficient-target
rejection over arbitrary expected/observation lists, and incomplete-over-violation
precedence. WITH remains one atom. No theorem proves
that a metadata declaration is legally complete or that Rust parsing/I/O matches
Lean for arbitrary inputs.

The [generator](Main.lean) computes expectations; do not edit the generated
JSONL by hand. Its initial finite domain has seven expressions, six allowed
sets (42 permission cases), and three expiry boundaries. It detects deliberately
broken OR, AND, expiry, sidecar-promotion, missing-result, and exit-precedence
implementations. Stage 4 adds 432 inspection cases over two expected targets,
missing/sidecar/archive slots, proof requirements, permission, duplicate/unknown
results, and warning escalation (477 strict cases at that checkpoint).
Stage 8 adds custom-LicenseRef alternatives: the current domain has eight
expressions × six allowed sets (48), three expiry cases, and 1,296 inspection
cases crossing three declarations with the earlier inspection dimensions, for
1,347 strict cases. The expansion follows an actual interaction regression, not
a larger exhaustive proof search. Imported proof modules contain no
large exhaustive search. Individual commands initially use a 20-second external
deadline and bounded default heartbeats; one Lean process runs at a time.

## Correspondence worksheet

The [verification decision](../../docs/development/knowledge/verification-strategy.md)
recorded the premises before model construction. The public Rust adapter now
executes each generated case in isolation:

| Claim | Lean premise | Rust configuration and public observation | Mode |
| --- | --- | --- | --- |
| Atomic permissions, AND/OR | Expression plus exact allowed strings | Policy::from_pyproject and LicenseExpression::parse; Policy::evaluate result | strict |
| WITH/GNU distinctions | Separate atomic strings | Same identifiers through the public evaluator | strict |
| Inclusive expiry | Today 30; expiry 29, 30, or 31 | 2026-10-30 and the corresponding valid October date; policy construction outcome | strict |
| Evidence, completeness, exit | Two expected IDs, three slot states, proof/policy/anomaly/warning combinations | Inspection::from_documents and evaluate, Report::exit_code | strict |
| Unbounded monotonicity | Arbitrary finite trees and Boolean predicates | Corpus exercises the stated finite subset; property tests generate further trees | model-only |
| Unbounded natural dates | Nat ordering | Rust calendar has a finite range; corpus uses only valid same-month dates | model-only |

The [Rust adapter](../../tests/oracle.rs) derives no expected values. Invalid
case schemas/adapter setup failures are infrastructure errors; a differing
observed permission or expiry result is a semantic mismatch named by case ID.
All 1,347 current strict cases matched after stage 8. No unresolved counterexample
remains in this finite slice. Cache/network concurrency remains outside these proofs.
Stage-4 proof/generator builds took 3.076/3.896 seconds under the unchanged
20-second deadline; no peak-memory measurement was available. Stage-8 generator
and extended proof builds took 4.936 and 3.522 seconds under the same deadline.

## Reproduction from the repository root

```sh
python3 -c "import subprocess; subprocess.run(['lake','build','TaloonOracle','generate_corpus'], cwd='formal/TaloonOracle', timeout=20, check=True)"
formal/TaloonOracle/.lake/build/bin/generate_corpus --stats
formal/TaloonOracle/.lake/build/bin/generate_corpus --check tests/fixtures/oracle/policy.jsonl
cargo test --locked --test oracle
```

After a model change, regenerate with `--output` instead of `--check`, inspect
the diff, then check correspondence. Preserve broken-variant sensitivity.
Initial measured build time was 4.792 seconds; no memory measurement was
available and no exploration bound was raised. This is local execution evidence,
not a cross-platform timing guarantee.
