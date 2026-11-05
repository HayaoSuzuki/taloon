# Remove obsolete compatibility

## Design

Remove the four unused error aliases. Error/Code already carries actionable
diagnostic categories across acquisition, interpretation and reporting. New
module-specific wrappers would add conversions without distinct recovery or
construction guarantees. Reconsider separate error enums when an API needs
different caller handling enforced by its return type.

Evidence payload schema 2 requires an explicit license-file observation state:
NotRead, Complete or Unavailable. Fold the optional wrapper into FileObservation
so missing/null fields cannot stand for an old record. The evidence owner owns
schema validation; obsolete payloads become cache misses without migration.
Keep the disk envelope and filename layout: their representation is unchanged,
and cache clear can still remove recognized old files without a legacy reader.
Current sidecars and archives with adequate declarations can still be NotRead.
Acquisition, proof strength, field priority and all license spellings are unchanged.

## Implementation

1. Add independent failing cases for omitted/null file state, payload schema 1
   and persisted NotRead. Replace the old-record upgrade acceptance test.
2. Remove unused aliases; use an explicit three-state FileObservation and
   payload schema 2. Validate sidecar/state restrictions and retain current
   archive escalation. Commit error and cache changes separately.
3. Update current design/specification and user cache guidance. Record five
   self-review passes per target, actual checks and independent review.
4. Run full workspace tests, strict Clippy/format/Rustdoc, focused shuffled
   tests, metadata fuzz smoke and documentation checks. Create a focused PR.

No formal decision semantics change: existing generated/proved adapters remain
in the full workspace checks; no new serialization proof is claimed.
