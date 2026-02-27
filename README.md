# peek

The Process Intelligence Tool.

`peek` is a single, unified CLI + TUI that explains what a process is, what it's doing, how it's consuming system resources, and what it’s connected to — in plain, readable terminal output.

This repository contains the full Rust implementation:

- a core library that gathers process info (on Linux via `/proc`, on macOS/Windows via `sysinfo`)
- a CLI (`peek`) with JSON/Markdown/HTML/PDF export and an interactive TUI
- a background daemon (`peekd`) for history and alerting over a Unix socket (Linux/Unix only)

### Platform support

| Feature | Linux (all distros) | macOS | Windows |
|--------|----------------------|-------|--------|
| Inspect process (PID, name, memory, CPU, exe) | ✅ Full | ✅ Basic | ✅ Basic |
| Kernel context (cgroups, OOM, namespaces, seccomp) | ✅ | — | — |
| Network (listening ports, connections) | ✅ | — | — |
| Open files, env vars, process tree | ✅ | — | — |
| GPU (NVIDIA/AMD) | ✅ | — | — |
| Port search (`--port`) | ✅ | — | — |
| Kill/signal panel (`--kill`) | ✅ | — | — |
| History & alerts (`peekd`, `--history`) | ✅ | — | — |
| TUI (`--watch`), export (JSON/HTML/MD/PDF) | ✅ | ✅ | ✅ |

**Linux** includes Debian, Ubuntu, Fedora, RHEL, Arch, and any other distro. Full functionality (kernel, network, cgroups, port search, kill panel, `peekd`) is available only on Linux. **macOS** and **Windows** get baseline process info (name, PID, memory, CPU, executable path, state) and TUI/export; advanced features are disabled with a clear message.

### Installation

#### macOS (Homebrew)

```bash
brew install peek
```

On macOS, `peek` provides basic process inspection (PID, name, memory, CPU, exe). For full features use Linux.

#### Linux (Debian / Ubuntu)

```bash
# From a .deb (when published) or a PPA
sudo apt update
sudo apt install peek peekd
```

#### Other Linux

- **Arch:** install from AUR (`peek`, `peekd`) or use the `packaging/PKGBUILD`.
- **Fedora / RHEL:** use the RPM from releases or `packaging/peek.spec` to build.
- **Generic:** download the static binary for your arch from [GitHub Releases](https://github.com/ankittk/peek/releases) and place `peek` and `peekd` in your `PATH` (e.g. `~/.local/bin` or `/usr/local/bin`).

#### From source (Linux, macOS, Windows)

If your distro or OS isn’t covered or you want to contribute, build and install locally:

```bash
cargo build --release --workspace
mkdir -p ~/.local/bin
cp target/release/peek  ~/.local/bin/peek
# peekd is Linux/Unix only; on Windows the build produces a stub that exits with a message
cp target/release/peekd ~/.local/bin/peekd   # optional, Linux/Unix
```

Ensure `~/.local/bin` is on your `PATH`. On Windows you can use `target\release\peek.exe`. For system-wide install and the optional `peekd` systemd unit on Linux, see `packaging/peekd.service` and your distro’s guidelines.

### Usage

#### Basic CLI

```bash
# Inspect a process by PID or name
peek 1234
peek nginx

# If you need root-only details or to signal root-owned processes,
# prefer the built-in sudo relaunch instead of prefixing with sudo:
peek 1234 --kill --sudo

# Show all sections (resources, kernel, network, files, env, tree, GPU)
peek 1234 --all

# JSON for scripting
peek 1234 --all --json
```

#### Live TUI

```bash
peek 1234 --all --watch          # interactive dashboard
```

The TUI shows CPU/RSS/FD sparklines, gauges, and tabs for kernel context, network, files, env, and process tree.

#### History and alerts with `peekd`

```bash
# Run the daemon (or use the systemd unit)
peekd &

# Register and query history for a PID
peek 1234 --history
```

The CLI talks to `peekd` over `/run/peekd/peekd.sock` to fetch a ring-buffer of recent samples and to manage alert rules (CPU%, memory MB, FD count, thread count).

#### Exporting reports

```bash
peek 1234 --all --export md      # Markdown
peek 1234 --all --export html    # HTML
peek 1234 --all --export pdf     # PDF (requires wkhtmltopdf/weasyprint/Chromium)
```

### Project layout

- `Cargo.toml`               — workspace definition and shared dependencies
- `crates/peek-core`        — core library (`ProcessInfo`, `/proc` readers, GPU, signal impact, ring buffer)
- `crates/peek-cli`         — CLI + TUI (`peek`) and `peekd` client
- `crates/peekd`            — daemon (`peekd`) for history + alerts over a Unix socket
- `packaging/`              — distro packaging metadata (RPM, Debian, Arch, systemd unit)
- `completions/`            — generated shell completions (written by `peek-cli`’s `build.rs`)
- `.github/workflows/`      — CI and release workflows
- `install.sh`              — simple installer script for future binary releases

### Contributing

See `CONTRIBUTING.md` for development setup, testing, and PR guidelines.
