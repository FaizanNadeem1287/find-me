#!/usr/bin/env bash
# ==============================================================================
# Find-Me User-Space Desktop Installer
# ==============================================================================
# Installs Find-Me into the current user's desktop environment without
# requiring root or sudo privileges, complying strictly with FreeDesktop
# standards (XDG Base Directory Specification).
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

cd "${REPO_ROOT}"

echo "=========================================================="
echo "  Find-Me Desktop Installation (User Environment)"
echo "=========================================================="

# 1. Compile Optimized Release Binary
echo ""
echo "--> [Step 1/5] Compiling optimized release binary..."
if ! command -v cargo >/dev/null 2>&1; then
    if [ -f "$HOME/.cargo/env" ]; then
        source "$HOME/.cargo/env"
    fi
fi

cargo build --release -p findme-cli
echo "    [PASS] Release binary built: target/release/find-me"

# 2. Setup Target User Directories
echo ""
echo "--> [Step 2/5] Preparing FreeDesktop user directory hierarchy..."
BIN_DIR="${HOME}/.local/bin"
APP_DIR="${HOME}/.local/share/applications"
ICON_DIR="${HOME}/.local/share/icons/hicolor/scalable/apps"
META_DIR="${HOME}/.local/share/metainfo"

mkdir -p "${BIN_DIR}" "${APP_DIR}" "${ICON_DIR}" "${META_DIR}"
echo "    [PASS] Directories verified:"
echo "      - Binary:       ${BIN_DIR}"
echo "      - Application:  ${APP_DIR}"
echo "      - Scalable Icon:${ICON_DIR}"
echo "      - AppStream:    ${META_DIR}"

# 3. Install Executable Binary
echo ""
echo "--> [Step 3/5] Installing executable binary..."
cp -f target/release/find-me "${BIN_DIR}/find-me"
chmod 755 "${BIN_DIR}/find-me"
echo "    [PASS] Installed: ${BIN_DIR}/find-me"

# 4. Install FreeDesktop Assets
echo ""
echo "--> [Step 4/5] Installing desktop launcher, branding icon & AppStream metadata..."
cp -f assets/icons/find-me.svg "${ICON_DIR}/find-me.svg"
chmod 644 "${ICON_DIR}/find-me.svg"
echo "    [PASS] Installed icon: ${ICON_DIR}/find-me.svg"

cp -f assets/io.github.faizan_nadeem.find_me.metainfo.xml "${META_DIR}/io.github.faizan_nadeem.find_me.metainfo.xml"
chmod 644 "${META_DIR}/io.github.faizan_nadeem.find_me.metainfo.xml"
echo "    [PASS] Installed metainfo: ${META_DIR}/io.github.faizan_nadeem.find_me.metainfo.xml"

# Update desktop file with absolute or system path
sed -e "s|Exec=find-me|Exec=${BIN_DIR}/find-me|" assets/find-me.desktop > "${APP_DIR}/find-me.desktop"
chmod 644 "${APP_DIR}/find-me.desktop"
echo "    [PASS] Installed launcher: ${APP_DIR}/find-me.desktop"

# Optional Desktop Shortcut if ~/Desktop exists
if [ -d "${HOME}/Desktop" ]; then
    cp -f "${APP_DIR}/find-me.desktop" "${HOME}/Desktop/find-me.desktop"
    chmod +x "${HOME}/Desktop/find-me.desktop"
    if command -v gio >/dev/null 2>&1; then
        gio set -t string "${HOME}/Desktop/find-me.desktop" metadata::trusted true 2>/dev/null || true
    fi
    echo "    [PASS] Created Desktop shortcut: ${HOME}/Desktop/find-me.desktop"
fi

# 5. Refresh System Desktop & Icon Caches
echo ""
echo "--> [Step 5/5] Refreshing FreeDesktop application database & icon caches..."
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "${APP_DIR}" 2>/dev/null || true
    echo "    [PASS] Desktop database updated."
fi

if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -f -t "${HOME}/.local/share/icons/hicolor" 2>/dev/null || true
    echo "    [PASS] Icon cache updated."
fi

# Test Binary Execution
echo ""
echo "=========================================================="
echo "  Verification & Launch Confirmation"
echo "=========================================================="
"${BIN_DIR}/find-me" --version
"${BIN_DIR}/find-me" --stats

echo ""
echo "=========================================================="
echo "  [SUCCESS] Find-Me is now installed and ready to use!"
echo "=========================================================="
echo "  You can launch Find-Me in the following ways:"
echo "  1. Click 'Find-Me' in your Application Menu / Dash."
echo "  2. Double-click the desktop shortcut on your Desktop."
echo "  3. Run 'find-me' directly in any terminal."
echo "=========================================================="
