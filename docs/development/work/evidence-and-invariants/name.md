# Verified package-name normalization

The production String adapter delegates to the actual byte normalizer and consumes
its Vec with String::from_utf8; one output allocation is retained. The accepted
domain is unchanged: nonempty ASCII alphanumeric endpoints, with only ASCII
alphanumeric or hyphen/underscore/dot inside. Unicode and malformed names retain
the same Configuration error and message.

Six isolated actual-source files prove validity iff, exact recursive prefix
semantics, lowercase/collapsed separator canonicality, output length and both
loop variants. Inductive prefix/full fixed-point lemmas strengthen the actual
normalizer's result contract, establishing normalization idempotence through its
semantics. The lemma call is verifier-only, adding no runtime work.

Thirteen component cases/properties and the existing public normalization
property pass. A permanent public adapter suite covers five invalid-input errors.
An isolated compiling +32-to-+31 lowercase mutation fails vc_normalize; source
was restored and no-cache proof rerun successfully. Numeric ASCII constants in
Pearlite avoid a reproduced Creusot 0.14 byte-literal translation failure; runtime
code retains byte literals. No trusted axiom or domain bound was added.

Five reviews: (1) original acceptance/rejection boundaries; (2) direct public
adapter; (3) consumed allocation/no copying; (4) independent component oracle,
fixed points and rejected mutation; (5) exact-source regeneration and explicit
UTF-8/allocator/compiler trust boundaries. Full integrated three-target results
and final named inventory are recorded in the PR. This does not prove the Python
metadata parser or allocator behavior.
