//! Comprehensive integration and QA tests for Arch Linux AUR Packaging (PKGBUILD & .SRCINFO)
//!
//! Validates:
//! 1. `packaging/aur/PKGBUILD` syntax, variables, dependencies, and packaging functions.
//! 2. `packaging/aur/.SRCINFO` consistency with PKGBUILD.
//! 3. `scripts/package_aur.sh` execution and validation.
//! 4. Consistency of referenced FreeDesktop assets (desktop entry, SVG icon, AppStream metadata).

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

#[test]
fn test_aur_pkgbuild_specification() {
    let root = find_repo_root();
    let pkgbuild_path = root.join("packaging/aur/PKGBUILD");
    assert!(
        pkgbuild_path.exists(),
        "PKGBUILD must exist in packaging/aur/"
    );

    let content = fs::read_to_string(&pkgbuild_path).expect("Failed to read PKGBUILD");

    // Package identity and version
    assert!(
        content.contains("pkgname=find-me"),
        "pkgname must be find-me"
    );
    assert!(content.contains("pkgver=0.1.0"), "pkgver must be 0.1.0");
    assert!(content.contains("pkgrel=1"), "pkgrel must be 1");
    assert!(
        content.contains("arch=('x86_64')"),
        "arch must include x86_64"
    );
    assert!(
        content.contains("license=('MIT' 'Apache-2.0')"),
        "license must match MIT OR Apache-2.0"
    );

    // Dependencies
    assert!(
        content.contains("depends=('glibc' 'libxkbcommon' 'wayland')"),
        "depends must list glibc, libxkbcommon, and wayland"
    );
    assert!(
        content.contains("makedepends=('cargo' 'git')"),
        "makedepends must list cargo and git"
    );
    assert!(
        content.contains("optdepends=("),
        "optdepends must be specified"
    );
    assert!(
        content.contains("org.freedesktop.FileManager1")
            || content.contains("nautilus")
            || content.contains("dolphin"),
        "optdepends must mention file manager or portal support"
    );

    // Functions
    assert!(content.contains("build()"), "build() function must exist");
    assert!(
        content.contains("cargo build --release --locked -p findme-cli"),
        "build() must invoke cargo build for findme-cli"
    );

    assert!(
        content.contains("package()"),
        "package() function must exist"
    );
    assert!(
        content.contains("install -Dm755 \"target/release/find-me\" \"$pkgdir/usr/bin/find-me\""),
        "package() must install binary to /usr/bin/find-me"
    );
    assert!(
        content.contains("install -Dm644 \"assets/find-me.desktop\""),
        "package() must install desktop entry"
    );
    assert!(
        content.contains("install -Dm644 \"assets/icons/find-me.svg\""),
        "package() must install SVG icon"
    );
    assert!(
        content.contains("install -Dm644 \"assets/io.github.faizan_nadeem.find_me.metainfo.xml\""),
        "package() must install AppStream metadata"
    );
}

#[test]
fn test_aur_srcinfo_specification() {
    let root = find_repo_root();
    let srcinfo_path = root.join("packaging/aur/.SRCINFO");
    assert!(
        srcinfo_path.exists(),
        ".SRCINFO must exist in packaging/aur/"
    );

    let content = fs::read_to_string(&srcinfo_path).expect("Failed to read .SRCINFO");

    assert!(
        content.contains("pkgbase = find-me"),
        ".SRCINFO must specify pkgbase = find-me"
    );
    assert!(
        content.contains("pkgname = find-me"),
        ".SRCINFO must specify pkgname = find-me"
    );
    assert!(
        content.contains("pkgver = 0.1.0"),
        ".SRCINFO must specify pkgver = 0.1.0"
    );
    assert!(
        content.contains("pkgrel = 1"),
        ".SRCINFO must specify pkgrel = 1"
    );
    assert!(
        content.contains("arch = x86_64"),
        ".SRCINFO must specify arch = x86_64"
    );
    assert!(
        content.contains("license = MIT") && content.contains("license = Apache-2.0"),
        ".SRCINFO must list MIT and Apache-2.0 licenses"
    );
    assert!(
        content.contains("depends = glibc")
            && content.contains("depends = libxkbcommon")
            && content.contains("depends = wayland"),
        ".SRCINFO must list runtime dependencies"
    );
    assert!(
        content.contains("makedepends = cargo"),
        ".SRCINFO must list cargo build dependency"
    );
}

#[test]
fn test_aur_package_script_properties_and_execution() {
    let root = find_repo_root();
    let script_path = root.join("scripts/package_aur.sh");
    assert!(script_path.exists(), "scripts/package_aur.sh must exist");

    let metadata = fs::metadata(&script_path).expect("Failed to inspect script");
    assert_ne!(
        metadata.permissions().mode() & 0o111,
        0,
        "scripts/package_aur.sh must have executable bits set"
    );

    let output = Command::new("bash")
        .arg(&script_path)
        .current_dir(&root)
        .output()
        .expect("Failed to execute package_aur.sh");

    assert!(
        output.status.success(),
        "package_aur.sh failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("[SUCCESS] Arch Linux AUR packaging assets verified!"),
        "Script output missing success confirmation: {}",
        stdout
    );
}

#[test]
fn test_aur_freedesktop_assets_consistency() {
    let root = find_repo_root();

    let desktop_path = root.join("assets/find-me.desktop");
    assert!(desktop_path.exists(), "assets/find-me.desktop must exist");

    let svg_path = root.join("assets/icons/find-me.svg");
    assert!(svg_path.exists(), "assets/icons/find-me.svg must exist");

    let xml_path = root.join("assets/io.github.faizan_nadeem.find_me.metainfo.xml");
    assert!(
        xml_path.exists(),
        "assets/io.github.faizan_nadeem.find_me.metainfo.xml must exist"
    );
}
