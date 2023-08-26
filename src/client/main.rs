mod sync;

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
fn main() {
    // parse command line args
    println!("Running client!");
    sync::sync(PathBuf::from(
        r"/home/knara/dev/rust/rsync-rs/logs/output.txt",
    ))
    .unwrap();
}
