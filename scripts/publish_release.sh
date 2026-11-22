#!/usr/bin/env bash
# ==============================================================================
# Find-Me GitHub Release Publishing Automation
# ==============================================================================
# Verifies packages, computes SHA256 checksums, and publishes to GitHub Releases
# via GitHub CLI (gh) or outputs instructions if not yet authenticated.
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

cd "${REPO_ROOT}"

TAG="${1:-v0.1.0}"
DIST_DIR="${REPO_ROOT}/target/release-dist"
NOTES_FILE="${REPO_ROOT}/releasenotes/${TAG}.md"

echo "=========================================================="
echo "  Find-Me Release Publisher (${TAG})"
echo "=========================================================="

# 1. Ensure packages are built
if [ ! -d "${DIST_DIR}" ] || [ -z "$(ls -A "${DIST_DIR}")" ]; then
    echo "--> [1/3] Building release packages..."
    bash scripts/package_deb.sh
    bash scripts/package_appimage.sh

    mkdir -p "${DIST_DIR}"
    cp target/debian/find-me_*.deb "${DIST_DIR}/"
    cp target/appimage/find-me-*.AppImage "${DIST_DIR}/"

    PKG_NAME="find-me-${TAG}-x86_64-unknown-linux-gnu"
    STAGING="target/${PKG_NAME}"
    rm -rf "${STAGING}"
    mkdir -p "${STAGING}"
    cp target/release/find-me "${STAGING}/"
    cp assets/find-me.desktop "${STAGING}/"
    cp assets/icons/find-me.svg "${STAGING}/"
    cp assets/io.github.faizan_nadeem.find_me.metainfo.xml "${STAGING}/"
    cp README.md "${STAGING}/"

    cd target
    tar -czvf "release-dist/${PKG_NAME}.tar.gz" "${PKG_NAME}"
    rm -rf "${PKG_NAME}"
    cd "${DIST_DIR}"
    sha256sum find-me* > SHA256SUMS.txt
    cd "${REPO_ROOT}"
fi

echo "--> [2/3] Checking release assets in ${DIST_DIR}..."
ls -lh "${DIST_DIR}"

# 2. Check GitHub CLI authentication
echo "--> [3/3] Publishing to GitHub Releases..."
if command -v gh >/dev/null 2>&1 && gh auth status >/dev/null 2>&1; then
    echo "    [PASS] GitHub CLI authenticated."
    NOTES_ARG=()
    if [ -f "${NOTES_FILE}" ]; then
        NOTES_ARG=(--notes-file "${NOTES_FILE}")
    else
        NOTES_ARG=(--generate-notes)
    fi

    gh release create "${TAG}" \
        "${DIST_DIR}"/find-me_*.deb \
        "${DIST_DIR}"/find-me-*.AppImage \
        "${DIST_DIR}"/find-me-*.tar.gz \
        "${DIST_DIR}"/SHA256SUMS.txt \
        --title "Find-Me ${TAG}" \
        "${NOTES_ARG[@]}"

    echo "=========================================================="
    echo "  [SUCCESS] GitHub Release published successfully!"
    echo "  URL: https://github.com/FaizanNadeem1287/find-me/releases/tag/${TAG}"
    echo "=========================================================="
else
    echo "    [NOTE] GitHub CLI (gh) is not authenticated."
    echo "    To publish this release directly from your terminal, run:"
    echo ""
    echo "      gh auth login"
    echo "      bash scripts/publish_release.sh ${TAG}"
    echo ""
    echo "    Or export a GitHub Personal Access Token:"
    echo "      export GH_TOKEN=\"ghp_...\""
    echo "      bash scripts/publish_release.sh ${TAG}"
    echo ""
    echo "    Alternatively, push the tag to trigger GitHub Actions:"
    echo "      git tag -a ${TAG} -m \"Release ${TAG}\""
    echo "      git push origin ${TAG}"
    echo "=========================================================="
fi
