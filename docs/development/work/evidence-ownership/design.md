# Immutable evidence payloads

Base `e7badc3ee22f5b5756ee0c1d0099ff44ab7461a0`, 2026-11-01. This
software-design-philosophy follow-up extends open PR #10. Inspection of evidence,
cache and observation accounting found repeated deep copies of immutable record
payloads during rebind and persistence. The public boundaries already satisfy
8/8 diagnostics (10/10 for this inspected scope); internal allocation can improve
without adding interfaces. This is not a score for every possible project input.

## Selected design

Store the private immutable RecordData behind Arc inside EvidenceRecord. Keep
parsed metadata and observed request identity in Evidence. Existing rebind,
to_record, from_record, cache Envelope and ledger APIs remain unchanged. serde's
rc feature serializes the inner object with the same data/checksum fields;
record deserialization moves its owned String into the shared allocation.

Retain every checksum/schema/size/format/content/source/identity/metadata check,
including the hashless observed-request guard and exact sidecar association.
Sharing is a storage decision, not a new evidence strength or trust shortcut.
Atomic reference counts permit existing cross-thread acquisition. No mutable
payload access is exposed. Parsed metadata and checksum serialization remain
allocated/recomputed as before; sharing does not eliminate all cloning.

## Alternatives and execution ruling

Retaining deep copies needs no dependency feature but repeats metadata-sized
allocations. Consuming rebind plus borrowed persistence APIs were prototyped and
measured, then replaced: their extra public methods, shared validation helper and
generic cache envelope impose more interface cost than private sharing. Sharing
only Arc<str> would copy metadata on deserialization into an unsized allocation;
sharing the complete sized RecordData moves the String without that copy.
Skipping revalidation was rejected because it changes trust reasoning.

Ruling: replace the ownership-API prototype with private payload sharing. The
cost is one retained shared allocation per record and atomic reference-count
operations. Locked serde also uses a transient Box<RecordData> allocation when
deserializing Arc; metadata String bytes move without a metadata-sized copy.
Workloads with short records may not benefit.
No new package/version, cache schema, CLI behavior or runtime prerequisite.

## Verification plan

1. Preserve public proof-binding behavior with named AAA cases and a 256-case
   declaration/strength property. These characterize existing semantics rather
   than demonstrate a newly fixed behavioral defect; the prototype's missing-API
   failures are not evidence for the final representation change.
2. Measure identical synthetic check/describe workloads before and after. Keep
   all samples, exclude setup, distinguish avoided copies from measured speed.
3. Run stable/shuffled suites, Clippy/rustdoc, existing Lean correspondence and
   relevant bounded fuzz, Python/docs checks, five self-review passes per stage
   and a fresh independent whole-PR review. Commit/push and observe CI.

No user documentation changes are needed because commands and cache bytes retain
their contracts. Measurement excludes I/O and does not establish peak RSS.
