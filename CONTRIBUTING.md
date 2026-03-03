# Contributing to peek

Thank you for your interest in contributing. This document explains how to build, test, and submit changes.

## Development setup

- **Rust:** Stable toolchain (install from [rustup.rs](https://rustup.rs)).
- **Linux:** Full feature set (kernel, network, peekd). macOS/Windows get a subset (see README).

```bash
git clone https://github.com/ankittk/peek.git
cd peek
cargo build --workspace
```

## Running checks

Before submitting a PR, please ensure:

```bash
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Or use [just](https://github.com/casey/just): `just check` (format check + clippy + test), `just fmt`, `just lint`, `just test`. Run `just` with no args to list all commands. Optional: `just deny` runs `cargo deny check` (install with `cargo install cargo-deny`).

CI runs these on every push to `main` and on pull requests.

## Developer tooling

- **rustfmt.toml** — Formatting (max width 100, edition 2021).
- **clippy.toml** — Clippy config (e.g. cognitive complexity threshold).
- **.editorconfig** — Indentation/line endings for Rust, TOML, YAML, Markdown.
- **deny.toml** — `cargo deny` for license and advisory checks.
- **justfile** — Shorthand: `just test`, `just lint`, `just release`, `just man`, etc.

## Project layout

- `crates/peek-cli` — CLI and TUI binary `peek` (crate: `peek-process`)
- `crates/peek-core` — Core library: `ProcessInfo`, `collect()`, `collect_extended()`
- `crates/peekd` — Daemon for history and alerts
- `crates/proc-reader`, `kernel-explainer`, `resource-sampler`, `network-inspector`, `signal-engine`, `export-engine` — Helper libraries

See [docs/architecture.md](docs/architecture.md) for details.

## Publishing releases (maintainers)

Releases are created by pushing a version tag (e.g. `v1.0.0`). The [Release workflow](.github/workflows/release.yml) builds binaries and packages, creates a GitHub Release, and publishes crates to [crates.io](https://crates.io) in dependency order.

To enable crates.io publish, add a repository secret: **Settings → Secrets and variables → Actions → New repository secret** — name `CARGO_REGISTRY_TOKEN`, value = your [crates.io API token](https://crates.io/settings/tokens). The CLI is published as the **peek-process** crate (binary name remains `peek`).

## Submitting changes

1. Open an issue or comment on an existing one to discuss non-trivial changes.
2. Fork the repo, create a branch, and make your changes.
3. Run `cargo fmt`, `cargo clippy`, and `cargo test` as above.
4. Open a pull request with a clear description. Reference any related issues.

## Code style

- Follow `cargo fmt` and `cargo clippy` (we use `-D warnings` in CI).
- Prefer clear names and short functions. Add doc comments for public APIs.


## License

By contributing, you agree that your contributions will be licensed under the MIT License.
