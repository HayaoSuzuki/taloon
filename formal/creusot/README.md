# Production Rust contracts

Creusot verifies `crates/taloon-decisions`, the same crate used by the production
ledger. Ordinary stable builds do not compile its `cfg(creusot)` dependency.
The current observation contracts concern slot cardinality and exact payload
consumption. They do not prove BTreeMap key selection/removal, request identity,
Evidence rebinding, parsing, allocation success or I/O.

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
is required. The image includes the upstream installation checker and its
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

The duplicate and consumption mutations must generate successfully, then fail
their own named contract goals. A compiler error, timeout, missing proof target
or unexpected successful proof fails the sensitivity check. The shipping source
is never mutated. These witnesses test contract sensitivity, not complete bug
coverage.
