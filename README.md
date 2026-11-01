# taloon

Check declared licenses of Python dependencies in `uv.lock`, including development
and transitive dependencies. Implemented in Rust.

## Usage

Add a license policy to the `pyproject.toml` next to your `uv.lock`:

```toml
[tool.taloon]
schema-version = 1
allow = ["MIT", "Apache-2.0"]
```

After installing taloon, run from your project directory:

```sh
taloon check
taloon check --format json
```

Exit codes are `0` for compliance, `1` for a policy violation or escalated warning,
and `2` for invalid input or incomplete inspection.

## Documentation

- [User guide](docs/user/index.md): installation, supported platforms, Python
  versions, and release availability.
- [License policy](docs/user/configuration.md): allowlists, exceptions, and clarifications.
- [Commands and automation](docs/user/usage.md): CLI options, offline mode, and hooks.
- [Development guide](docs/development/index.md): building, testing, and contributing.
- [Design decisions](docs/development/knowledge/index.md): contracts and architecture.
- [Release procedure](docs/development/releases.md): wheel builds, publication, and recovery.

Licensed under the [Elastic License 2.0](LICENSE).
