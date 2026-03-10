# Changelog

## v1.1.0

- New **Port Monitor TUI**: `peek --listen` shows a live, auto-refreshing table of all listening TCP/UDP sockets (protocol, port, address, process name, PID).
- Supports **auto-refresh every 3s**, manual refresh with `r`, sortable columns (`s` cycles port/process/protocol/PID), row selection with `j/k` or arrow keys, and **SIGTERM** via `K` to kill the selected process.
- Styled header, selected row highlight, and help bar; handles missing PIDs and permission errors gracefully while scanning `/proc`.

## v1.0.0

Initial public release.

- Single-process deep inspection: identity, resources, kernel context, network, files, env, tree
- Live TUI with sparklines (`--watch`)
- Export to JSON, Markdown, HTML, PDF
- Signal/kill panel with impact analysis and systemd awareness
- Port search (`--port`)
- `peekd` daemon for resource history and threshold alerts
- Packaging: static musl binaries, .deb, .rpm, AUR PKGBUILD, one-line install script
- Shell completions (bash, zsh, fish) and man pages
