use anyhow::Result;
use clap::Parser;
use not_rsync::sync::{self, Location};
use std::path::PathBuf;

/// arg might look like
/// not_rsync knara@localhost:src/
///
/// rsync ... SRC ... [USER@]HOST:DEST # synchronize a remote file with local
/// rsync ... [USER@]HOST:SRC ... DEST # synchronize a local file with remote

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
    let args = Args::parse();

    // check if source or path is dir
    // can't sync a file to a directionary
    // can sync a file a file to a file
    // can sync a directory to a directory

    sync::sync(Location::from_arg(args.src), Location::from_arg(args.dest))?;

    Ok(())
}
