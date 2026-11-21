//! Find-Me CLI entry point and headless harness

use anyhow::Result;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    info!("Starting Find-Me v{}", findme_core::VERSION);
    println!(
        "Find-Me v{} - High-Performance Linux File Search",
        findme_core::VERSION
    );
    println!("Offline-first, native Rust indexing engine.");

    Ok(())
}
