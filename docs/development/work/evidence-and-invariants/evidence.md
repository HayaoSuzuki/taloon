# Verified evidence strength and identity binding

Actual 32-byte content/source/request identities are compared by the standard
array equality operation, with its pinned Creusot contract. Borrowed ProofBinding
views leave private persisted Proof values and cache encoding unchanged. Root
ContentHash/RequestId wrappers lend their actual bytes; no formatted IDs or
comparison-result booleans replace the identities in the proof.

The production helpers prove no sidecar-to-archive promotion, exact source and
request association, rejection of differing locked content, hashless same-request
reuse, and the present matching lock digest required for sidecars. Evidence
constructors, summary strength, rebind and from_record call these actual bodies.
SHA-256, schema/checksum validation, metadata parsing and runtime fresh hashing of
hashless cache content remain outside these proofs and retain their guards.

Four strength cases, four sidecar-lock cases and three independent generated
single-byte-mutation properties exercise the kernel. Existing archive/sidecar,
wrong identity, wrong content, cache and shared-record regressions exercise the
public adapters. Full integrated source generation proves all helper obligations;
root identity comparisons are not marked trusted.

Five reviews: (1) weak/strong and pinned/unpinned domains; (2) real constructor and
reuse call sites; (3) borrowed fixed-size IDs/unchanged cache schema and no extra
payload copies; (4) independent changed-byte oracles and public error outcomes;
(5) fresh whole-kernel generation with cryptographic/caller trust boundaries stated.
Final three-target and rejected-mutation evidence is recorded in PR #12.
