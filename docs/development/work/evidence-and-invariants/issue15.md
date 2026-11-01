# Declaration reporting correction (Issue #15)

Issue #15 recorded that applying locator redaction to entire declaration strings
lost otherwise readable prose. At b19f596, four named reproduction cases (email,
question, URL-containing prose and folded metadata) failed with whole-declaration
placeholders. All four pass after the correction. Only public synthetic fixtures
were used; no sample repositories were consulted.

A private declaration owner scans whitespace-delimited URL tokens and reuses the
locator policy for those tokens. Prose/email remains text; malformed URL-like
tokens fail closed individually. Trailing prose punctuation is retained. URL
credentials containing punctuation, IPv6 hosts and multiple URLs have regressions.
Permission parsing continues to read the original metadata, independently of
report rendering. Cache payloads/checksums are unchanged.

Four command/header combinations reproduced raw bidi controls in JSON after
preserving prose. Shared CLI JSON formatting now emits Unicode escapes for the
same disruptive scalar predicate as text output; decoding the JSON recovers the
exact declaration. A generated arbitrary-Unicode roundtrip property checks both
losslessness and the absence of disruptive literal scalars. JSON and text escaping
remain distinct from URL redaction. Check/describe still own their report schemas.

Five review passes:

1. Requirements: original_expression/original_license and check/describe JSON
   each have independently executed cases; header precedence/permission is unchanged.
2. Ownership: declaration prose no longer enters a whole-string URL operation;
   only extracted tokens do. Both CLI JSON surfaces share the actual safe writer.
3. Security/storage: malformed schemes, quotes inside credentials, IPv6 and query
   fragments are covered; ordinary emails survive. One output buffer is used;
   no copies of cache records or declaration parsing algorithms are added.
4. Sensitivity: four prose cases and four bidi CLI cases failed before their fixes;
   fixed cases and independent generated email/credential/Unicode properties pass.
5. Reproduction/docs: synthetic archives exercise the public Evidence and binary
   entry points; user evidence guidance and the owning inspection contract are updated.

This is rendering/test evidence, not a proof of URL parsing or legal completeness.
