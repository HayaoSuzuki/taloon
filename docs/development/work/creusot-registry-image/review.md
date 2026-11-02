# Durable, attested Creusot image review

## Scope and observed limits

The owner requested both durable GHCR environments and the separate deny feature.
This record and PR #22 concern environment acquisition only. The
[plan](plan.md) separates environment publication from current-source proof checks.
The existing cache-only contract was insufficient for durable bootstrap: cache
entries may expire and the initial cache was empty during the snapshot outage.

The initial pin explicitly contains bootstrap and the actual current builder
hash. There is no published image digest to adopt yet. At the initial investigation on 2026-11-02, the pinned
snapshot returned HTTP 502; a local gh package-list request lacked the
read:packages scope. No token upgrade, manual image publication or fabricated
successful bootstrap is claimed. The trusted-main workflow uses its own scoped
GITHUB_TOKEN after merge, not a stored user token. Its first real build/push and
pin adoption remain operational prerequisites requiring successful acquisition.
A later probe during implementation returned HTTP 200, and the original failed
main CI was resubmitted. This is not proof of complete cold acquisition or image
publication. No main merge, publisher dispatch, Python release or policy bypass
was performed.

## Consumption: five review passes

1. Boundary domains: one terminated, three-field pin selects explicit bootstrap
   or the exact own-repository SHA-256 image. Missing, empty, extra-field,
   unterminated, multi-line, mutable-tag/source and foreign-repository inputs fail.
2. Identity connection: the builder labels its own script hash. The reader
   requires current builder bytes to match the pin and pulled image label.
   OCI verification binds the digest to repository, publisher workflow, main,
   exact source/signer commit and hosted runners before Docker pulls/runs it.
3. Failures: invalid pins, provenance, pull, labels and tagging fail closed.
   An independent reviewer found that exit 3 from gh could collide with an early
   bootstrap sentinel. The old path was rejected by a RED integration case;
   successful --mode classification now selects bootstrap independently of exit
   codes. The reviewer reproduced gh exit 3 after the fix: status 3, no build or
   Docker execution. No failed registry path falls back to cache or rebuilding.
4. Cases: 17 named registry and 15 preparation cases exercise pin grammar,
   mode probes, exact provenance arguments, architecture/installation/cache
   behavior and external failure statuses. Mac Bash 3.2 exposed an empty-array
   nounset issue in the added test harness; nonempty positional arguments fixed
   it. Temporary fixture paths are normalized for stable script-directory checks.
5. Structure/performance: one reader owns format, builder binding and provenance;
   preparation reuses architecture/installation checks. Registry mode skips the
   disposable image cache and builder entirely. A successful pin mode query is
   repeated before pulling so bad state cannot silently become bootstrap. No
   source/proof outcome, mutable-tag lookup or generic policy layer is introduced.

## Publication: five review passes

1. Scope: only the expected repository and main at an exact source commit may
   publish. The workflow also guards those identities; there are no arbitrary
   image/source/ref inputs and no PR publishing event.
2. Connection: publication forces the existing checksum-pinned cold builder,
   validates installation and image builder identity, pushes a source tag and
   retrieves the actual matching RepoDigest. A source tag is for storage only;
   consumers require the committed digest, not resolution of that mutable tag.
3. Provenance: the pinned actions/attest Action receives the actual registry
   digest with push-to-registry enabled. The candidate pin is uploaded only after
   the normal registry reader independently verifies the signed OCI identity.
   Contents are read-only; packages/attestations write and OIDC are confined to
   the trusted publisher job. The workflow never writes/adopts a repository pin.
4. Failures: eight named producer cases pin trusted source, rejected branch/repo,
   invalid source, failed build, stale builder label, failed push and missing
   registry digest. No failed producer case emits a candidate pin.
5. Operational bounds: the existing 1,200-second preparation deadline and
   30-minute jobs remain. A first build still depends on the fixed upstream;
   environment publication/adoption is a distinct lifecycle, not a way to mark
   unperformed source proofs as successful.

## Integration/documentation: five review passes

1. CI selects only a validated bootstrap/registry mode. Read-only package and
   attestation access is job-scoped. Bootstrap cache keys now include all reader
   inputs and the pin; locked mode authenticates and pulls without cache restore.
2. Every source still executes project preflights, fresh proofs and mutations.
   No release gate, production Rust, Python source or dependency graph changed.
3. The formal README owns publishing/adoption and bootstrap limitations; the
   existing verification concept cites these exact review bytes. Historical
   cache/retry reviews remain evidence of their own revisions.
4. Local verification includes 47 shell cases (7 builder, 15 preparation,
   17 registry, 8 producer), Bash syntax, pinned ShellCheck/actionlint/zizmor,
   137 Python tooling tests, OKF/link checks and strict user-site build.
   Workflow ShellCheck found two unused receipt fields; underscore names fixed
   the issue without suppressing checks. A shared test-variable name was split
   across independent subshell fixtures to avoid a static-analysis ambiguity.
5. A fresh-context reviewer completed five passes and found the exit-status
   collision above; it independently approved the corrected control-flow scope
   and reran the 40 registry/preparation/producer cases under Bash 3.2. Actual
   Docker/GHCR publication, attestation, artifact delivery and cache use remain
   unobserved locally. This is executable orchestration evidence, not a new
   Lean/Creusot proof of the shell, OCI verifier or package registry.

Five additional syntactically valid mutations were rejected: foreign image
acceptance, missing builder binding, changed trusted source ref, swallowed
registry failure and publication from an untrusted branch. These witnesses are
finite sensitivity checks, not exhaustive verification.

Design score: 9/10 for the slice. Pin parsing and provenance are hidden behind a
small reader, while the publisher owns environment creation. The explicit
bootstrap/adoption lifecycle is necessary until actual signed bytes exist;
revisit it if automated reviewed pin updates are needed. Further abstraction
without another consumer would add more interface than capability.
