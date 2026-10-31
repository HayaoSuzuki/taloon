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
`taloon` needs neither Python nor uv. Binary releases, PyPI packages, publishing,
and deployment are deferred.

Add a [policy](configuration.md), then run from your project or a subdirectory:

```sh
taloon check
```

See [commands and automation](usage.md) for JSON, offline checks, and hooks, and
[evidence and troubleshooting](evidence.md) for supported inputs and trust limits.
For contributor procedures, follow the repository's
[development entry point](https://github.com/HayaoSuzuki/taloon/blob/main/docs/development/index.md).
