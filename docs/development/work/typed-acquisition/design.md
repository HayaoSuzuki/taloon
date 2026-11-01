# Typed acquisition inputs

Base `df637758fa6bb3349981641c68f63bf60f0950eb`, 2026-11-01. The owner explicitly
removed backward compatibility as a constraint and prioritizes implementation
simplicity, performance and meaningful types. The latest instruction was:
“後方互換性は考慮せず、実装や構造のシンプルさ、パフォーマンスを優先してください。” Extend open PR #10. Coverage,
content integrity and proof requirements remain product invariants.

## Decisions

Replace formatted worker keys with private Pinned(ContentHash, ArtifactFormat)
and Unpinned(RequestId) variants. Derive deterministic ordering directly on the
values, adding ordering to ArtifactFormat. Do not retain the old lexical/debug
string order. Preserve same-content/same-format affinity and every individual
request result. Empty scope needs no worker rather than an empty worker partition.

Decode JSON indexes into small private serde records for response version, file
URL, SHA-256 hashes and metadata advertisements. An advertisement Enum represents
boolean availability versus hash dictionaries; only an actual digest can support
sidecar evidence. Modern field presence takes precedence over its legacy alias,
including false. Reject malformed known field shapes, explicit null metadata and
duplicate known keys instead of reproducing Value's permissive skipping/last-win
behavior. Acquisition falls back to verifying the requested archive. Unknown
extension fields remain ignored. Every consumed record is wrapped in a private
object-only decoder: deriving a serde struct alone also admits positional arrays. Missing version still means 1.0.

The [Simple repository API](https://packaging.python.org/en/latest/specifications/simple-repository-api/)
was inspected 2026-11-01: metadata advertisements are boolean or hash mappings;
the legacy alias remains an external protocol option. Removing taloon API
compatibility does not justify removing valid server representations.

Choose owned typed fields over a custom streaming visitor or lifetime-heavy
borrowed document. They remove arbitrary Value graphs and unknown-field retention
without a new parser framework. The file vector/selected strings still consume
memory proportional to the response; no constant-memory or RSS claim is made.

## Assessment and verification

The inspected boundaries satisfy eight diagnostics (10/10), but formatted keys
and generic JSON trees hide avoidable interpretation/allocation work. Retain no
compatibility wrappers solely for old order/malformed inputs. No public API
redesign is needed beyond the useful ArtifactFormat ordering trait.

Measure ignored 1k/10k worker and JSON fixtures before/after. Use runtime RED for
empty-scope worker count and malformed JSON rejection, named AAA cases, and
256-case integer-group/advertisement properties. Existing Lean corpus adapters
must retain their current scope; no theorem about serde or worker scheduling is
claimed. Five self-reviews per stage, full stable/shuffled tests, existing core
fuzz/formal, quality/docs and a fresh whole-PR review precede commit/push/CI.
