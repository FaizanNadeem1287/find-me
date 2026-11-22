//! Comprehensive integration and QA tests for AppImage Universal Portable Packaging
//!
//! Validates:
//! 1. `scripts/package_appimage.sh` script attributes, permissions, and shebang.
//! 2. `target/appimage/AppDir` structure, FreeDesktop asset compliance, symlinks, and permissions.
//! 3. `AppRun` script execution with flags `--version` and `--help`.
//! 4. AppStream metainfo validation (`io.github.faizan_nadeem.find_me.metainfo.xml`).
//! 5. Standalone AppImage execution via `--appimage-extract-and-run`.

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;

fn find_repo_root() -> PathBuf {
    let mut current = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    while !current.join("Cargo.toml").exists() || !current.join("assets").exists() {
        if let Some(parent) = current.parent() {
            current = parent.to_path_buf();
        } else {
            break;
        }
    }
    current
}

static BUILD_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn ensure_appimage_built() {
    let _guard = match BUILD_LOCK.lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    };
    let root = find_repo_root();
    let appimage_path = root.join("target/appimage/find-me-0.1.0-x86_64.AppImage");
    let appdir_path = root.join("target/appimage/AppDir");
    if !appimage_path.exists() || !appdir_path.exists() {
        let script = root.join("scripts/package_appimage.sh");
        let status = Command::new("bash")
            .arg(&script)
            .current_dir(&root)
            .status()
            .expect("Failed to execute package_appimage.sh");
        assert!(status.success(), "package_appimage.sh execution failed");
    }
}

#[test]
fn test_package_script_properties() {
    let root = find_repo_root();
    let script_path = root.join("scripts/package_appimage.sh");
    assert!(
        script_path.exists(),
        "scripts/package_appimage.sh must exist"
    );

    let metadata = fs::metadata(&script_path).expect("Failed to get script metadata");
    let permissions = metadata.permissions();
    assert_ne!(
        permissions.mode() & 0o111,
        0,
        "scripts/package_appimage.sh must have executable bits set"
    );

    let content = fs::read_to_string(&script_path).expect("Failed to read script");
    assert!(
        content.starts_with("#!/usr/bin/env bash") || content.starts_with("#!/bin/bash"),
        "Script must have bash shebang"
    );
    assert!(
        content.contains("set -euo pipefail"),
        "Script must use strict bash error handling (set -euo pipefail)"
    );
    assert!(
        content.contains("AppDir"),
        "Script must reference AppDir assembly"
    );
    assert!(
        content.contains("appimagetool"),
        "Script must reference appimagetool"
    );
}

#[test]
fn test_appdir_hierarchy_and_permissions() {
    ensure_appimage_built();
    let root = find_repo_root();
    let appdir = root.join("target/appimage/AppDir");

    assert!(
        appdir.exists(),
        "AppDir must exist at target/appimage/AppDir"
    );

    // 1. AppRun
    let apprun = appdir.join("AppRun");
    assert!(apprun.exists(), "AppRun launcher must exist in AppDir root");
    let apprun_meta = fs::metadata(&apprun).expect("Failed to inspect AppRun");
    assert_ne!(
        apprun_meta.permissions().mode() & 0o111,
        0,
        "AppRun must be executable"
    );
    let apprun_content = fs::read_to_string(&apprun).expect("Failed to read AppRun");
    assert!(apprun_content.contains("exec \"${HERE}/usr/bin/find-me\" \"$@\""));

    // 2. Binary in usr/bin/find-me
    let binary = appdir.join("usr/bin/find-me");
    assert!(binary.exists(), "usr/bin/find-me must exist in AppDir");
    let bin_meta = fs::metadata(&binary).expect("Failed to inspect binary");
    assert_ne!(
        bin_meta.permissions().mode() & 0o111,
        0,
        "usr/bin/find-me must be executable"
    );

    // 3. Desktop Entry in usr/share/applications/find-me.desktop
    let desktop = appdir.join("usr/share/applications/find-me.desktop");
    assert!(desktop.exists(), "Desktop entry must exist in AppDir");
    let desktop_content = fs::read_to_string(&desktop).expect("Failed to read desktop file");
    assert!(desktop_content.contains("[Desktop Entry]"));
    assert!(desktop_content.contains("Exec=find-me %F"));
    assert!(desktop_content.contains("Icon=find-me"));

    // 4. SVG icon in usr/share/icons/hicolor/scalable/apps/find-me.svg
    let icon_svg = appdir.join("usr/share/icons/hicolor/scalable/apps/find-me.svg");
    assert!(icon_svg.exists(), "SVG icon must exist in AppDir");
    let icon_content = fs::read_to_string(&icon_svg).expect("Failed to read SVG icon");
    assert!(icon_content.contains("<svg"));

    // 5. AppStream metadata in usr/share/metainfo
    let metainfo = appdir.join("usr/share/metainfo/io.github.faizan_nadeem.find_me.metainfo.xml");
    assert!(metainfo.exists(), "AppStream metainfo must exist in AppDir");
    let metainfo_content = fs::read_to_string(&metainfo).expect("Failed to read metainfo");
    assert!(metainfo_content.contains("<id>io.github.faizan_nadeem.find_me</id>"));

    // 6. Root symlinks
    let root_desktop = appdir.join("find-me.desktop");
    assert!(
        root_desktop.exists(),
        "find-me.desktop must exist in AppDir root"
    );

    let root_svg = appdir.join("find-me.svg");
    let root_png = appdir.join("find-me.png");
    assert!(
        root_svg.exists() || root_png.exists(),
        "Application icon must exist in AppDir root"
    );

    let dir_icon = appdir.join(".DirIcon");
    assert!(dir_icon.exists(), ".DirIcon must exist in AppDir root");
}

#[test]
fn test_appdir_freedesktop_validation() {
    ensure_appimage_built();
    let root = find_repo_root();
    let appdir = root.join("target/appimage/AppDir");

    let desktop = appdir.join("usr/share/applications/find-me.desktop");
    if let Ok(output) = Command::new("desktop-file-validate").arg(&desktop).output() {
        assert!(
            output.status.success(),
            "desktop-file-validate failed on AppDir desktop file: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let metainfo = appdir.join("usr/share/metainfo/io.github.faizan_nadeem.find_me.metainfo.xml");
    if let Ok(output) = Command::new("appstreamcli")
        .args(["validate", "--no-net", &metainfo.to_string_lossy()])
        .output()
    {
        assert!(
            output.status.success(),
            "appstreamcli validate failed on AppDir metainfo: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn test_apprun_environment_and_execution() {
    ensure_appimage_built();
    let root = find_repo_root();
    let apprun = root.join("target/appimage/AppDir/AppRun");

    // Execute AppRun --version
    let version_output = Command::new(&apprun)
        .arg("--version")
        .output()
        .expect("Failed to execute AppRun --version");
    assert!(
        version_output.status.success(),
        "AppRun --version failed: {}",
        String::from_utf8_lossy(&version_output.stderr)
    );
    let stdout = String::from_utf8_lossy(&version_output.stdout);
    assert!(
        stdout.contains("find-me 0.1.0"),
        "Expected version 'find-me 0.1.0', got: {}",
        stdout
    );

    // Execute AppRun --help
    let help_output = Command::new(&apprun)
        .arg("--help")
        .output()
        .expect("Failed to execute AppRun --help");
    assert!(
        help_output.status.success(),
        "AppRun --help failed: {}",
        String::from_utf8_lossy(&help_output.stderr)
    );
    let help_stdout = String::from_utf8_lossy(&help_output.stdout);
    assert!(
        help_stdout.contains("--rebuild-index"),
        "Help output must mention CLI flags, got: {}",
        help_stdout
    );
}

#[test]
fn test_appimage_bundle_execution() {
    ensure_appimage_built();
    let root = find_repo_root();
    let appimage_path = root.join("target/appimage/find-me-0.1.0-x86_64.AppImage");

    assert!(
        appimage_path.exists(),
        "AppImage file must exist at target/appimage/find-me-0.1.0-x86_64.AppImage"
    );

    // Check file size > 2MB
    let metadata = fs::metadata(&appimage_path).expect("Failed to inspect AppImage file");
    let size_bytes = metadata.len();
    assert!(
        size_bytes > 2 * 1024 * 1024,
        "AppImage bundle should be > 2MB (was {} bytes)",
        size_bytes
    );

    // Check executable bits
    assert_ne!(
        metadata.permissions().mode() & 0o111,
        0,
        "AppImage must be executable"
    );

    // Execute bundle via --appimage-extract-and-run --version
    let version_output = Command::new(&appimage_path)
        .args(["--appimage-extract-and-run", "--version"])
        .output()
        .expect("Failed to execute AppImage --version");
    assert!(
        version_output.status.success(),
        "AppImage --version failed: {}",
        String::from_utf8_lossy(&version_output.stderr)
    );
    let stdout = String::from_utf8_lossy(&version_output.stdout);
    assert!(
        stdout.contains("find-me 0.1.0"),
        "Expected version 'find-me 0.1.0', got: {}",
        stdout
    );

    // Execute bundle via --appimage-extract-and-run --help
    let help_output = Command::new(&appimage_path)
        .args(["--appimage-extract-and-run", "--help"])
        .output()
        .expect("Failed to execute AppImage --help");
    assert!(
        help_output.status.success(),
        "AppImage --help failed: {}",
        String::from_utf8_lossy(&help_output.stderr)
    );
    let help_stdout = String::from_utf8_lossy(&help_output.stdout);
    assert!(
        help_stdout.contains("--rebuild-index"),
        "Help output must mention CLI flags, got: {}",
        help_stdout
    );
}
