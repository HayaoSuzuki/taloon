# First reviewed GHCR image adoption

## Published evidence

The trusted-main [environment publisher run 41450013770](https://github.com/HayaoSuzuki/taloon/actions/runs/41450013770)
completed successfully for source `8b9cd3a238985ba8cd827bf6e94fce674f25b648`.
Its build/validation, registry push, OCI attestation, provenance/builder verification
and candidate-pin upload steps all succeeded. This is hosted evidence, distinct
from the earlier synthetic tests and bootstrap-only state.

Adopt the candidate artifact bytes unchanged:

- Image: `ghcr.io/hayaosuzuki/taloon-creusot@sha256:730f97792c5b7b0a0ae75a9a1ce3d254ffb8d7a3dc50d1b0e4debc6ac632d523`.
- Publisher source/signer commit: `8b9cd3a238985ba8cd827bf6e94fce674f25b648`.
- Builder SHA-256: `6be6ce8a171f606cf2549cf7ac60e27c110126bd63d98aa23b134176e15fcb1b`.
- Artifact: `creusot-image-pin`, ID `13145202110`, from that exact main push.
- Artifact ZIP digest reported by GitHub: `sha256:9deef76d729d7672521664b1c7b0ec4fa5d3e35aaa5c3862f2151403da9910f2`.

The bootstrap form remains available for explicit future environment replacement;
this change makes normal source CI select registry mode. It does not publish or
adopt a mutable tag, rebuild tools, change their pinned versions or cache proofs.

## Five self-review passes

1. Source identity: require the expected repository's successful main publisher
   at the exact merged source. Check every publish/provenance/upload step rather
   than assuming that a pushed tag alone proves successful publication.
2. Bytes and binding: compare the adopted pin byte-for-byte with the run artifact,
   verify its artifact archive digest, and independently hash current build-image.sh.
   The recorded source and builder match the published receipt.
3. Production connection: run the existing reader's --mode on the actual pin;
   it returns registry. CI therefore skips bootstrap cache/build and uses the
   mandatory provenance/pull/builder/installation path. No consumer code changes.
4. Security boundary: preserve exact repository/digest/source/signer checks,
   architecture validation, no fallback after a failed registry operation,
   current-source configuration checks, fresh proofs and compiling mutations.
   The seven builder, fifteen preparation, seventeen registry and eight publisher
   regressions still pass. No new implementation-mirroring tests were added.
5. Documentation and limits: update only the existing formal procedure and
   verification owner, citing this actual snapshot. The publisher's hosted
   verification is observed; the first normal source CI using the adopted pin
   must be checked separately on the adoption PR. Local Docker/GHCR verification
   is not claimed. No production Rust/Python, dependencies or release gates change.

Follow the [formal environment procedure](../../../../formal/creusot/README.md#publish-and-adopt-a-durable-environment)
for later rotations. This record does not assert new parser, registry, compiler
or solver correctness beyond the existing trust boundaries.

A fresh-context reviewer independently checked the ZIP/pin bytes, builder hash,
registry mode, unchanged consumer/publisher guards and documentation source hash.
Its five-pass review found no Critical, Important or Minor issue. Normal consumer
CI with the adopted pin remains separate hosted evidence.
