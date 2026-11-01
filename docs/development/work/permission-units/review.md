# Permission unit staged review

Base `1cc77aa737829e56306e57182dadcb9bdd719bea`, 2026-11-01, macOS arm64.
This extends open PR #10 under the owner's repeated design/type improvement
request and earlier discussion of validated permission units.

## Design: five passes

1. Kept catalogue ownership in spdx 0.13.4. Existing LicenseExpression remains
   the Python SPDX validation owner; no per-license Enum or second AST is added.
2. Inspected native requirement comparison and lexer mode. Canonical text remains
   the application identity; GNU only/later and non-GNU plus remain distinct.
3. Chose private fields plus validating construction, not an unchecked String
   conversion or Deserialize. Type identity, ordering and borrowing agree on
   canonical spelling independently of the classification Enum.
4. Kept scope separate from syntax: custom units are constructible, but global
   permissions still reject them and exact hash-bound overrides control proof.
5. Preserved the existing canonical string reporting view and report schema.
   This view is an intentional output contract, not typed policy backing storage.

## Implementation: five passes

1. Policy/global/scoped collections use PermissionUnit. The parser moves the
   existing canonical String instead of cloning it after expression validation.
2. Private Standard/CustomReference Kind comes from LicenseItem, including WITH.
   Minimum evidence and scope checks use that classification, replacing repeated
   substring classification without adding a public state flag.
3. Eq/Ord/PartialOrd/Borrow compare the same canonical text. No native requirement
   Eq is derived, and report ordering and borrowed membership remain unchanged.
4. New constructor tests first failed to compile for the absent public type.
   Exploratory positive cases then exposed a test assumption: the catalogue has
   GNU-plus spellings rejected by existing strict parsing. Corrected the tests,
   retained fixed negative cases and did not widen production syntax. Those
   erroneous generator seeds are not claimed as production regressions.
5. Fixed cases cover valid/invalid atomic forms, custom WITH, global custom
   rejection, proof requirements, GNU distinctions and whitespace normalization.
   The generated property checks set/borrow/order coherence, not independent
   parser correctness. It samples the old parser's supported catalogue subset.

## Independent review and final verification: five passes

1. Fresh whole-PR review found no Critical/Important defect. The reviewer initially
   ran 84 focused tests and suggested custom WITH coverage plus 256-case checking.
2. Added the requested custom WITH constructor/global/proof cases and raised both
   relevant generated test groups to 256. Reviewer follow-up passed 68 focused
   tests and confirmed the minor findings resolved.
3. Rechecked legacy public units() as a reporting view. Reviewer withdrew its
   initial partial diagnostic after confirming typed policy storage and native
   AST remain independently replaceable. No API migration is needed for this work.
4. Full stable/shuffled suites, Clippy/rustdoc, existing Lean freshness/strict
   adapter, bounded policy/inspection fuzz and documentation/Python checks passed
   on the final tree. No proof/evidence rule or runtime dependency changed.
5. Inspect final staged diff/source hashes and obtain GitHub CI results for the
   actual final SHA. Keep timing samples and limitations separate from type
   guarantees; the change does not promise a runtime speedup.

All eight scoped diagnostics pass (10/10): coherent responsibilities, deep API,
replaceable internals, accurate comments, design review, information hiding,
understandable boundaries and explicit design investment. No failed row remains.
This score is a checklist assessment, not a claim of exhaustive correctness.

Existing Lean models/corpora remain unchanged. They prove their original model
properties; strict Rust adapters still exercise those public semantics. New
constructor/trait tests are separate implementation evidence, not new theorems.
No user command/syntax changed, so the user guide requires no update. No Python
implementation changed, so mutation testing of Python is not applicable here.


Final local results: 198 regular Rust tests passed in stable, generated shuffle
and seed-12345 runs. Python 134 passed; policy and inspection fuzz each completed
1,000 runs at max_len 4096. Lean build completed in 0.448s under the retained
20s deadline, and all three existing corpora were freshness-checked. Constructor
syntax/proof scope did not expand. The first red result was the absent public
type (compile failure), not a claimed pre-existing runtime defect.
