# peek-process

The **peek** binary: CLI and TUI for process inspection on Linux (and limited support on macOS/Windows).

- **Inspect by PID or name** — `peek 1234`, `peek nginx`
- **Sections** — Resources, kernel context, network, open files, env, process tree, GPU (Linux)
- **Live TUI** — `--watch` for a live-updating dashboard with sparklines
- **Export** — JSON, Markdown, HTML, PDF (via `export-engine`)
- **Port search & kill panel** — `--port 443`, `--kill` with impact analysis (Linux)
- **History & alerts** — Talks to **peekd** over a Unix socket when the daemon is running

This crate contains the main entrypoint (`main.rs`), argument parsing (`args.rs`), and the TUI modules under `ui/`. It depends on `peek-core` for data collection and `export-engine` for formatted output.
