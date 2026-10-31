# Distribution implementation evidence

## Stage 1: binary packaging

At the working tree based on `55b6134`, the metadata regression first failed
because the project was `taloon-development`. After configuration it passed.
Maturin 1.15.0 built `taloon-0.1.0-py3-none-macosx_11_0_arm64.whl` locally.
The actual uv wheel smoke passed on Python 3.14 and 3.15 using the same file.
All fixtures are synthetic. Local evidence does not establish other platforms
or a public-index upload.

Five self-review passes:

1. Checked metadata ownership: Cargo alone supplies version; Python declares
   dynamic version and an exact `>=3.14,<3.16` support range, with no dependencies.
2. Checked executable packaging: bin bindings produce a platform wheel, with
   macOS 11 arm64 tag; no Python wrapper or extension-module ABI was introduced.
3. Exercised actual uv installation for both supported interpreters with isolated
   tools, preventing an existing installed taloon from satisfying the invocation.
4. Exercised paths containing spaces and cold/warm cache, then changed policy
   without deleting evidence: exit 0 changes to 1; missing local bytes yield 2.
5. Reviewed developer/runtime separation and truthful guidance: docs dependencies
   remain a separate group; the guide does not claim a PyPI release exists.

The documentation checker and 29 Python tooling tests passed. Rust source was
unchanged; the baseline Rust suite passed before packaging changes.
