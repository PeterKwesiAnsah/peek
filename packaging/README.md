# Packaging

This directory holds packaging metadata for Linux distributions. CI builds **.deb** (cargo-deb) and **.rpm** (cargo-generate-rpm) on each GitHub release and uploads them as release assets.

## Contents

| File | Use |
|------|-----|
| **peekd.service** | systemd unit for the peekd daemon. Installed by .deb/.rpm and by `install.sh` (optional). |
| **debian/** | Debian packaging (debhelper). For maintainers who build with `dpkg-buildpackage` or upload to a PPA. |
| **peek.spec** | RPM spec for Fedora/RHEL. For maintainers who build with `rpmbuild` or submit to Copr/EPEL. |
| **PKGBUILD** | Arch Linux. For AUR: upload to the AUR and users install with `yay -S peek` / `paru -S peek`. |

## One-line install (from GitHub Releases)

- **Script (any Linux):**  
  `curl -sSL https://raw.githubusercontent.com/ankittk/peek/main/install.sh | sudo bash`
- **.deb (Debian/Ubuntu):** Download `peek_1.0_amd64.deb` and `peekd_1.0_amd64.deb` (or 1.1, etc.) from [Releases](https://github.com/ankittk/peek/releases), then `sudo dpkg -i peek*.deb`.
- **.rpm (Fedora/RHEL):** Download the `.rpm` files from Releases, then `sudo rpm -ivh peek*.rpm` (or `dnf install ./peek*.rpm`).
- **Static binary:** Download `peek-v1.0-x86_64-linux-musl` and `peekd-v1.0-x86_64-linux-musl` (or `aarch64-linux-musl` on ARM) from the release for tag v1.0.0, put them in `PATH` as `peek` and `peekd`.

## PPA / AUR / Copr (maintainer-driven)

- **PPA (Ubuntu):** Use `debian/` and `dput` to upload to a PPA. Adapt `debian/control` (maintainer, dependencies) to your PPA.
- **AUR:** Use `PKGBUILD`; submit to the AUR so users can install with an AUR helper. Keep `pkgver`/`pkgrel` and source URL in sync with releases.
- **Copr / Fedora:** Use `peek.spec` with `fedpkg` or Copr; ensure `Source0` points to the release tarball.

These files are intended as a starting point; adapt paths, dependencies, and policies to your distribution’s guidelines.
