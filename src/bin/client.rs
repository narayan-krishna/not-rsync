use anyhow::Result;
use clap::Parser;
use env_logger::{Builder, Target};
use log::info;
use not_rsync::sync::{self, Location};

// arg might look like
// not_rsync knara@localhost:src/
//
// rsync ... SRC ... [USER@]HOST:DEST # synchronize a remote file with local
// rsync ... [USER@]HOST:SRC ... DEST # synchronize a local file with remote

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    src: String,
    #[arg(short, long)]
    dest: String,
    #[arg(long)]
    ssh: bool,
    #[arg(short, long)]
    verbose: bool,
}

/// Runs the not_rsync client for syncing a file to a server.
fn main() -> Result<()> {
    // parse command line args
    let args = Args::parse();

    let mut builder = Builder::from_default_env();
    builder
        .target(if args.verbose {
            Target::Stdout
        } else {
            Target::Stderr
        })
        .init();

    info!("starting client!");
    // check if source or path is dir
    // can't sync a file to a directionary
    // can sync a file a file to a file
    // can sync a directory to a directory

    sync::sync(
        Location::from_arg(args.src),
        Location::from_arg(args.dest),
        args.ssh,
    )?;

    Ok(())
}
