# Production Rust contracts

Creusot verifies `crates/taloon-decisions`, the same crate used by the production
ledger and report evaluator. Ordinary stable builds do not compile its
`cfg(creusot)` dependency. Observation contracts cover slot cardinality and exact
payload consumption. Retained slots contain only Single/Duplicate; Option/map
absence represents Missing, avoiding a second stored empty-state discriminant.
Verdict contracts cover incomplete/violation precedence, warning-dependent exit
codes and the actual borrowed early-stopping package scan. Additional contracts
cover ASCII package-name validity/canonicalization/idempotence, exact evidence
strength/content/source/request matching, and saturating acquisition counter
merges that preserve elapsed time. The current inventory contains 21 files and
43 named goals per target.

These contracts do not prove BTreeMap selection/removal, complete graph traversal,
warning discovery, serialization, cryptographic hashing, metadata/URL parsing,
allocation success, arbitrary destructors, clocks or I/O. Evidence adapters call
the verified byte-comparison rules; their surrounding parsing/storage remains
outside the proof. See the [scope records](../../docs/development/work/evidence-and-invariants/plan.md)
and [ordered-container probe](../../docs/development/work/evidence-and-invariants/graph.md).
The internal crate keeps version 0.1.0 independently: automatic release preparation
changes only the published root package and its locked entry.

## Pinned environment

The environment reuses the public hoimin recipe at
[`7d473e08ec5ebb871e5df541369f247c20ea6c5a`](https://github.com/HayaoSuzuki/hoimin/tree/7d473e08ec5ebb871e5df541369f247c20ea6c5a/formal/creusot).
The archive checksum in `build-image.sh` pins the fetched bytes. The recipe pins
Creusot 0.14.0 (`65349025c67e69fcd7479f1f420fa5904f69bc27`), nightly-2026-08-03,
Why3/Why3find, Z3 5.1.0, CVC5 1.4.1 and their input hashes. Its preflight checks
installed executable versions, receipts and effective solver configuration.
Its solver commands impose 30-second wall and 2-GiB address-space limits, with
two concurrent provers. This is not an aggregate container memory bound.

Build from the repository root with Docker and authenticated gh:

```sh
formal/creusot/build-image.sh          # Linux amd64, used by CI
formal/creusot/build-image.sh arm64    # optional ARM64 development image
```

The source archive and Docker build context stay temporary. No hoimin checkout
is required. A failed Docker build is retried once after 10 seconds using the
same verified archive, exact context bytes, architecture arguments and layer
cache. This allows recovery from transient upstream acquisition errors such as
Ubuntu snapshot HTTP 502 responses. The wrapper retries any Docker build failure;
it does not classify logs or bypass recipe checks. A second failure returns
Docker's exit status. Source fetch, checksum and context creation failures are
not retried. CI's existing image-build deadline covers both attempts together.

Hosted CI selects the committed `formal/creusot/image.pin`. Its single line is
`IMAGE SOURCE_COMMIT BUILDER_SHA256`, without shell evaluation. `IMAGE` is either
`bootstrap` with source `-`, or exactly the taloon GHCR repository with a SHA-256
digest. The builder hash must match the current build-image.sh bytes. Invalid,
missing, multi-line or stale pins fail. The committed initial pin explicitly
selects bootstrap because no registry image has yet been published.

In bootstrap mode, CI reuses the exact-key Actions image cache or runs the pinned
builder. The key binds the builder, preparation/registry helpers, pin and runner
OS/architecture, with no prefix fallback. GitHub repository/ref cache isolation is
a trust boundary. Both cached and built images pass architecture/installation
checks. An empty bootstrap cache still needs a functioning snapshot source.

With a registry pin, CI authenticates using its read-only workflow token, verifies
OCI build provenance for the exact digest, repository, trusted-main environment
publisher and pinned source/signer commit, then pulls by digest. The image's
builder label must match the current builder. Architecture/installation and all
current-project configuration checks still run. Failed provenance, pull or label
verification stops the job; it cannot select a cache or local rebuild. This path
does not contact the Ubuntu snapshot to reconstruct tools. It still depends on
GitHub/GHCR, and source-dependent proof inputs may need their own acquisition.
Fresh generation, all proof inventories/goals and compiling mutations remain
mandatory. Local build commands above retain their existing behavior.

### Publish and adopt a durable environment

The [environment publisher](../../.github/workflows/creusot-image.yml) builds only
trusted main source, validates the installation, pushes the image and attests its
actual digest using GitHub OIDC. It runs on main changes to its builder/workflow
or through manual dispatch after the workflow exists on main:

```sh
gh workflow run creusot-image.yml --repo HayaoSuzuki/taloon --ref main
```

Successful publication verifies the OCI provenance and builder identity before
uploading the `creusot-image-pin` artifact. Review its actual digest, source
commit and builder hash, then commit that file as `formal/creusot/image.pin` in a
normal PR. The publisher does not write the repository or automatically adopt a
new environment. No digest is fabricated to represent a pending publication.
Changing the builder requires a reviewed replacement pin (or explicit bootstrap
with its new builder hash) and a newly validated image. Proof results are never
part of this image or the cache.

The first environment publication still needs the pinned upstream. Current
snapshot outages therefore block initial bootstrap until acquisition recovers;
these workflow changes do not turn that failure into success. A workflow must be
merged before its manual dispatch is available. Source CI and the application
release gate remain required; recover failed main runs and their original release
using the [release procedure](../../docs/development/releases.md#recovery-and-provenance-verification).
Registry image publication is separate from Python package publication.

Run the synthetic wrapper regressions without network access or Docker:

```sh
bash formal/creusot/test-build-image.sh
bash formal/creusot/test-prepare-image.sh
bash formal/creusot/test-registry-image.sh
bash formal/creusot/test-publish-image.sh
```

These checks use a local recipe archive and substitute its checksum in a temporary
wrapper copy. They exercise orchestration, not the public recipe or real proofs.
The image includes the upstream installation checker and its
manifest; the application never imports those Python tools. The recipe's
Elastic-2.0 attribution remains in the pinned source. Creusot and its standard
contracts are LGPL-2.1-or-later; source and license are available at the pinned
[Creusot revision](https://github.com/creusot-rs/creusot/tree/65349025c67e69fcd7479f1f420fa5904f69bc27).

`why3find.json` at the workspace root selects exactly the recipe's solver
configuration. Keep it in sync when updating the environment. Compiler/Why3/SMT
correctness and the supplied standard-library contracts are trusted boundaries;
no application function is marked trusted.

## Verify the production source

```sh
docker run --rm -v "$PWD:/workspace" -w /workspace \
  -e CARGO_TARGET_DIR=/tmp/taloon-contract-target taloon-creusot:local \
  timeout 1200s formal/creusot/verify.sh
```

The runner cleans only this crate's generated output/build results, generates
fresh Coma files, compares their exact inventory to `obligations.tsv`, then proves
each file without the solver cache. Every named goal and the nonempty per-file
success marker must appear. Source/config checksums are compared before/after
both generation and proof. Input changes, missing tools, compilation failures,
unproved goals and timeouts fail the command. Diagnostics and generated files
remain in ignored `formal/creusot/reports/`; they are not proof inputs.

All three targets compile and prove sequentially inside Linux: x86_64 Linux,
ARM64 macOS and x86_64 Windows. This does not claim native OS execution or prove
OS APIs. Each generation/file-proof command has a 300-second timeout; the outer
command has a 1,200-second timeout. Ordinary Rust regressions still run natively.
Keep the inventory exact when adding/changing contracts; a successful command
with no reviewed goals is not accepted as verification.

Run compiling negative witnesses in a temporary copy:

```sh
docker run --rm -v "$PWD:/workspace" -w /workspace \
  -e CARGO_TARGET_DIR=/tmp/taloon-contract-witness-target taloon-creusot:local \
  timeout 900s formal/creusot/sensitivity.sh
```

Fourteen mutations cover duplicate/consumption, precedence/exit/warnings,
evidence strength/source-request association/content/hashless requests/sidecar
pinning, dropped final aggregation input, incorrect name lowercase conversion,
elapsed-time merging and counter loss. Each must generate successfully, then
fail its own named contract goal. A compiler error, timeout, missing proof target
or unexpected successful proof fails the sensitivity check. The shipping source
is never mutated. These witnesses test contract sensitivity, not complete bug
coverage.

## CI enforcement

The existing required `Lean proofs and generated corpus freshness` context runs
both the unchanged Lean checks and the Creusot build/proofs/sensitivity checks.
Its 30-minute job deadline bounds the complete run; CI uses additional 1,200-second
image-build, 600-second verification and 600-second sensitivity deadlines. A
timeout fails the existing required context. The extra tooling is built from
checksummed source on each fresh runner; cold setup can dominate job time. Proof
reports are retained for seven days on success or failure. This task does not
change GitHub rules or add an unenforced optional proof job.
