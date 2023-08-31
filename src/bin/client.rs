use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;
use rsync_rs::sync::{self, Location};

/// arg might look like
/// rsync-rs knara@localhost:src/
///

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
    // let args = Args::parse();

    sync::sync(
        Location::new(
            "knara",
            "localhost",
            "dev/rust/rsync-rs/tests/test_files/base.txt",
        ),
        Location::new(
            "knara",
            "localhost",
            "dev/rust/rsync-rs/tests/test_files/modified.txt",
        ),
    )?;

    Ok(())
}
