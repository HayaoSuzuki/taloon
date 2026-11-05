# Commands and automation

```sh
taloon check
taloon check --project /path/to/project --format json
taloon check --offline --deny-warnings
taloon check --verbose
```

Without `--project`, taloon walks upward from the current directory to the nearest
`uv.lock` and requires its sibling `pyproject.toml`. An explicit project is an
exact directory; missing files there do not trigger an upward fallback.

All external locked records are checked, including development, transitive,
extra, group, and platform branches. Every recorded wheel and source archive is
a target. Only the workspace/project packages themselves are excluded. Filenames,
the current environment, and selected dependency groups do not narrow the check.
The lock is never re-resolved, refreshed, or modified. Packages and build backends
are never installed or executed.

| Exit | Meaning |
| --- | --- |
| 0 | Complete and compliant, with no escalated warnings |
| 1 | Complete with a violation or escalated warning |
| 2 | Invalid input/configuration or incomplete inspection |

Exit 2 takes precedence in mixed outcomes. Recoverable acquisition failures are
aggregated. Failure paths show a shortest known route per entry point, with
recorded group/extra/marker conditions; absent origins are explained rather than
invented. Conditions describe locked alternatives, not environment evaluation.

Default text groups artifacts within each package when their displayed outcome,
redacted original declarations, diagnostics, proof strength, license origin and policy
units/overrides agree. It shows a count, one representative locator, declaration
previews and cause-specific next steps. Different full redacted declarations stay separate
even when their 200-character previews look identical. Compliant packages with
applied overrides remain visible so their review basis can be inspected.

Use `--verbose` for each artifact's locator, observed archive/metadata hashes,
selected file summaries and full original declarations. JSON always contains
every artifact; `--verbose` does not change it. Grouping affects presentation
only: every artifact is still acquired/evaluated and contributes to the exit code.
The [evidence guide](evidence.md#license-problem-causes) explains typed causes.

For `check --format json`, stdout contains exactly one schema-version-1 object,
including startup errors. `policy_mode` identifies `allow` or `deny` after valid
policy loading, including empty or incomplete inspections; startup errors before
policy loading omit it. Each artifact permission decision also includes `mode`.
Text output identifies the selected mode. Reports do not contain the configured
allow/deny arrays. Optional `license_problem` on license diagnostics is
an additive schema-1 field with a snake_case cause; other errors omit it. Existing
fields remain, wording is not a machine contract, and incompatible report layouts
require a new schema version. Help/version requests print normal CLI help/version text.
When stderr is an interactive terminal, taloon shows processed/total artifact
counts and elapsed time during acquisition, including while waiting for a
response. Processed counts include failures; the final report determines the
result. Progress is omitted when stderr is redirected and never enters stdout.
`schema_version`, `status`, `exit_code`, `packages`, `diagnostics`, and
`acquisition` describe the run. Package records include the source kind and redacted location.
Artifact records include identity associations,
original/interpreted licenses, proof strength, digests, and decisions. Status
records completeness/compliance; `exit_code` additionally reflects warning
escalation. Counter values measure requests, response-body bytes, archive
extraction attempts, and cache hits. Incidental output belongs on stderr.

The acquisition summary also separates HTTP work into `http.index`,
`http.metadata`, and `http.archive`. Each contains actual `requests`,
`downloaded_bytes`, `request_time_us` through body receipt, and `retry_delay_us`.
Attempts include retries, redirects, and failed partial responses. Durations
accumulate across workers and can overlap; they are not wall elapsed time.
`elapsed_us` measures acquisition wall time, excluding input discovery and final
policy/report rendering. `index_parse_count`/`index_parse_us` measure parsing;
`index_cache_hits`/`index_wait_us` describe in-run reuse and coordination waiting.
All timing values use integer microseconds and do not decide compliance. Ready
index reuse adds no waiting time; initialization coordination excludes the
initializing worker's own fetch and parse time.

During one check, artifacts from the same package index endpoint share one
fetched and parsed index, including a terminal fetch failure after bounded
retries. Every artifact still checks its exact URL and locked archive hash. A
failed lookup falls back independently to archive verification. Index snapshots
are kept only in memory for that check; the next check fetches afresh when
cached artifact evidence is insufficient.

## Describe package licenses

```sh
taloon describe requests
taloon describe requests --offline
taloon describe requests --verbose
taloon describe requests --project /path/to/project --format json
```

`describe` reads the selected `uv.lock` and displays the named package and its
transitive dependency licenses. It uses the same upward lock discovery and
exact `--project` directory selection as `check`. It does not require or read
`pyproject.toml` or `[tool.taloon]`: allowlists, denylists, exceptions, clarifications,
custom-license definitions and their expiry do not alter these observations.
It reports declared licenses without granting permission.

Names use Python normalization (case and runs of `-`, `_`, `.`). Every matching
locked version/source is included and identified separately. For an external
package, traversal includes normal dependencies and every recorded extra at
every level, across all locked environment branches. Development edges are
excluded in this mode. Extra names, requested extras, edge markers and package
resolution markers remain annotations; the output describes locked alternatives,
not just the current environment or simultaneously active dependencies.

Naming a project or workspace member displays **all external records**, exactly
the inventory used by `check`, including development, transitive, unsupported
and unreachable records. Own/workspace license observations are excluded. If a
name matches both workspace and external identities, this whole-inventory mode
wins. Workspace nodes can still appear as dependency-routing context, with their
own license explicitly excluded. Missing package names fail; no index search or
dependency re-resolution is performed.

Each selected package is listed once, with its version, redacted source,
license groups and annotated dependency relationships. Cycles/shared dependencies
retain their edges without recursive expansion. Every recorded wheel/sdist is
an evidence target. Equal validated expression text is grouped with artifact
counts; different declarations are listed separately and never joined into an
invented AND/OR expression. Unknown/invalid declarations and unsupported targets
remain visible, alongside available observations. Problem groups use the same
summary/`--verbose` presentation as check. A syntactically valid
LicenseRef is displayed as declared without supplying its definition or permission.

Evidence verification, provenance, acquisition limits, credentials, progress and
the persisted evidence cache are shared with `check`. Only selected artifacts
are acquired. `--offline` prohibits network access; missing or insufficient
cached evidence can make a description incomplete. A record acquired by either
command can be reused by the other when its identity and proof are sufficient.

Exit 0 means complete valid declarations for all selected external targets;
exit 2 means an input error or incomplete description. Different valid licenses
alone are not an error, and there is no permission-violation exit 1. Warnings
remain warnings when declarations are usable. Partial observations are printed
with an incomplete result. An empty project inventory can complete successfully.

`--format json` emits one schema-version-1 describe object, including startup
errors, with `command: "describe"`, `query`, `roots`, `selection_mode`,
`status` (`complete`/`incomplete`), `packages`, annotated `graph`, `diagnostics`,
`acquisition` and `exit_code`. Each license group contains its exact expression
and artifact IDs; every artifact retains its observed metadata/proof and
failure diagnostics. This schema is separate from check's permission report.
Progress is confined to interactive stderr. `describe` does not accept
`--deny-warnings`, version/source filters or extra/environment selectors.

## Cache and offline mode

Set `TALOON_CACHE_DIR` to isolate or relocate the evidence cache. Defaults use
`~/Library/Caches/taloon` on macOS, `XDG_CACHE_HOME/taloon` or `~/.cache/taloon`
on Unix, and `LOCALAPPDATA/taloon` on Windows. Windows paths are implemented but
have not been tested locally.

Clear taloon's cached evidence with:

```sh
taloon cache clear
```

This works outside a project and uses the same location as `check`, including
`TALOON_CACHE_DIR`. The cache is shared across projects using that location.
The command removes recognized completed cache records, including obsolete or damaged
records, and prints the number removed. It retains the directory, unrelated or
unknown filenames, temporary files, subdirectories and symlinks. It does not
follow a cache-root symlink; select its actual directory instead. An absent
cache succeeds with zero removed. Success exits 0; an I/O failure exits 2 with
a stderr diagnostic and may leave partial progress.

Stop other taloon checks before clearing; active writers can repopulate the
cache. The next online check reacquires missing evidence, and offline checks
can become incomplete. An empty `TALOON_CACHE_DIR` selects the current directory,
matching existing cache writes; prefer an explicit dedicated directory.

Evidence payloads from the previous format are no longer accepted or migrated.
The first online run after upgrading reacquires required evidence. An offline
run needs current-format evidence or available local artifacts; old cache files
alone cannot satisfy it. `taloon cache clear` can remove the obsolete files.

The cache stores observed declarations and provenance, never permission verdicts.
The first online check may take substantially longer than a cached check because
it acquires evidence for every recorded artifact across all locked environments.
`check` re-evaluates policy each run. Corrupt/incompatible records are misses; cache
read/write failures warn while available evidence remains usable. Offline mode
makes no network calls and retains the full target scope. Missing or insufficient
cached proof yields exit 2. A new clarification can require archive acquisition
online and fail incomplete offline when only sidecar evidence exists.

Local archives are read and hashed again each run, including warm-cache checks.
Reports distinguish a lock-pinned hash from the freshly observed digest of a
local archive without one. Cache records are trusted local output with corruption
checks, not signed evidence supplied by arbitrary third parties.

## Local hook and CI

Install taloon first and keep it on PATH. In the inspected Python project's
`.pre-commit-config.yaml`, use this local hook:

```yaml
repos:
  - repo: local
    hooks:
      - id: taloon
        name: Check uv.lock dependency licenses
        entry: taloon check
        language: system
        pass_filenames: false
        always_run: true
```

Run `prek install` and `prek run taloon --all-files`. The integration was tested
with prek 0.5.7. The hook runner is an integration prerequisite; taloon
itself does not need it. `pass_filenames: false` and `always_run: true` ensure an
unchanged transitive dependency is still checked. CI invokes the same
`taloon check` command after installing from the checked-out source. Add `--offline`
only when every required observation is already available locally.
