#!/usr/bin/env bash
# ==============================================================================
# Find-Me Arch Linux AUR Package & PKGBUILD Validator
# ==============================================================================
# Validates PKGBUILD syntax, verifies .SRCINFO metadata parity, and checks
# FreeDesktop asset compliance for Arch Linux User Repository (AUR) packaging.
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

cd "${REPO_ROOT}"

echo "=========================================================="
echo "  Find-Me Arch Linux AUR Package & PKGBUILD Validator"
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

# 2. Validate PKGBUILD Syntax & Attributes
echo "--> [2/4] Validating PKGBUILD syntax and required attributes..."
PKGBUILD="packaging/aur/PKGBUILD"
if [ ! -f "${PKGBUILD}" ]; then
    echo "ERROR: PKGBUILD not found at ${PKGBUILD}!"
    exit 1
fi

grep -q "pkgname=find-me" "${PKGBUILD}" || { echo "ERROR: pkgname must be find-me"; exit 1; }
grep -q "pkgver=0.1.0" "${PKGBUILD}" || { echo "ERROR: pkgver must be 0.1.0"; exit 1; }
grep -q "pkgrel=1" "${PKGBUILD}" || { echo "ERROR: pkgrel must be 1"; exit 1; }
grep -q "arch=('x86_64')" "${PKGBUILD}" || { echo "ERROR: arch must include x86_64"; exit 1; }
grep -q "license=('MIT' 'Apache-2.0')" "${PKGBUILD}" || { echo "ERROR: license must match MIT OR Apache-2.0"; exit 1; }
grep -q "build()" "${PKGBUILD}" || { echo "ERROR: build() function missing"; exit 1; }
grep -q "package()" "${PKGBUILD}" || { echo "ERROR: package() function missing"; exit 1; }
grep -q "target/release/find-me" "${PKGBUILD}" || { echo "ERROR: target binary missing in package()"; exit 1; }
echo "    [PASS] PKGBUILD attributes and functions verified."

# 3. Validate .SRCINFO Parity
echo "--> [3/4] Validating .SRCINFO consistency..."
SRCINFO="packaging/aur/.SRCINFO"
if [ ! -f "${SRCINFO}" ]; then
    echo "ERROR: .SRCINFO not found at ${SRCINFO}!"
    exit 1
fi

grep -q "pkgbase = find-me" "${SRCINFO}" || { echo "ERROR: pkgbase mismatch in .SRCINFO"; exit 1; }
grep -q "pkgname = find-me" "${SRCINFO}" || { echo "ERROR: pkgname mismatch in .SRCINFO"; exit 1; }
grep -q "pkgver = 0.1.0" "${SRCINFO}" || { echo "ERROR: pkgver mismatch in .SRCINFO"; exit 1; }
grep -q "arch = x86_64" "${SRCINFO}" || { echo "ERROR: arch mismatch in .SRCINFO"; exit 1; }
echo "    [PASS] .SRCINFO metadata synchronized."

# 4. Check makepkg availability for local build verification
echo "--> [4/4] Checking makepkg availability..."
if command -v makepkg >/dev/null 2>&1; then
    echo "    makepkg detected. Validating .SRCINFO generation..."
    cd packaging/aur
    makepkg --printsrcinfo >/dev/null
    echo "    [PASS] makepkg verified PKGBUILD syntax successfully."
else
    echo "    [INFO] makepkg not installed on host. Standard linting completed."
fi

echo "=========================================================="
echo "  [SUCCESS] Arch Linux AUR packaging assets verified!"
echo "=========================================================="
