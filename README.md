# taloon

License checking for uv-resolved Python dependencies, implemented in Rust.

Run `taloon check` with a license policy in `pyproject.toml` to inspect all
external locked records and recorded artifacts. The core separates metadata and
policy processing from file/network acquisition. Install from this checkout
with `cargo install --locked --path .`; publishing remains deferred.

Read [the user guide](docs/user/index.md) for availability and usage.
Developers and AI agents should start with [the development guide](docs/development/index.md)
and its [OKF catalog](docs/development/knowledge/index.md).

Licensed under the [Elastic License 2.0](LICENSE).
