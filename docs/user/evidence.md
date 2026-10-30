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
back to `License`. When absent, a complete legacy License field is accepted only
if it parses as SPDX. Missing, repeated, unknown, or prose-only declarations are
incomplete unless a valid hash-bound clarification supplies the expression.
Classifiers, license files, and full-text guessing do not grant permission.
This checks declarations; it does not establish that undeclared bundled software
or future built wheels have the same license.

Index sidecars require the exact locked artifact URL/digest and a verified
advertised metadata digest. Their evidence is index-associated, not archive
verified. Sidecar failures fall back to full archive verification; integrity
failures remain warnings when that fallback succeeds. Clarifications and custom
permissions require the stronger proof. Stored observations retain metadata
headers and full-observation hashes, excluding Description bodies.

Inputs are bounded: 64 MiB lock, 16 MiB pyproject/index response, 4 MiB metadata,
256 MiB compressed archive, and 1 GiB expanded source tar. Oversized input is an
explicit error, never a skipped target. Archives are buffered and interpreted in
memory without filesystem extraction. UTF-8 metadata is required.

## HTTPS and credentials

Production acquisition uses HTTPS with certificate verification and rejects
HTTP redirects. It uses eight workers, 30-second request/10-second connect
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
violation, change the dependency or deliberately amend the policy. Re-running
with a broader cache does not change the policy's meaning.
