# Creusot 0.15 published image adoption

The trusted-main [publisher run 45308067272](https://github.com/HayaoSuzuki/taloon/actions/runs/45308067272)
succeeded for merged source `301ed0fbfe1b1ccb2f7b2069ffa923dca839fd95`.
Build/install validation, push, attestation, provenance/builder verification and
candidate pin upload succeeded. The adopted pin bytes are copied unchanged from
its `creusot-image-pin` artifact (ID `14883280397`). GitHub reports artifact ZIP
digest `sha256:e281dca272122dc0a98789ecf88d5ef84eb50820460d4a65385c5b25427ba74f`.

- Image: `ghcr.io/hayaosuzuki/taloon-creusot@sha256:2952b1f8f521d50b65cfeb56410e2bd8c328ded85468a15228954973c45e9eba`.
- Source/signer: `301ed0fbfe1b1ccb2f7b2069ffa923dca839fd95` on trusted main.
- Builder SHA-256: `442f0b1f18ff131679f06754eae6d7fb491031a0232ad369e18b921c2d5d1b0a`.

The builder hash was independently compared with current build-image.sh.
The existing pin reader selects registry mode; no consumer security check or
bootstrap fallback behavior was changed. OCI attestation verification binds the
exact repository, workflow, source ref and source/signer commit, and rejects
self-hosted runners. CI additionally pulls amd64 by digest, checks builder and
installation, and runs fresh proofs and compiling negative witnesses.

The new snapshot ARM64 recipe image built locally and passed installation
preflight, all 21 proof files on each of three compilation targets, and all
14 negative witnesses. PR #41 also passed the full hosted amd64 bootstrap and
proof/sensitivity job before its merge. These results do not substitute for the
adoption PR's actual registry-path CI.

## Required check configuration

The renamed nightly job initially left the live main ruleset expecting the old
date, so ordinary gh merge was refused despite all current checks succeeding.
Pulumi preview showed only the exact required-check context replacement. Apply
updated one ruleset; seven resources remained unchanged. Final refreshed preview
reported eight unchanged resources. All 13 checks, Actions integration binding,
strict current-main policy and no-bypass settings remain in force. The matching
program and canonical state are included with this adoption; no protection was
removed or administratively bypassed.
