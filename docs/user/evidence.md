# Evidence and troubleshooting

The current lock reader supports uv.lock version 1 revision 5. Other revisions
fail explicitly. Poetry and other lock formats are unsupported. The check uses
recorded artifacts and does not discover unrecorded release files or determine
whether the lock is fresh relative to dependency requirements.

| Input | Supported interpretation |
| --- | --- |
| Wheel | ZIP Stored/Deflate with one regular root `.dist-info/METADATA` |
| Source archive | gzip/tar with one regular root-directory `PKG-INFO` |
| Core Metadata | 1.0–1.2 and 2.1–2.6; later major-2 minors warn and use known fields |
| Registry metadata | Simple API JSON and HTML; current and legacy hashed Core Metadata advertisements |
| Sources | HTTPS registry/direct artifacts and local distribution archives |

External Git repositories, directories, and editable sources remain explicit
incomplete targets. Metadata identity must match the locked normalized package
name and parsed version. Unsupported versions/formats, ambiguous members,
malformed identity, and hash mismatches yield exit 2.

`License-Expression` takes precedence. An invalid present expression cannot fall
back to `License`. When absent, a single legacy `License` value is interpreted
as a strict SPDX expression, a precise license name, or a complete recognized
license text. Official full names come from the pinned SPDX list; explicit
version/clause aliases include `Apache 2.0`, `BSD 2-Clause License`, `MPL 1.1`
and `MPL 2.0`. The Mozilla names `Mozilla Public License Version 1.1` and
`Mozilla Public License, Version 2.0` are also supported. The MPL names resolve to
MPL-1.1 and MPL-2.0 respectively and undergo the normal allow/deny policy check.
Additional reviewed versions include `Mozilla Public License v1.0` / `v1.1` /
`v2.0`, `ZPL 1.1` / `2.0` / `2.1`, `BSL 1.0` and `CDDL v1.0` / `v1.1`.
The full versioned names `Zope Public License Version 2.1`,
`Boost Software License Version 1.0` and `Eclipse Public License Version 2.0`
are supported as well. Versionless family names do not acquire a default version.
Name matching uses the whole value, trims outer whitespace and ignores ASCII
case; it does not collapse internal spaces or remove additional terms.
Other reviewed names include `The MIT License`, `Apache License (v2.0)`,
`Apache Software License 2.0`, `BSD 2 Clause`, `The BSD 3-Clause License`,
`CDDL 1.0`, `CDDL 1.1`, `EPL 1.0`, `EPL 2.0` and `CC0 1.0 Universal`.
GNU names such as `GNU General Public License v2 or later` and
`LGPL, v2.1 or later` preserve their explicit later-version permission.
This is a fixed set of reviewed spellings, not arbitrary punctuation removal
or fuzzy matching. `MPL`, `CDDL`, `EPL`, `GPL v2`, `LGPL 2.1`, unknown versions
and `MPL 2.0 with additional restrictions` remain Unknown.
Incomplete family labels can use verified bundled text under the narrow
rules below; added conditions
and other ambiguous names remain Unknown.
This name/text handling applies only to `License`: `License-Expression: MIT
License` is still invalid. Repeated values and unrecognized declarations need a
valid hash-bound clarification.
When both declaration fields are absent, or one eligible incomplete legacy value
is present, taloon can inspect bundled license files as described below.
Classifiers never supply a license expression.
This checks declarations; it does not establish that undeclared bundled software
or future built wheels have the same license.

Index sidecars require the exact locked artifact URL/digest and a verified
advertised metadata digest. Their evidence is index-associated, not archive
verified. Malformed JSON field types, null metadata advertisements, positional arrays in
place of object records and duplicate consumed fields invalidate an index response and cause archive fallback. Unknown index
extension fields are ignored. Sidecar failures fall back to full archive verification; integrity
failures remain warnings when that fallback succeeds. Clarifications and custom
permissions require the stronger proof. Stored observations retain metadata
headers and full-observation hashes, excluding Description bodies.

Inputs are bounded: 64 MiB lock, 16 MiB pyproject/index response, 4 MiB metadata,
256 MiB compressed archive, and 1 GiB expanded source tar. Oversized input is an
explicit error, never a skipped target. Archives are buffered and interpreted in
memory without filesystem extraction. UTF-8 metadata is required.

## Bundled license text

`check` and `describe` inspect `License-File` entries inside the hash-verified
distribution when both declaration fields are absent. They also complete a single
legacy `License` value equal to `UNKNOWN`, `BSD`, `BSD License`, `Apache`,
`Apache License`, `Apache Software License` or `Apache 2` (ignoring outer
whitespace and ASCII case). BSD labels require recognized BSD-2-Clause or
BSD-3-Clause text; Apache labels require Apache-2.0. A family mismatch stays
Unknown. `UNKNOWN` accepts any text recognized by the matcher. This supplies an
expression for the normal policy check; it does not automatically permit it.
A sidecar only triggers retrieval: the archive's own metadata must authorize
supplementation. With no entries, they try conventional LICENSE/LICENCE/COPYING
names (optionally `.txt`
or `.md`) in the source archive root or wheel metadata directory. Arbitrary
source files and external repositories are not searched.

The matcher recognizes complete MIT, BSD-2-Clause, BSD-3-Clause, ISC,
Apache-2.0 and MPL-2.0 texts. It accepts whitespace and a narrow set of
copyright/holder variants for the shorter licenses.
Apache requires every canonical mandatory term through `END OF TERMS AND
CONDITIONS`. Its application appendix can be absent or complete; within a
complete appendix, one structured copyright year/holder line can vary.
Partial appendices, changed terms and appended conditions remain Unknown.
MPL requires the entire canonical text, including both exhibits. Its exhibit's
historical HTTP locator is accepted alongside canonical HTTPS; neither URL is
fetched. BSD-3-Clause also accepts the reviewed plural-holder third-clause
wording while retaining every other term.
Structured variants include multiple copyright notices, year lists and selected
corporate/author-role spellings. Separate lines and sentences remain meaningful;
additional conditions cannot be discarded as copyright formatting.
The exact MIT title `The MIT License (MIT)` is supported. Shared attribution
rules also accept two comma-separated two-word personal names and a final
parenthesized web contact origin. A contact origin must use a literal HTTP(S)
origin spelling with no credentials, non-root path, query or fragment; URL
normalization cannot erase extra path material. This remains a narrow lexical
grammar, not an assertion about a holder's identity.
MIT also accepts an omitted preamble, known MIT titles with an optional single
Markdown heading marker, yearless structured notices, comma-separated dated
holder groups, spaced year ranges and a name-shaped alias followed by
`and contributors`. Corporate groups can include `and affiliates`.
Up to two bare contact domains may follow valid copyright notices; domains are
never fetched, and notices cannot resume afterward. These additional MIT forms
require the entire mandatory body unchanged and validate every preceding raw
line, with a 5,000-byte preamble limit, 32 notices and 32 dated groups per notice.
They do not relax other licenses' attribution rules or permit arbitrary text
before the grant.
This is template recognition, not fuzzy similarity or legal analysis. Unknown
variants, changed terms, appended text and differing license expressions across
files remain Unknown. Valid declarations take precedence. Any present
`License-Expression`, repeated legacy fields, additional terms and legacy values
outside the exact list above cannot be rescued by a permissive file.

AUTHORS, AUTHORS.txt and AUTHORS.md are classified by name as attribution files,
retained as observations and excluded from license matching. This classification
does not establish that those files contain no additional obligations. Every
other declared file must match the same supported expression. Missing, ambiguous,
nonregular, non-UTF-8 or oversized selected files leave the license unknown;
clarifications can still supply a reviewed expression. Files are limited to 32,
256 KiB each, 1 MiB total and 4,096 bytes per relative path. JSON `evidence.license_files` records relative paths,
file hashes, roles and matched expressions; original declaration fields remain
empty when absent. JSON `evidence.license_origin` identifies `license_expression`,
`legacy_declaration`, `bundled_files` or `legacy_supplement`, and is null when
the evidence cannot supply an expression. It describes evidence before policy
clarifications, which remain separately reported. Text output explicitly labels
successful legacy supplementation, including compliant artifacts. This recognition
does not establish licensing completeness
for bundled third-party code.

Precise legacy names and recognized text in metadata use the existing sidecar
and raw-metadata cache; interpreting them requires no extra archive download.
Valid metadata keeps its small sidecar fast path. Absent or eligible incomplete
legacy declarations trigger full archive verification at most once per cached observation, with the existing
bounded HTTP retry policy. The bundled canonical texts require no runtime
network database. Subsequent runs reuse observed file text, including completed
unknown scans; no archive is repeatedly fetched merely because recognition
failed. Cached older/sidecar observations needing files upgrade directly to an
archive, without refetching their index or sidecar. Offline performs no network
requests and retains Unknown when file evidence is unavailable. Normal cache
clearing, corruption recovery and eviction can require acquisition again.

## License problem causes

A license interpretation diagnostic keeps `code: unknown_license` and adds
`license_problem` in JSON. Acquisition, hash/identity mismatches and malformed
metadata retain their separate codes; they are not guessed to be license errors.

| `license_problem` | Meaning and next step |
| --- | --- |
| `missing_declaration` | No expression or legacy field and no file observation yet. Acquire available evidence online, or review an exact artifact for clarification. |
| `repeated_declaration` | The authoritative field occurs more than once. Review every occurrence; request corrected metadata or a reviewed clarification. |
| `invalid_expression` | The present `License-Expression` has unsupported syntax or identifiers. A legacy field cannot replace it. |
| `unrecognized_legacy` | The legacy value is not an accepted expression, name or full text. Review the original terms without discarding additional conditions. |
| `license_files_unavailable` | A completed file scan lacks usable selected files (missing, ambiguous, invalid, oversized or attribution-only). Inspect the paths/limits and exact artifact. |
| `unrecognized_license_text` | Selected text is outside supported complete templates. Review the full text. |
| `conflicting_license_files` | Selected files require different expressions. Review their relationship; do not choose one or invent AND/OR. |
| `legacy_family_mismatch` | Recognized bundled text disagrees with the incomplete legacy family. Review the disagreement. |

Text displays the corresponding cause and next step once per equivalent group.
Use `--verbose` or JSON to trace individual artifacts and full declaration values.
A clarification still needs a reviewed expression, reason and verified archive
hash as described in [configuration](configuration.md); these messages do not
approve a license or write configuration. File-read problems remain deliberately
one bounded category rather than claiming to identify an unobserved legal cause.

## HTTPS and credentials

Production acquisition uses HTTPS with certificate verification and rejects
HTTP redirects. It uses up to eight workers, 30-second request/10-second connect
limits, at most three attempts on transport failures/429/5xx, and five redirects.
URLs and signed queries retain fetching/matching semantics while reports redact
userinfo and query values. Userinfo does not supply HTTP Authorization.

`NETRC` selects an explicit UTF-8 credentials file; otherwise `~/.netrc` is used
when present. Explicit missing/unreadable/invalid files are configuration errors
online. Offline checks do not need credentials. Machine entries support login,
password, quoted values, comments, Unicode, and ignored account fields. Default
credentials are not sent; macros and unknown fields are rejected. Tokens can be
used as passwords with the index's required login. Keyrings and uv authentication
caches are not consulted. Credentials are never forwarded after a redirect to a
different origin, even if that destination has its own netrc entry.

For an incomplete result, inspect its machine diagnostic and artifact association,
then restore access or sufficient cache evidence. For unknown metadata, review
an exact artifact before adding a clarification and explicit permission. For a
violation, change the dependency or deliberately amend the policy. In deny mode,
explicit denials win over exceptions; a clarification still receives the same
policy evaluation. An empty denylist does not repair unknown declarations or
insufficient proof. Re-running
with a broader cache does not change the policy's meaning.

## Original license declarations

The JSON output of `check` and `describe` retains declaration prose in
`original_expression` and `original_license`, including ordinary email addresses
and question marks. URL tokens embedded in that prose have credentials, query
values and fragments removed; malformed or ambiguous adjacent URL tokens are
individually redacted. Embedded tabs and newlines remain part of URL inspection
so they cannot hide credentials.
This does not change which declarations count as valid SPDX permissions.
Text output escapes terminal controls; JSON encodes disruptive control and bidi
characters as escapes without changing the decoded declaration value.
