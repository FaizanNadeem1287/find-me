#!/usr/bin/env bash
# ==============================================================================
# Find-Me Debian (.deb) Release Package Builder
# ==============================================================================
# Automates compiling the release binary, validating FreeDesktop standards,
# and producing a production-ready Debian package for Ubuntu and Debian systems.
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

cd "${REPO_ROOT}"

echo "=========================================================="
echo "  Find-Me Debian Release Packaging"
echo "=========================================================="

# 1. Validate FreeDesktop Assets
echo "--> [1/4] Validating FreeDesktop Standards..."
if command -v desktop-file-validate >/dev/null 2>&1; then
    desktop-file-validate assets/find-me.desktop
    echo "    [PASS] assets/find-me.desktop validated successfully."
else
    echo "    [WARN] desktop-file-validate not found; skipping."
fi

if command -v appstreamcli >/dev/null 2>&1; then
    appstreamcli validate --no-net assets/io.github.faizan_nadeem.find_me.metainfo.xml
    echo "    [PASS] AppStream metadata validated successfully."
else
    echo "    [WARN] appstreamcli not found; skipping."
fi

# 2. Build Optimized Release Binary
echo "--> [2/4] Compiling optimized release binary..."
cargo build --release -p findme-cli

# 3. Packaging Debian Archive
echo "--> [3/4] Generating Debian package (.deb)..."
if command -v cargo-deb >/dev/null 2>&1; then
    DEB_OUTPUT=$(cargo deb -p findme-cli --no-build)
    echo "    [PASS] Generated package via cargo-deb: ${DEB_OUTPUT}"
elif command -v dpkg-deb >/dev/null 2>&1; then
    PKG_DIR="target/debian-staging/find-me_0.1.0_amd64"
    rm -rf "${PKG_DIR}"
    mkdir -p "${PKG_DIR}/usr/bin" \
             "${PKG_DIR}/usr/share/applications" \
             "${PKG_DIR}/usr/share/icons/hicolor/scalable/apps" \
             "${PKG_DIR}/usr/share/metainfo" \
             "${PKG_DIR}/DEBIAN"

    cp target/release/find-me "${PKG_DIR}/usr/bin/"
    chmod 755 "${PKG_DIR}/usr/bin/find-me"
    cp assets/find-me.desktop "${PKG_DIR}/usr/share/applications/"
    chmod 644 "${PKG_DIR}/usr/share/applications/find-me.desktop"
    cp assets/icons/find-me.svg "${PKG_DIR}/usr/share/icons/hicolor/scalable/apps/"
    chmod 644 "${PKG_DIR}/usr/share/icons/hicolor/scalable/apps/find-me.svg"
    cp assets/io.github.faizan_nadeem.find_me.metainfo.xml "${PKG_DIR}/usr/share/metainfo/"
    chmod 644 "${PKG_DIR}/usr/share/metainfo/io.github.faizan_nadeem.find_me.metainfo.xml"

    cat << 'EOF' > "${PKG_DIR}/DEBIAN/control"
Package: find-me
Version: 0.1.0-1
Section: utils
Priority: optional
Architecture: amd64
Maintainer: Find-Me Team <dev@find-me.local>
Depends: libc6 (>= 2.31), libwayland-client0 (>= 1.20.0), libxkbcommon0 (>= 1.0.0)
Homepage: https://github.com/Faizan-Nadeem/Find-Me
Description: Instant Linux desktop file search utility
 Find-Me is an offline-first, high-performance Linux desktop file search
 utility written in 100% pure Rust.
EOF

    mkdir -p target/debian
    dpkg-deb --build "${PKG_DIR}" target/debian/find-me_0.1.0-1_amd64.deb
    echo "    [PASS] Generated package via dpkg-deb: target/debian/find-me_0.1.0-1_amd64.deb"
else
    echo "    [ERROR] Neither cargo-deb nor dpkg-deb found." >&2
    exit 1
fi

# 4. Package Verification
echo "--> [4/4] Verifying Debian package contents..."
DEB_FILE=$(ls -t target/debian/find-me_*.deb | head -n 1)
dpkg-deb -I "${DEB_FILE}"
dpkg-deb -c "${DEB_FILE}"

echo "=========================================================="
echo "  [SUCCESS] Debian release package verified: ${DEB_FILE}"
echo "=========================================================="
