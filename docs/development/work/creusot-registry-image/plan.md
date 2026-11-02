# Durable Creusot image implementation plan

**Goal:** Separate tool-environment publication from current-source proof checks,
using an attested, digest-pinned GHCR image once bootstrapped.

**Authorization:** The owner requested durable GHCR tool environments.
Continue this environment work in PR #22.
Use software-design-philosophy, writing-plans, TDD and five reviews per stage.

## Selected design and alternatives

Increasing retries cannot solve sustained snapshot outages. An Actions cache is
useful but disposable. Use an explicit committed image pin plus a trusted-main
publisher, retaining the reviewed cache/builder only during explicit bootstrap.
Do not resolve a mutable tag during source CI or silently rebuild after a bad pin.

The one-line image.pin contains IMAGE SOURCE_COMMIT BUILDER_SHA256. IMAGE is either
literal bootstrap (SOURCE_COMMIT '-') or this repository's exact GHCR name with
sha256 digest. The current builder bytes must match BUILDER_SHA256. A bootstrap
pin is committed initially because no image digest has yet been published; never
invent one. Missing/malformed pins fail, rather than enabling bootstrap silently.

The builder labels its image with the SHA-256 of its own script and repository
source URL. Registry consumption verifies GitHub OCI build provenance for the
exact image, trusted-main publisher workflow and source/signer commit; it pulls
by digest, checks the builder label, then reuses architecture/installation checks.
All project configuration checks, fresh proofs and mutations remain mandatory.

A separate workflow publishes only from main on builder/workflow changes or a
manual main dispatch. It builds/validates once, pushes a source-commit tag, attests
the exact registry digest and uploads the actual candidate pin as an artifact.
Review and commit that pin to switch normal CI to GHCR; there is no automatic
repository write. Publisher permissions are contents-read, packages/attestations
write and OIDC only in its trusted-main job. Source CI uses read permissions.

The first environment build still needs the pinned snapshot. Initial HTTP 502 blocked bootstrap; later HTTP 200 allowed resubmitting the
original main CI, while the new publisher still needs a merged workflow; GitHub package listing with the local CLI also lacks
read:packages, so publication uses workflow-scoped GITHUB_TOKEN after merge.
No token upgrade or secret is requested. Record implementation versus unobserved
first publish/adoption honestly, and continue independent deny work regardless.

## Tasks and interfaces

- [x] Registry reader: formal/creusot/registry-image.sh PIN BUILDER [--mode].
  --mode outputs bootstrap/registry only. Normal bootstrap pull is rejected; registry
  verifies/pulls/labels/tags, and any failure propagates. Bootstrap is selected
  only by successful --mode classification, never an external exit status. Add named synthetic
  tests with real pin/hash parsing and gh/docker doubles, RED before GREEN.
- [x] Bind builder label and validate it; extend build tests and preparation's
  optional third pin argument. Add current builder-bound bootstrap image.pin.
- [x] CI chooses bootstrap cache or pinned GHCR; login only for registry. New
  trusted-main creusot-image.yml publishes/attests/upload candidate pin. Fixed
  Action SHAs and strict workflow lint/security checks.
- [x] Update formal procedure and existing verification owner; source hashes,
  five-pass reviews, fresh review, tests and PR #22 body limited to this change.

Review focus: pin mutation/multiple lines; exact repository/digest/source hash;
failed provenance cannot pull/run or fall back; recipe changes invalidate pins;
no publishing from PR/other branches; no fabricated successful bootstrap.

Initial slice score 8/10: ephemeral environment reuse is deeper than raw build,
but durable identity and publication lifecycle are missing. The pin reader hides
validation/provenance decisions from CI; the publisher owns environment creation.
