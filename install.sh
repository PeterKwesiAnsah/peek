#!/usr/bin/env bash
set -euo pipefail

ARCH=$(uname -m)
VERSION="1.0.0"
URL="https://github.com/ankittk/peek/releases/download/v${VERSION}/peek-${ARCH}-unknown-linux-musl.tar.gz"

echo "Downloading peek ${VERSION} for ${ARCH}..."
curl -sSL "$URL" | tar -xz -C /usr/local/bin
echo "peek ${VERSION} installed to /usr/local/bin/peek"

