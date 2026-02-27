## Contributing to `peek`

Thank you for your interest in improving `peek` — the Process Intelligence Tool for Linux.

### Development setup

- Install the Rust toolchain (stable) from `https://rustup.rs`.
- Clone the repository:

```bash
git clone https://github.com/ankittk/peek
cd peek
```

### Building and running

Workspace build (all crates):

```bash
cargo build --workspace
```

Run the CLI against a real PID:

```bash
cargo run -p peek-cli -- 1           # or any valid PID
cargo run -p peek-cli -- 1 --all
cargo run -p peek-cli -- 1 --watch
```

Run the daemon:

```bash
cargo run -p peekd
```

### Tests and checks

```bash
cargo fmt
cargo clippy --workspace -- -D warnings
cargo test --workspace
```

### Commit and PR guidelines

- Keep changes focused and self-contained.
- Add or update tests for any new behaviour (including integration tests in `crates/peek-core/tests/` where appropriate).
- Run `cargo fmt`, `cargo clippy --workspace -- -D warnings`, and `cargo test --workspace` before opening a PR.
- Write clear commit messages describing the motivation and behaviour change.

