use anyhow::Result;
use clap::Parser;
use not_rsync::sync::{self, Location};
use std::path::PathBuf;

/// arg might look like
/// not_rsync knara@localhost:src/
///

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    src: String,
    #[arg(short, long)]
    dest: String,
}

/// Runs the not_rsync client for syncing a file to a server.
fn main() -> Result<()> {
    // parse command line args
    println!("Running client!");
    // let args = Args::parse();

    sync::sync(
        Location::new(
            "knara",
            "localhost",
            "dev/rust/not-rsync/tests/test_files/base.txt",
        ),
        Location::new(
            "knara",
            "localhost",
            "dev/rust/not-rsync/tests/test_files/modified.txt",
        ),
    )?;

    Ok(())
}
