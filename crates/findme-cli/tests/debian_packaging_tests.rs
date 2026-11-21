//! Comprehensive integration and QA tests for Debian Packaging & FreeDesktop standards
//!
//! Validates `find-me.desktop`, AppStream metadata, vector SVG icon, Cargo.toml
//! debian metadata, binary CLI options, and `.deb` archive integrity.

use std::fs;
use std::process::Command;

fn find_repo_root() -> std::path::PathBuf {
    let mut current = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
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
fn test_desktop_entry_specification() {
    let root = find_repo_root();
    let desktop_path = root.join("assets/find-me.desktop");
    assert!(
        desktop_path.exists(),
        "assets/find-me.desktop must exist in repo root"
    );

    let content = fs::read_to_string(&desktop_path).expect("Failed to read desktop file");
    assert!(content.contains("[Desktop Entry]"));
    assert!(content.contains("Type=Application"));
    assert!(content.contains("Name=Find-Me"));
    assert!(content.contains("GenericName=File Search Utility"));
    assert!(content.contains("Exec=find-me %F"));
    assert!(content.contains("Icon=find-me"));
    assert!(content.contains("Categories=System;Filesystem;"));
    assert!(content.contains("Keywords=search;find;everything;files;fast;index;"));
    assert!(content.contains("StartupNotify=true"));
    assert!(content.contains("StartupWMClass=find-me"));
    assert!(content.contains("Actions=NewWindow;RebuildIndex;"));

    // Run official validator if installed on system
    if let Ok(output) = Command::new("desktop-file-validate")
        .arg(&desktop_path)
        .output()
    {
        assert!(
            output.status.success(),
            "desktop-file-validate failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn test_appstream_metadata_specification() {
    let root = find_repo_root();
    let xml_path = root.join("assets/io.github.faizan_nadeem.find_me.metainfo.xml");
    assert!(
        xml_path.exists(),
        "AppStream XML metadata file must exist in assets/"
    );

    let content = fs::read_to_string(&xml_path).expect("Failed to read AppStream XML");
    assert!(content.contains("<id>io.github.faizan_nadeem.find_me</id>"));
    assert!(content.contains("<name>Find-Me</name>"));
    assert!(content.contains("<launchable type=\"desktop-id\">find-me.desktop</launchable>"));
    assert!(content.contains("<provides>\n    <binary>find-me</binary>\n  </provides>"));
    assert!(content.contains("<metadata_license>FSFAP</metadata_license>"));
    assert!(content.contains("<project_license>MIT OR Apache-2.0</project_license>"));

    // Run official appstreamcli validator if installed on system
    if let Ok(output) = Command::new("appstreamcli")
        .args(["validate", "--no-net", &xml_path.to_string_lossy()])
        .output()
    {
        assert!(
            output.status.success(),
            "appstreamcli validate failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn test_svg_vector_icon_specification() {
    let root = find_repo_root();
    let svg_path = root.join("assets/icons/find-me.svg");
    assert!(
        svg_path.exists(),
        "Vector SVG icon must exist in assets/icons/"
    );

    let content = fs::read_to_string(&svg_path).expect("Failed to read SVG file");
    assert!(content.contains("<svg"));
    assert!(content.contains("viewBox=\"0 0 512 512\""));
    assert!(content.contains("width=\"512\""));
    assert!(content.contains("height=\"512\""));
    assert!(content.contains("linearGradient"));
    assert!(content.contains("</svg>"));
}

#[test]
fn test_cargo_deb_metadata_configuration() {
    let root = find_repo_root();
    let cargo_toml_path = root.join("crates/findme-cli/Cargo.toml");
    let content =
        fs::read_to_string(&cargo_toml_path).expect("Failed to read findme-cli Cargo.toml");

    assert!(content.contains("[package.metadata.deb]"));
    assert!(content.contains("name = \"find-me\""));
    assert!(content.contains("section = \"utils\""));
    assert!(content.contains("priority = \"optional\""));
    assert!(content.contains("depends = \"$auto"));
    assert!(content.contains("usr/bin/find-me"));
    assert!(content.contains("usr/share/applications/find-me.desktop"));
    assert!(content.contains("usr/share/icons/hicolor/scalable/apps/find-me.svg"));
    assert!(content.contains("usr/share/metainfo/io.github.faizan_nadeem.find_me.metainfo.xml"));
}

#[test]
fn test_cli_help_and_version_flags() {
    let root = find_repo_root();
    let bin_path = root.join("target/debug/find-me");
    let release_bin_path = root.join("target/release/find-me");

    let target_bin = if bin_path.exists() {
        bin_path
    } else if release_bin_path.exists() {
        release_bin_path
    } else {
        // Compile binary if not present
        let status = Command::new("cargo")
            .args(["build", "-p", "findme-cli"])
            .status()
            .expect("Failed to build findme-cli for test");
        assert!(status.success());
        root.join("target/debug/find-me")
    };

    // 1. Test --help
    let help_output = Command::new(&target_bin)
        .arg("--help")
        .output()
        .expect("Failed to execute find-me --help");
    assert!(help_output.status.success());
    let help_text = String::from_utf8_lossy(&help_output.stdout);
    assert!(help_text.contains("Find-Me"));
    assert!(help_text.contains("OPTIONS:"));
    assert!(help_text.contains("FreeDesktop Integration:"));

    // 2. Test --version
    let ver_output = Command::new(&target_bin)
        .arg("--version")
        .output()
        .expect("Failed to execute find-me --version");
    assert!(ver_output.status.success());
    let ver_text = String::from_utf8_lossy(&ver_output.stdout);
    assert!(ver_text.contains("find-me"));
    assert!(ver_text.contains(findme_core::VERSION));

    // 3. Test --rebuild-index
    let rebuild_output = Command::new(&target_bin)
        .arg("--rebuild-index")
        .output()
        .expect("Failed to execute find-me --rebuild-index");
    assert!(rebuild_output.status.success());
    let rebuild_text = String::from_utf8_lossy(&rebuild_output.stdout);
    assert!(rebuild_text.contains("Rebuilding Find-Me index"));
}

#[test]
fn test_debian_package_archive_integrity() {
    let root = find_repo_root();
    let deb_dir = root.join("target/debian");

    if !deb_dir.exists() {
        return; // Staging environment where package has not yet been built
    }

    let mut entries = fs::read_dir(&deb_dir).expect("Failed to read deb dir");
    let deb_file = entries.find_map(|e| {
        let entry = e.ok()?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("deb") {
            Some(path)
        } else {
            None
        }
    });

    if let Some(deb_path) = deb_file {
        let meta = fs::metadata(&deb_path).expect("Failed to read deb metadata");
        assert!(meta.len() > 500_000, "Debian package must be over 500KB");

        // Inspect package structure with dpkg-deb if available
        if let Ok(output) = Command::new("dpkg-deb").arg("-c").arg(&deb_path).output() {
            if output.status.success() {
                let listing = String::from_utf8_lossy(&output.stdout);
                assert!(listing.contains("usr/bin/find-me"));
                assert!(listing.contains("usr/share/applications/find-me.desktop"));
                assert!(listing.contains("usr/share/icons/hicolor/scalable/apps/find-me.svg"));
                assert!(listing
                    .contains("usr/share/metainfo/io.github.faizan_nadeem.find_me.metainfo.xml"));
            }
        }
    }
}
