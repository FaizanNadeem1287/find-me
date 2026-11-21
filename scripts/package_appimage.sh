#!/usr/bin/env bash
# ==============================================================================
# Find-Me AppImage Universal Portable Release Bundle Builder
# ==============================================================================
# Automates compiling the release binary, assembling the FreeDesktop AppDir,
# and producing a self-contained, portable x86_64 AppImage for any Linux system.
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

cd "${REPO_ROOT}"

echo "=========================================================="
echo "  Find-Me AppImage Universal Release Packaging"
echo "=========================================================="

# 1. Validate FreeDesktop Assets
echo "--> [1/5] Validating FreeDesktop Standards..."
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
echo "--> [2/5] Compiling optimized release binary..."
if [ -f "$HOME/.cargo/env" ]; then
    # shellcheck source=/dev/null
    source "$HOME/.cargo/env"
fi
cargo build --release -p findme-cli

# 3. Assemble AppDir
echo "--> [3/5] Assembling AppDir structure..."
APPDIR="target/appimage/AppDir"
rm -rf "${APPDIR}"
mkdir -p "${APPDIR}/usr/bin" \
         "${APPDIR}/usr/share/applications" \
         "${APPDIR}/usr/share/icons/hicolor/scalable/apps" \
         "${APPDIR}/usr/share/metainfo"

# Install binary
cp target/release/find-me "${APPDIR}/usr/bin/find-me"
chmod 755 "${APPDIR}/usr/bin/find-me"

# Install FreeDesktop assets
cp assets/find-me.desktop "${APPDIR}/usr/share/applications/find-me.desktop"
chmod 644 "${APPDIR}/usr/share/applications/find-me.desktop"

cp assets/icons/find-me.svg "${APPDIR}/usr/share/icons/hicolor/scalable/apps/find-me.svg"
chmod 644 "${APPDIR}/usr/share/icons/hicolor/scalable/apps/find-me.svg"

cp assets/io.github.faizan_nadeem.find_me.metainfo.xml "${APPDIR}/usr/share/metainfo/io.github.faizan_nadeem.find_me.metainfo.xml"
chmod 644 "${APPDIR}/usr/share/metainfo/io.github.faizan_nadeem.find_me.metainfo.xml"

# Root symlinks & icon assets for AppImage runtime
ln -sf usr/share/applications/find-me.desktop "${APPDIR}/find-me.desktop"
ln -sf usr/share/icons/hicolor/scalable/apps/find-me.svg "${APPDIR}/find-me.svg"

# Generate 256x256 PNG for file manager thumbnailing and .DirIcon
if command -v rsvg-convert >/dev/null 2>&1; then
    rsvg-convert -w 256 -h 256 assets/icons/find-me.svg -o "${APPDIR}/find-me.png"
    ln -sf find-me.png "${APPDIR}/.DirIcon"
elif command -v convert >/dev/null 2>&1; then
    convert -background none -resize 256x256 assets/icons/find-me.svg "${APPDIR}/find-me.png"
    ln -sf find-me.png "${APPDIR}/.DirIcon"
else
    ln -sf find-me.svg "${APPDIR}/.DirIcon"
fi

# Create AppRun launcher script
cat << 'EOF' > "${APPDIR}/AppRun"
#!/bin/sh
SELF=$(readlink -f "$0")
HERE=${SELF%/*}
export PATH="${HERE}/usr/bin:${PATH}"
export LD_LIBRARY_PATH="${HERE}/usr/lib:${LD_LIBRARY_PATH:-}"
export XDG_DATA_DIRS="${HERE}/usr/share:${XDG_DATA_DIRS:-/usr/local/share:/usr/share}"
exec "${HERE}/usr/bin/find-me" "$@"
EOF
chmod 755 "${APPDIR}/AppRun"

echo "    [PASS] AppDir assembled at: ${APPDIR}"

# 4. Acquire appimagetool if not available
echo "--> [4/5] Preparing appimagetool..."
APPIMAGETOOL=""
if command -v appimagetool >/dev/null 2>&1; then
    APPIMAGETOOL="appimagetool"
elif [ -f "target/tools/appimagetool" ]; then
    APPIMAGETOOL="target/tools/appimagetool"
else
    mkdir -p target/tools
    echo "    Downloading appimagetool to target/tools/..."
    curl -sSL -o target/tools/appimagetool https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage
    chmod +x target/tools/appimagetool
    APPIMAGETOOL="target/tools/appimagetool"
fi

# 5. Package into standalone AppImage
echo "--> [5/5] Generating AppImage bundle..."
OUTPUT_APPIMAGE="target/appimage/find-me-0.1.0-x86_64.AppImage"
rm -f "${OUTPUT_APPIMAGE}"

ARCH=x86_64 "${APPIMAGETOOL}" --appimage-extract-and-run -n "${APPDIR}" "${OUTPUT_APPIMAGE}"
chmod +x "${OUTPUT_APPIMAGE}"

# Verify bundle execution
echo "--> Verifying AppImage bundle execution..."
"${OUTPUT_APPIMAGE}" --appimage-extract-and-run --version

FILE_SIZE=$(du -h "${OUTPUT_APPIMAGE}" | cut -f1)
echo "=========================================================="
echo "  [SUCCESS] AppImage package built: ${OUTPUT_APPIMAGE} (${FILE_SIZE})"
echo "=========================================================="
