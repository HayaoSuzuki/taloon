# taloon

taloon checks declared dependency licenses in `uv.lock`, using a policy in the
sibling `pyproject.toml`. It is implemented in Rust and uses the
[Elastic License 2.0](https://github.com/HayaoSuzuki/taloon/blob/main/LICENSE).

Install from a checkout with Rust 1.99.1 and a native C build toolchain:

```sh
cargo install --locked --path .
```

The installation was tested on macOS arm64 with the pinned compiler and Apple
command-line build tools. Cargo downloads and compiles dependencies; an installed
`taloon` needs neither Python nor uv. Python wheels are built and checked on PRs;
main publication requires the PyPI/TestPyPI publisher setup. Until the first
verified PyPI release, install from a checkout as above. Contributors can build a local
binary wheel with `uvx --from maturin==1.15.0 maturin build --release --locked`
and run `uv tool run --from /absolute/path/to/taloon-VERSION-PLATFORM.whl taloon check`.
Python installation support is restricted to 3.14 and 3.15; the wheel contains
the Rust executable and has no Python runtime package dependencies.

After the first PyPI release, run or install the command with a supported Python:

```sh
uv tool run --python 3.14 taloon check
uv tool install --python 3.15 taloon
```

Prebuilt wheels target Windows x86_64, Linux x86_64 with glibc 2.17 or later,
and macOS 11 or later on Apple Silicon. Other architectures and musl Linux do
not have a published wheel in this release configuration. Python 3.13 and older,
and Python 3.16 or later, are excluded by package metadata. Each platform uses
the same wheel for Python 3.14 and 3.15.

Add a [policy](configuration.md), then run from your project or a subdirectory:

```sh
taloon check
```

See [commands and automation](usage.md) for JSON, offline checks, and hooks, and
[evidence and troubleshooting](evidence.md) for supported inputs and trust limits.
For contributor procedures, follow the repository's
[development entry point](https://github.com/HayaoSuzuki/taloon/blob/main/docs/development/index.md).
