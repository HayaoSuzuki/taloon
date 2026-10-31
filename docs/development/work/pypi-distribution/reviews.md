# Distribution implementation evidence

## Stage 1: binary packaging

At the working tree based on `55b6134`, the metadata regression first failed
because the project was `taloon-development`. After configuration it passed.
Maturin 1.15.0 built `taloon-0.1.0-py3-none-macosx_11_0_arm64.whl` locally.
The actual uv wheel smoke passed on Python 3.14 and 3.15 using the same file.
All fixtures are synthetic. Local evidence does not establish other platforms
or a public-index upload.

Five self-review passes:

1. Checked metadata ownership: Cargo alone supplies version; Python declares
   dynamic version and an exact `>=3.14,<3.16` support range, with no dependencies.
2. Checked executable packaging: bin bindings produce a platform wheel, with
   macOS 11 arm64 tag; no Python wrapper or extension-module ABI was introduced.
3. Exercised actual uv installation for both supported interpreters with isolated
   tools, preventing an existing installed taloon from satisfying the invocation.
4. Exercised paths containing spaces and cold/warm cache, then changed policy
   without deleting evidence: exit 0 changes to 1; missing local bytes yield 2.
5. Reviewed developer/runtime separation and truthful guidance: docs dependencies
   remain a separate group; the guide does not claim a PyPI release exists.

The documentation checker and 29 Python tooling tests passed. Rust source was
unchanged; the baseline Rust suite passed before packaging changes.

## Stage 2: reservations and inventories

The new release modules first failed test collection because they did not exist.
After implementation the reservation and inventory suite passed. Three further
tests reproduced acceptance of a missing executable, inconsistent internal wheel
tags, and unexpected Python dependencies; all now reject those inputs. The actual
local macOS wheel also passes the validator.

Five self-review passes:

1. Exercised real bare-remotes with repeated commits, stale/unrelated commits,
   raised manifest floors, collisions, and a successful push with a lost response.
2. Exercised simultaneous independent checkouts: immutable tag identities and the
   compare-and-swap ordering marker agree; older work can be explicitly skipped.
3. Checked version ownership and strict syntax, plus 1,024 generated floor/order
   assertions. Only the root package version in Cargo.lock is changed.
4. Checked exact three-platform inventory and no staged output on failure;
   missing/extra wheels, substitutions, duplicate checksums/ZIP paths, wrong
   declarations/text, scripts and tags are rejected before staging.
5. Reviewed formal claims: the new Lean release decision theorems build within
   the initial 20-second bound. They assume ancestry/CAS and do not prove Git,
   Python execution, empty-history handling, signatures, or network behavior.

The release policy/inventory code credits hoimin at the inspected revision.
No upstream vendored-license requirements were carried into taloon's inventory.
