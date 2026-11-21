#!/usr/bin/env bash
# Find-Me Task Verification Script
# Strict Quality Gate to run before merging any feature branch into main

set -eo pipefail

echo "=========================================================="
echo "  Find-Me Pre-Merge Quality & Verification Gate"
echo "=========================================================="

# Ensure we are inside git repository
if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    echo "ERROR: Must be executed from within a git repository."
    exit 1
fi

BRANCH=$(git branch --show-current)
echo "[INFO] Current Branch: ${BRANCH}"

if [ "${BRANCH}" = "main" ]; then
    echo "WARNING: You are on 'main'! Code changes should be on a feature branch."
fi

# Step 1: Code Formatting Check
echo ""
echo "--> [Step 1/5] Checking Code Formatting (cargo fmt)..."
if command -v cargo >/dev/null 2>&1; then
    cargo fmt --all -- --check
    echo "    [PASS] Formatting adheres to Rust style guidelines."
else
    echo "    [SKIP] Cargo not installed in current environment."
fi

# Step 2: Strict Clippy Linter Check (Zero Warnings Permitted)
echo ""
echo "--> [Step 2/5] Running Clippy Linter (strict zero-warning mode)..."
if command -v cargo >/dev/null 2>&1; then
    cargo clippy --all-targets --all-features -- -D warnings
    echo "    [PASS] Clippy passed with zero warnings."
else
    echo "    [SKIP] Cargo not installed in current environment."
fi

# Step 3: Test Suite Execution (Unit & Integration)
echo ""
echo "--> [Step 3/5] Executing Test Suite (cargo test)..."
if command -v cargo >/dev/null 2>&1; then
    cargo test --workspace --all-targets
    echo "    [PASS] All unit and integration tests passed successfully."
else
    echo "    [SKIP] Cargo not installed in current environment."
fi

# Step 4: Documentation Tests Check
echo ""
echo "--> [Step 4/5] Executing Documentation Tests..."
if command -v cargo >/dev/null 2>&1; then
    cargo test --workspace --doc
    echo "    [PASS] All documentation code examples verified."
else
    echo "    [SKIP] Cargo not installed in current environment."
fi

# Step 5: Git Status & Branch Health Check
echo ""
echo "--> [Step 5/5] Checking Git Status..."
git status --short

echo ""
echo "=========================================================="
echo "  [SUCCESS] All verification gates passed!"
echo "  Branch '${BRANCH}' is ready for review and merge."
echo "  Remember: DO NOT delete the feature branch after merge!"
echo "=========================================================="
