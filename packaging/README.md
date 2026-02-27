This directory contains packaging metadata for various Linux distributions.

- `peek.spec`        — RPM spec file (Fedora/RHEL and derivatives)
- `debian/`          — Debian packaging files (`control`, `rules`, etc.)
- `PKGBUILD`         — Arch Linux PKGBUILD defining `peek` and `peekd` packages
- `peekd.service`    — systemd unit for the `peekd` daemon

These files are intended as a starting point for distro maintainers; please adapt
paths, dependencies, and policies to your distribution’s guidelines.

