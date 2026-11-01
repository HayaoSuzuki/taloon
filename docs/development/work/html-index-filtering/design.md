# Filter HTML index entries during tokenization

Base `56fff43cdec130da66510a4f1bb5f135efbf9544`, 2026-11-01. The owner's
repeated software-design assessment authorizes internal improvements preserving
existing coverage, proof, CLI and cache contracts. Extend still-open PR #10.

The HTML parser currently retains every anchor/base/meta tag and owned attribute
map until tokenization finishes, then keeps only locked artifact URLs. A large
historical index therefore requires a second complete tag representation even
though acquisition needs few entries. Existing module boundaries satisfy eight
scoped diagnostics (10/10); that checklist does not establish allocation quality.

## Decision and alternatives

Move HTML association interpretation into its private TokenSink. Retain only
locked URL bindings plus the selected base URL and terminal parse outcome. A
Result keeps failure absorbing rather than combining independent error/data
flags. Option<Url> records whether an explicit base was selected. Existing Proof
and Binding Enums remain appropriate; no new public type, generic streaming
framework or per-tag Enum is required.

The sink preserves raw-text/RCDATA handling, the first base with href, sequential
base application, unsupported repository-version rejection even after a matching
link, modern attribute precedence and ambiguous duplicate target rejection.
Borrow tokenizer-owned attributes directly, using reverse local-name lookup to
preserve the former map's last-value rule. No duplicate attribute map is needed.
No early success terminates parsing. No networking, source normalization, JSON
parsing or evidence-strength rule changes.

Retaining all tags is simpler but duplicates historical index storage. A generic
event/callback abstraction adds protocol without another consumer. Direct private
sink interpretation is selected. JSON streaming is separate work: replacing its
Value semantics requires its own compatibility and resource evidence.

## Limits

The tokenizer/current tag and bounded HTTP body remain allocated. Retained output
is proportional to locked URLs, not historical anchors, but this is not a measured
peak-RSS guarantee. Release timing samples must distinguish structural storage
removal from observed speed. No user documentation changes are needed.
