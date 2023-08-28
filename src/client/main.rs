mod sync;

use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    src: String,
    #[arg(short, long)]
    dest: String,
}

/// Runs the rsync-rs client for syncing a file to a server.
fn main() -> Result<()> {
    // parse command line args
    println!("Running client!");
    sync::sync(
        PathBuf::from(r"/home/knara/dev/rust/rsync-rs/tests/base.txt"),
        PathBuf::from(r"/home/knara/dev/rust/rsync-rs/tests/modified.txt"),
    )?;

    Ok(())
}
