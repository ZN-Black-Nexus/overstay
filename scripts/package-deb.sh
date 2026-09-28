#!/usr/bin/env bash
# Packages a built Overstay Linux export directory into a minimal .deb for
# playtesting. The whole export directory is required, not just the
# executable: GDExtension shared libraries are exported as sibling files
# (e.g. bin/linux-arm64/liboverstay.so) since they can't live inside the
# embedded .pck — the OS loader needs them as real files on disk.
# Usage: package-deb.sh <version> <export_dir> <output_deb_path>
set -euo pipefail

VERSION="$1"
EXPORT_DIR="$2"
OUT="$3"

WORKDIR="$(mktemp -d)"
PKGDIR="$WORKDIR/overstay"

mkdir -p "$PKGDIR/DEBIAN" "$PKGDIR/opt/overstay" "$PKGDIR/usr/bin"

cp -r "$EXPORT_DIR"/. "$PKGDIR/opt/overstay/"
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
