# Legacy interpretation and structured notice review

The owner selected further precise-name, metadata-text and structured-attribution
support. The [plan](plan.md) records scope. This extends the former whole-legacy-
value SPDX-only rule; it does not relax License-Expression or introduce fallback
from present fields to different files. Fixtures and expectations use synthetic
identities and the pinned public SPDX catalog/texts.

## Stage 1: precise legacy names

Names use the existing SPDX result type. Strict expression parsing remains first;
whole-name ASCII-case-insensitive lookup then considers official catalog names
and explicit version/clause aliases. Different identities under the same name
are rejected. No imprecise-name helper, substring matching or generic BSD/Apache
alias is enabled. Lookup trims once before iterating the catalog. Original fields,
policy permission evaluation and source proof remain unchanged.

Eleven initial name/property cases failed before implementation, then passed.
Named ambiguity/extra-term/valid-expression cases and generated case/whole-value
properties retain the conservative boundary. Four synthetic catalog-collision
cases check unique identity independently of current upstream contents.

Five passes: (1) strict-vs-legacy field ownership; (2) whole-name/identity semantics;
(3) no I/O or cached-verdict state, one trim; (4) independent named/generated
negative/positive outcomes; (5) pinned catalog attribution and recheck conditions.

## Stage 2: complete legacy texts

The legacy owner reuses the existing complete-template matcher for the same field.
Raw metadata observations are reinterpreted, retaining sidecar/archive strength.
No cache version, stored checksum inputs or transport behavior changes. HTTP
fixtures check no archive escalation for recognized legacy names/text and zero
requests/bytes when restoring cached names online or offline.

Five complete-text cases and one cache case failed before connection, then passed.
Explicit-expression priority, expression-as-name rejection, repeated values and
added terms remain separate public cases. The final public file has 43 cases,
including generated whole-value, appended-term and Unicode/cache properties.

An independent reviewer found one Important issue: using the unfolded legacy value
for text matching merged separate restrictive lines into a holder field, and also
lost valid multiple notices. Eight public LF/CRLF by archive/sidecar cases all
failed before correction. Metadata parsing now retains the selected raw value's
byte range in the existing immutable headers. The matcher uses that raw slice;
expressions/names/reporting keep the unfolded value. Unicode-generated offsets
and serialized-record replay retain the boundary without duplicate owned text or
a second header parser. Unavailable slices cannot trigger text recognition.

Five passes: (1) complete-field scope and priority/cardinality; (2) actual public
constructor/rebind/cache calls; (3) bounded raw spans and no serialization or
proof promotion; (4) eight red/green boundary cases plus independent generated
checks; (5) preserved raw-vs-unfolded abstraction and documentation limits.

## Stage 3: structured notices

At most 32 separate copyright notices and 32 comma-separated four-digit year/range
entries per line are supported, with specific corporate comma suffixes and
lowercase role/connector words. At least one name token is required. Raw line
and sentence boundaries remain meaningful; arbitrary added lines/terms are not
removed. Canonical text initialization remains process-local and lazy; no extra
production storage or dynamic template engine is added.

The initial expanded run had 47 passing and 16 failing cases. One negative
expectation was removed because it conflicted with previously accepted terminal
name punctuation; the feature was not weakened to satisfy it. The final matcher
module passes 74 cases including new independent generated structures and earlier
sentence/line attacks.

Five passes: (1) complete-body/API invariants; (2) original-line framing;
(3) bounded notice/year loops and allocation-free grammar; (4) independent
positive/adversarial cases and existing regressions; (5) synthetic/public sources
and explicit limitations. Name-shaped holder content remains syntactically
ambiguous, date chronology is not validated, and unlisted formatting stays
unknown. None of this proves legal equivalence or actual authorship.

## Integration and review boundary

The fresh reviewer found no additional Critical/Important issue in catalog
collisions, strict expression precedence, cache reinterpretation or resource
bounds. The executor corrected the raw-boundary finding through red/green public
regressions and the full suite; no second independent approval is claimed. The
review did not certify unfinished documentation or legal equivalence.

Design assessment: 10/10 for the scoped structural diagnostics. Each owner has a
single purpose, the legacy interface hides the expanded formats, raw framing is
metadata-owned, and callers gain no matching or transport policy parameters.
The initial mixed interpretation decision is now isolated. This assessment is
about complexity and ownership, not a correctness proof.

Existing Lean/Creusot models and trust boundaries remain unchanged; there is no
new proof of metadata parsing, matching or licensing completeness. New metadata
fuzz seeds contain precise names, a canonical complete text and a synthetic added
condition. They derive only from public SPDX text/synthetic values. Reproduction
commands remain in the developer guide and the plan links the maintained scope.

Final local integrated verification on macOS arm64: 621 stable Rust tests and
621 nightly randomized-order tests pass. Strict Clippy, formatting, rustdoc,
repository documentation/OKF checks and strict user-site build pass. Lean builds
and all three corpus freshness checks pass. Metadata, wheel and sdist fuzz each
complete 1,000 bounded executions without crashes. Existing ignored fixtures and
benchmarks remain ignored. Python source, dependency graph and workflow definitions
are unchanged. Hosted checks are observed separately on PR #20.
