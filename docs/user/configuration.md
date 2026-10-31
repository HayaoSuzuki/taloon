# License policy

The policy belongs in `[tool.taloon]` of the `pyproject.toml` next to `uv.lock`.
It applies to the entire workspace; member and user-wide policies are not merged.
Both `schema-version` and `allow` are required. An empty allowlist denies every
license. Unknown policy fields are errors.

```toml
[tool.taloon]
schema-version = 1
allow = ["MIT", "Apache-2.0", "Apache-2.0 WITH LLVM-exception"]
```

Each permission is one SPDX license or `WITH` unit. `AND` requires both operands;
`OR` requires one. GNU `-only` and `-or-later` identifiers remain distinct.
No compatibility rules or cargo-deny configuration are imported.
The pinned spdx 0.13.6 implementation uses SPDX License List 3.29.0.

## Scoped overrides

Exceptions add permissions for an exact normalized name, parsed PEP 440 version,
and source. A reason is required. Local versions remain distinct; this is not a
version range. The optional artifact hash restricts an exception to exact content.
The following complete example illustrates both override types. Replace the
placeholder hash and selectors with the reviewed artifact's actual values.

```toml
[tool.taloon]
schema-version = 1
allow = ["MIT", "Apache-2.0"]

[[tool.taloon.exceptions]]
name = "example"
version = "1.0"
source = { kind = "registry", location = "https://pypi.org/simple" }
allow = ["LicenseRef-Reviewed"]
artifact-hash = "sha256:0000000000000000000000000000000000000000000000000000000000000000"
reason = "Reviewed this exact artifact's custom declaration"
expires = "2099-12-31"

[[tool.taloon.clarifications]]
artifact-hash = "sha256:0000000000000000000000000000000000000000000000000000000000000000"
expression = "LicenseRef-Reviewed"
reason = "Reviewed declaration replacing missing or malformed metadata"
expires = "2099-12-31"
```

A clarification replaces the interpreted declaration; it still needs permission.
It cannot repair invalid archives, identity mismatches, unsupported sources, or
hash failures. Both clarifications and custom `LicenseRef-` permissions require
full archive verification. Global custom permissions are rejected. A custom
permission requires an exact hash-scoped exception, even after clarification.

Expiry is inclusive through the specified UTC date. Every expired entry fails
configuration validation, including entries matching no dependency. Reasons
cannot be blank. Reports preserve original declarations and applied reasons. Applied entries use
`exception[n]` identifiers (one-based array positions) or content-hash clarification
identifiers; reordering exception entries changes their reported positions.

Unused permissions and overrides warn only after complete inspection. All units
in an interpreted OR expression count as observed, even if another branch allows
it. Exceptions count as used when their selectors match a target; clarifications
count when their hashes match. `--deny-warnings` escalates complete-run warnings.
