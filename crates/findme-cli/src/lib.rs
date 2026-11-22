//! Command-line interface logic, argument parsing, and search execution for Find-Me

use anyhow::Result;
use findme_core::{
    IndexStorage, QueryParser, SearchConfig, SearchEngine, SortColumn, SortDirection,
};
use serde::{Deserialize, Serialize};
use std::env;
use std::path::Path;

/// Command-line parsed options
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CliOptions {
    pub query: Option<String>,
    pub is_cli_mode: bool,
    pub is_gui_mode: bool,
    pub is_json: bool,
    pub max_results: usize,
    pub show_stats: bool,
    pub rebuild_index: bool,
    pub show_help: bool,
    pub show_version: bool,
}

/// Parses command line arguments into `CliOptions`
pub fn parse_cli_args(args: &[String]) -> CliOptions {
    let mut opts = CliOptions {
        max_results: 100,
        ..Default::default()
    };

    let mut iter = args.iter().skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-h" | "--help" => opts.show_help = true,
            "-v" | "--version" => opts.show_version = true,
            "--gui" => opts.is_gui_mode = true,
            "--cli" | "--search" => opts.is_cli_mode = true,
            "--json" => {
                opts.is_json = true;
                opts.is_cli_mode = true;
            }
            "--stats" => {
                opts.show_stats = true;
                opts.is_cli_mode = true;
            }
            "--rebuild-index" => opts.rebuild_index = true,
            "--new-window" => opts.is_gui_mode = true,
            "-q" | "--query" => {
                if let Some(val) = iter.next() {
                    opts.query = Some(val.clone());
                }
            }
            "-n" | "--max-results" => {
                if let Some(val) = iter.next() {
                    if let Ok(num) = val.parse::<usize>() {
                        opts.max_results = num;
                    }
                }
            }
            other if !other.starts_with('-') && opts.query.is_none() => {
                opts.query = Some(other.to_string());
            }
            _ => {}
        }
    }

    opts
}

/// Serialized search result item for CLI JSON output
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct CliSearchResult {
    pub path: String,
    pub name: String,
    pub size_bytes: u64,
    pub mtime_secs: u64,
    pub is_dir: bool,
    pub is_network: bool,
    pub score: u32,
}

/// Returns true if a graphical display server (X11 or Wayland) is available in environment
pub fn has_display_server() -> bool {
    env::var("DISPLAY").is_ok() || env::var("WAYLAND_DISPLAY").is_ok()
}

/// Loads the arena from persistent cache file, falling back to sample arena if absent or corrupt
pub fn load_or_fallback_arena(cache_path: &Path) -> findme_core::FileArena {
    if cache_path.exists() {
        if let Ok(arena) = IndexStorage::load_from_file(cache_path) {
            return arena;
        }
    }
    findme_ui::engine::EngineBridge::default_sample_arena()
}

/// Executes a search using `CliOptions` against the provided or loaded `FileArena`
pub fn search_arena(opts: &CliOptions, arena: &findme_core::FileArena) -> Vec<CliSearchResult> {
    let query_str = opts.query.as_deref().unwrap_or("");
    let ast = QueryParser::parse(query_str);

    let config = SearchConfig {
        max_results: opts.max_results,
        sort_column: SortColumn::Score,
        sort_direction: SortDirection::Descending,
    };

    let match_results = SearchEngine::search(arena, &ast, &config);
    let mut results = Vec::with_capacity(match_results.len());

    for m in &match_results {
        if let Some(record) = arena.get(m.record_idx) {
            if let Some(path) = arena.reconstruct_path(record) {
                let name = arena.get_name(m.record_idx).unwrap_or_default().to_string();
                results.push(CliSearchResult {
                    path: path.to_string_lossy().to_string(),
                    name,
                    size_bytes: record.size_bytes,
                    mtime_secs: record.mtime_secs,
                    is_dir: record.flags.is_dir(),
                    is_network: record.flags.is_network(),
                    score: m.score,
                });
            }
        }
    }

    results
}

/// Executes a headless CLI search and outputs formatted results to stdout
pub fn execute_cli_search(opts: &CliOptions, cache_path: &Path) -> Result<()> {
    let arena = load_or_fallback_arena(cache_path);
    let results = search_arena(opts, &arena);

    if opts.is_json {
        println!("{}", serde_json::to_string_pretty(&results)?);
    } else {
        for res in &results {
            println!("{}", res.path);
        }
    }

    Ok(())
}

/// Outputs index statistics in CLI mode
pub fn execute_cli_stats(cache_path: &Path) {
    let cache_status = if cache_path.exists() {
        "Persistent Cache Present"
    } else {
        "Using Built-in Sample Index"
    };

    let arena = load_or_fallback_arena(cache_path);
    let memory_bytes = arena.memory_usage_bytes();
    let memory_mb = memory_bytes as f64 / (1024.0 * 1024.0);

    println!("Find-Me Index Statistics:");
    println!("  Cache Status:        {}", cache_status);
    println!("  Cache File Path:     {}", cache_path.display());
    println!("  Indexed Files:       {}", arena.len());
    println!("  Indexed Directories: {}", arena.dirs().len());
    println!(
        "  Arena Memory Usage:  {} bytes ({:.2} MB)",
        memory_bytes, memory_mb
    );
}

/// Prints application help message to stdout
pub fn print_help() {
    println!(
        r#"Find-Me v{} — Instant Linux Desktop File Search Utility

USAGE:
    find-me [OPTIONS] [QUERY]

OPTIONS:
    -h, --help                Print help information
    -v, --version             Print version information
    -q, --query <STRING>      Search query expression (e.g. "ext:rs test")
    --cli, --search           Run in headless command-line search mode
    --gui                     Explicitly launch graphical desktop interface (default)
    --json                    Output search results as a JSON array of objects
    -n, --max-results <NUM>   Limit the maximum number of returned results (default: 100)
    --stats                   Display index cache information and metrics
    --rebuild-index           Perform background filesystem index crawl in CLI mode
    --new-window              Open new application search window

SHORTCUTS (GUI):
    Ctrl + F / /              Focus search input bar
    Ctrl + 1 .. 6             Switch navigation screens
    Ctrl + B                  Toggle sidebar expansion
    Ctrl + T                  Toggle dark / light visual theme
    F5 / Ctrl + R             Refresh active query
    Alt + 1 .. 5              Change table sorting column
    Enter                     Open selected file with default desktop app
    Alt + Enter               Open file properties modal dialog
    Ctrl + Shift + C          Copy full POSIX path to clipboard
    Shift + F10               Open right-click context menu
    Escape                    Context-sensitive dismiss hierarchy

FreeDesktop Integration:
    Desktop Launcher:         /usr/share/applications/find-me.desktop
    Scalable Icon:            /usr/share/icons/hicolor/scalable/apps/find-me.svg
    AppStream Metadata:       /usr/share/metainfo/io.github.faizan_nadeem.find_me.metainfo.xml"#,
        findme_core::VERSION
    );
}

/// Prints application version string to stdout
pub fn print_version() {
    println!("find-me {}", findme_core::VERSION);
}
