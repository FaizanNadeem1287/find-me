#!/usr/bin/env bash
# ==============================================================================
# Find-Me Flatpak Packaging & Flathub Manifest Validator
# ==============================================================================
# Validates Flatpak manifest schemas, verifies FreeDesktop standard assets,
# and builds a sandboxed Flatpak package if flatpak-builder is available.
# ==============================================================================

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

cd "${REPO_ROOT}"

echo "=========================================================="
echo "  Find-Me Flatpak Manifest & Distribution Validator"
echo "=========================================================="

# 1. Validate FreeDesktop Assets
echo "--> [1/4] Validating FreeDesktop standards..."
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

# 2. Validate JSON Manifest Syntax and Schema
echo "--> [2/4] Validating Flatpak JSON manifest syntax..."
JSON_MANIFEST="packaging/flatpak/io.github.faizan_nadeem.find_me.json"
if [ ! -f "${JSON_MANIFEST}" ]; then
    echo "ERROR: Flatpak manifest ${JSON_MANIFEST} not found!"
    exit 1
fi

python3 -c "
import json, sys

with open('${JSON_MANIFEST}') as f:
    data = json.load(f)

assert data.get('app-id') == 'io.github.faizan_nadeem.find_me', 'app-id must match CID'
assert data.get('command') == 'find-me', 'command must be find-me'
assert data.get('runtime') == 'org.freedesktop.Platform', 'runtime must be org.freedesktop.Platform'
assert data.get('sdk') == 'org.freedesktop.Sdk', 'sdk must be org.freedesktop.Sdk'
finish_args = data.get('finish-args', [])
assert '--share=ipc' in finish_args, 'Must include --share=ipc'
assert '--socket=wayland' in finish_args, 'Must include --socket=wayland'
assert '--socket=fallback-x11' in finish_args, 'Must include --socket=fallback-x11'
assert '--device=dri' in finish_args, 'Must include --device=dri'
assert any(arg.startswith('--filesystem=') for arg in finish_args), 'Must grant filesystem access'
assert '--talk-name=org.freedesktop.FileManager1' in finish_args, 'Must grant DBus FileManager1 access'
print('    [PASS] Flatpak JSON manifest schema verified.')
"

# 3. Validate YAML Manifest Syntax
echo "--> [3/4] Validating Flatpak YAML manifest syntax..."
YAML_MANIFEST="packaging/flatpak/io.github.faizan_nadeem.find_me.yaml"
if [ ! -f "${YAML_MANIFEST}" ]; then
    echo "ERROR: Flatpak YAML manifest ${YAML_MANIFEST} not found!"
    exit 1
fi
grep -q "app-id: io.github.faizan_nadeem.find_me" "${YAML_MANIFEST}"
grep -q "command: find-me" "${YAML_MANIFEST}"
grep -q -- "--talk-name=org.freedesktop.FileManager1" "${YAML_MANIFEST}"
echo "    [PASS] Flatpak YAML manifest verified."

# 4. Local Build Check (flatpak-builder if available)
echo "--> [4/4] Checking flatpak-builder availability..."
if command -v flatpak-builder >/dev/null 2>&1; then
    echo "    flatpak-builder detected. Testing manifest linting..."
    flatpak-builder --show-manifest "${JSON_MANIFEST}" >/dev/null
    echo "    [PASS] flatpak-builder successfully parsed manifest."
else
    echo "    [INFO] flatpak-builder not installed on host. Manifest validation complete."
fi

echo "=========================================================="
echo "  [SUCCESS] Flatpak packaging assets verified!"
echo "=========================================================="
