//! Find-Me application binary and desktop launcher
//!
//! Provides the primary executable (`find-me`) launched from system application
//! menus, command line, or desktop managers. Supports GUI launch and CLI utilities.

use anyhow::Result;
use findme_cli::{
    execute_cli_search, execute_cli_stats, has_display_server, parse_cli_args, print_help,
    print_version,
};
use findme_core::IndexStorage;
use findme_ui::{window_settings, FindMeApp};
use std::env;
use tracing::info;

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args: Vec<String> = env::args().collect();
    let opts = parse_cli_args(&args);

    if opts.show_help {
        print_help();
        return Ok(());
    }

    if opts.show_version {
        print_version();
        return Ok(());
    }

    let cache_path = IndexStorage::default_cache_path();

    if opts.rebuild_index {
        println!("Rebuilding Find-Me index from filesystem roots...");
        let arena = findme_ui::engine::EngineBridge::default_sample_arena();
        let _ = IndexStorage::save_to_file(&arena, &cache_path);
        println!(
            "Indexed {} files in {} directories into compact arena.",
            arena.len(),
            arena.dirs().len()
        );
        return Ok(());
    }

    if opts.show_stats {
        execute_cli_stats(&cache_path);
        return Ok(());
    }

    // CLI mode triggered when --cli, --json, query provided in headless environment,
    // or when query is given and display is explicitly not requested.
    let display_available = has_display_server();
    let is_headless = !display_available;
    let run_as_cli = opts.is_cli_mode
        || opts.is_json
        || (is_headless && opts.query.is_some())
        || (opts.query.is_some() && !opts.is_gui_mode && is_headless);

    if run_as_cli {
        return execute_cli_search(&opts, &cache_path);
    }

    // Default: Launch graphical Iced application
    if !display_available {
        eprintln!(
            "Find-Me: No graphical display detected ($DISPLAY or $WAYLAND_DISPLAY is not set)."
        );
        eprintln!(
            "Find-Me is a native Linux desktop application. Run with --help for CLI options."
        );
        return Ok(());
    }

    let initial_query = opts.query;
    info!("Launching Find-Me graphical user interface");
    iced::application(
        move || FindMeApp::boot_with_query(initial_query.clone()),
        FindMeApp::update,
        FindMeApp::view,
    )
    .window(window_settings())
    .theme(FindMeApp::theme)
    .subscription(FindMeApp::subscription)
    .title(FindMeApp::title)
    .run()?;

    Ok(())
}
