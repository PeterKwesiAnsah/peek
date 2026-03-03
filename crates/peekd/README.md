# peekd

The **peekd** daemon: background service for resource history and alerting used by the peek CLI.

- **History** — Samples watched PIDs on an interval (e.g. 1s) into an in-memory ring buffer; the CLI requests history via `peek <pid> --history`.
- **Unix socket** — Listens at `/run/peekd/peekd.sock` (or configurable path). JSON request/response protocol for watch, unwatch, list, history, and alert actions.
- **Alerts** — Configurable rules (CPU, memory, FD count, thread count, etc.). Rules can be added by the CLI or loaded from `alerts.toml`; alerts are evaluated each sample and can trigger notifications (e.g. log or future hooks).
- **Optional persistence** — Per-PID history can be written to `$XDG_STATE_HOME/peekd/<pid>.jsonl` for reload after restart.

This crate is Linux-only. It depends on `peek-core` for `collect()`. For installation and systemd, see the repo root `packaging/` and `install.sh`.
