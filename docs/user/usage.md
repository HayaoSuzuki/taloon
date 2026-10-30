# Commands and automation

```sh
taloon check
taloon check --project /path/to/project --format json
taloon check --offline --deny-warnings
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

JSON stdout contains exactly one schema-version-1 object, including startup
errors. Help/version requests print normal CLI help/version text.
`schema_version`, `status`, `exit_code`, `packages`, `diagnostics`, and
`acquisition` describe the run. Artifact records include identity associations,
original/interpreted licenses, proof strength, digests, and decisions. Status
records completeness/compliance; `exit_code` additionally reflects warning
escalation. Counter values measure requests, response-body bytes, archive
extraction attempts, and cache hits. Incidental output belongs on stderr.

## Cache and offline mode

Set `TALOON_CACHE_DIR` to isolate or relocate the evidence cache. Defaults use
`~/Library/Caches/taloon` on macOS, `XDG_CACHE_HOME/taloon` or `~/.cache/taloon`
on Unix, and `LOCALAPPDATA/taloon` on Windows. Windows paths are implemented but
have not been tested locally.

The cache stores observed declarations and provenance, never permission verdicts.
Policy is re-evaluated each run. Corrupt/incompatible records are misses; cache
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

Run `prek install` and `prek run taloon --all-files` (or the corresponding
pre-commit commands). The hook runner is an integration prerequisite; taloon
itself does not need it. `pass_filenames: false` and `always_run: true` ensure an
unchanged transitive dependency is still checked. CI invokes the same
`taloon check` command after installing from the checked-out source. Add `--offline`
only when every required observation is already available locally.
