//! Comprehensive integration and QA tests for Flatpak Manifest & Packaging
//!
//! Validates:
//! 1. `packaging/flatpak/io.github.faizan_nadeem.find_me.json` schema, App ID, sandboxing permissions.
//! 2. `packaging/flatpak/io.github.faizan_nadeem.find_me.yaml` structure and parity.
//! 3. `scripts/package_flatpak.sh` execution and validation.
//! 4. App ID alignment across AppStream, desktop file, and Flatpak manifest.

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
fn test_flatpak_manifest_json_specification() {
    let root = find_repo_root();
    let json_path = root.join("packaging/flatpak/io.github.faizan_nadeem.find_me.json");
    assert!(
        json_path.exists(),
        "Flatpak JSON manifest must exist in packaging/flatpak/"
    );

    let content = fs::read_to_string(&json_path).expect("Failed to read Flatpak JSON manifest");
    let json: serde_json::Value =
        serde_json::from_str(&content).expect("Flatpak manifest must be valid JSON");

    assert_eq!(
        json.get("app-id").and_then(|v| v.as_str()),
        Some("io.github.faizan_nadeem.find_me"),
        "app-id must match Flathub component ID"
    );
    assert_eq!(
        json.get("command").and_then(|v| v.as_str()),
        Some("find-me"),
        "command must be find-me"
    );
    assert_eq!(
        json.get("runtime").and_then(|v| v.as_str()),
        Some("org.freedesktop.Platform"),
        "runtime must be org.freedesktop.Platform"
    );
    assert_eq!(
        json.get("sdk").and_then(|v| v.as_str()),
        Some("org.freedesktop.Sdk"),
        "sdk must be org.freedesktop.Sdk"
    );

    // Verify SDK extension for Rust
    let sdk_extensions = json
        .get("sdk-extensions")
        .and_then(|v| v.as_array())
        .expect("sdk-extensions must be an array");
    assert!(
        sdk_extensions
            .iter()
            .any(|ext| ext.as_str() == Some("org.freedesktop.Sdk.Extension.rust-stable")),
        "sdk-extensions must include org.freedesktop.Sdk.Extension.rust-stable"
    );

    // Verify Finish-Args (Sandboxing Permissions)
    let finish_args = json
        .get("finish-args")
        .and_then(|v| v.as_array())
        .expect("finish-args must be an array");
    let args_strs: Vec<&str> = finish_args.iter().filter_map(|v| v.as_str()).collect();

    assert!(
        args_strs.contains(&"--share=ipc"),
        "finish-args must contain --share=ipc"
    );
    assert!(
        args_strs.contains(&"--socket=fallback-x11"),
        "finish-args must contain --socket=fallback-x11"
    );
    assert!(
        args_strs.contains(&"--socket=wayland"),
        "finish-args must contain --socket=wayland"
    );
    assert!(
        args_strs.contains(&"--device=dri"),
        "finish-args must contain --device=dri for GPU acceleration"
    );
    assert!(
        args_strs.iter().any(|arg| arg.starts_with("--filesystem=")),
        "finish-args must contain host filesystem permissions for file search crawler"
    );
    assert!(
        args_strs.contains(&"--talk-name=org.freedesktop.FileManager1"),
        "finish-args must allow talking to org.freedesktop.FileManager1 for file highlighting"
    );

    // Verify Modules
    let modules = json
        .get("modules")
        .and_then(|v| v.as_array())
        .expect("modules must be an array");
    assert!(
        !modules.is_empty(),
        "modules must contain at least one module"
    );
    let findme_module = modules
        .iter()
        .find(|m| m.get("name").and_then(|v| v.as_str()) == Some("find-me"))
        .expect("find-me module must exist in manifest");
    assert_eq!(
        findme_module.get("buildsystem").and_then(|v| v.as_str()),
        Some("simple")
    );
}

#[test]
fn test_flatpak_manifest_yaml_specification() {
    let root = find_repo_root();
    let yaml_path = root.join("packaging/flatpak/io.github.faizan_nadeem.find_me.yaml");
    assert!(
        yaml_path.exists(),
        "Flatpak YAML manifest must exist in packaging/flatpak/"
    );

    let content = fs::read_to_string(&yaml_path).expect("Failed to read Flatpak YAML manifest");
    assert!(content.contains("app-id: io.github.faizan_nadeem.find_me"));
    assert!(content.contains("runtime: org.freedesktop.Platform"));
    assert!(content.contains("sdk: org.freedesktop.Sdk"));
    assert!(content.contains("org.freedesktop.Sdk.Extension.rust-stable"));
    assert!(content.contains("command: find-me"));
    assert!(content.contains("--share=ipc"));
    assert!(content.contains("--socket=wayland"));
    assert!(content.contains("--socket=fallback-x11"));
    assert!(content.contains("--device=dri"));
    assert!(content.contains("--filesystem=host"));
    assert!(content.contains("--talk-name=org.freedesktop.FileManager1"));
}

#[test]
fn test_package_script_properties_and_execution() {
    let root = find_repo_root();
    let script_path = root.join("scripts/package_flatpak.sh");
    assert!(
        script_path.exists(),
        "scripts/package_flatpak.sh must exist"
    );

    let metadata = fs::metadata(&script_path).expect("Failed to inspect script");
    assert_ne!(
        metadata.permissions().mode() & 0o111,
        0,
        "scripts/package_flatpak.sh must have executable bits set"
    );

    let output = Command::new("bash")
        .arg(&script_path)
        .current_dir(&root)
        .output()
        .expect("Failed to execute package_flatpak.sh");

    assert!(
        output.status.success(),
        "package_flatpak.sh failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("[SUCCESS] Flatpak packaging assets verified!"),
        "Script output missing success confirmation: {}",
        stdout
    );
}

#[test]
fn test_flatpak_app_id_matches_appstream_and_desktop() {
    let root = find_repo_root();
    let xml_path = root.join("assets/io.github.faizan_nadeem.find_me.metainfo.xml");
    let xml_content = fs::read_to_string(&xml_path).expect("Failed to read AppStream XML");
    assert!(
        xml_content.contains("<id>io.github.faizan_nadeem.find_me</id>"),
        "AppStream XML must have matching CID"
    );

    let desktop_path = root.join("assets/find-me.desktop");
    let desktop_content = fs::read_to_string(&desktop_path).expect("Failed to read desktop file");
    assert!(desktop_content.contains("StartupWMClass=find-me"));
    assert!(desktop_content.contains("Exec=find-me %F"));
}
