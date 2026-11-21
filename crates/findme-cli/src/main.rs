//! Find-Me application binary and desktop launcher
//!
//! Provides the primary executable (`find-me`) launched from system application
//! menus, command line, or desktop managers. Supports GUI launch and CLI utilities.

use anyhow::Result;
use findme_ui::{window_settings, FindMeApp};
use std::env;
use tracing::info;

fn print_help() {
    println!(
        r#"Find-Me v{} — Instant Linux Desktop File Search Utility

USAGE:
    find-me [OPTIONS] [QUERY]

OPTIONS:
    -h, --help           Print help information
    -v, --version        Print version information
    --gui                Explicitly launch graphical desktop search interface (default)
    --rebuild-index      Perform background filesystem index crawl in CLI mode
    --new-window         Open new application search window

SHORTCUTS:
    Ctrl + F / /         Focus search input bar
    Ctrl + 1 .. 6        Switch navigation screens
    Ctrl + B             Toggle sidebar expansion
    Ctrl + T             Toggle dark / light visual theme
    F5 / Ctrl + R        Refresh active query
    Alt + 1 .. 5         Change table sorting column
    Enter                Open selected file with default desktop app
    Alt + Enter          Open file properties modal dialog
    Ctrl + Shift + C     Copy full POSIX path to clipboard
    Shift + F10          Open right-click context menu
    Escape               Context-sensitive dismiss hierarchy

FreeDesktop Integration:
    Desktop Launcher:    /usr/share/applications/find-me.desktop
    Scalable Icon:       /usr/share/icons/hicolor/scalable/apps/find-me.svg
    AppStream Metadata:  /usr/share/metainfo/io.github.faizan_nadeem.find_me.metainfo.xml"#,
        findme_core::VERSION
    );
}

fn print_version() {
    println!("find-me {}", findme_core::VERSION);
}

fn has_display_server() -> bool {
    env::var("DISPLAY").is_ok() || env::var("WAYLAND_DISPLAY").is_ok()
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args: Vec<String> = env::args().collect();

    if args.iter().any(|a| a == "-h" || a == "--help") {
        print_help();
        return Ok(());
    }

    if args.iter().any(|a| a == "-v" || a == "--version") {
        print_version();
        return Ok(());
    }

    if args.iter().any(|a| a == "--rebuild-index") {
        println!("Rebuilding Find-Me index from filesystem roots...");
        let arena = findme_ui::engine::EngineBridge::default_sample_arena();
        println!(
            "Indexed {} files in {} directories into compact arena.",
            arena.len(),
            arena.dirs().len()
        );
        return Ok(());
    }

    // Default: Launch graphical Iced application
    if !has_display_server() {
        eprintln!(
            "Find-Me: No graphical display detected ($DISPLAY or $WAYLAND_DISPLAY is not set)."
        );
        eprintln!(
            "Find-Me is a native Linux desktop application. Run with --help for CLI options."
        );
        return Ok(());
    }

    info!("Launching Find-Me graphical user interface");
    iced::application(FindMeApp::boot, FindMeApp::update, FindMeApp::view)
        .window(window_settings())
        .theme(FindMeApp::theme)
        .subscription(FindMeApp::subscription)
        .title(FindMeApp::title)
        .run()?;

    Ok(())
}
