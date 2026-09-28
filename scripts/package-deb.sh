#!/usr/bin/env bash
# Packages a built Overstay Linux binary into a minimal .deb for playtesting.
# Usage: package-deb.sh <version> <binary_path> <output_deb_path>
set -euo pipefail

VERSION="$1"
BINARY="$2"
OUT="$3"

WORKDIR="$(mktemp -d)"
PKGDIR="$WORKDIR/overstay"

mkdir -p "$PKGDIR/DEBIAN" "$PKGDIR/opt/overstay" "$PKGDIR/usr/bin"

cp "$BINARY" "$PKGDIR/opt/overstay/overstay"
chmod 755 "$PKGDIR/opt/overstay/overstay"

cat >"$PKGDIR/usr/bin/overstay" <<'LAUNCHER'
#!/bin/sh
exec /opt/overstay/overstay "$@"
LAUNCHER
chmod 755 "$PKGDIR/usr/bin/overstay"

cat >"$PKGDIR/DEBIAN/control" <<CONTROL
Package: overstay
Version: $VERSION
Section: games
Priority: optional
Architecture: arm64
Maintainer: ZN-Black-Nexus <noreply@znblacknexus.invalid>
Description: Overstay (debug build)
 Co-op/solo 3D horror game. Debug playtest build, not for release.
CONTROL

mkdir -p "$(dirname "$OUT")"
dpkg-deb --build --root-owner-group "$PKGDIR" "$OUT"
rm -rf "$WORKDIR"
