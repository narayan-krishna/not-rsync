use clap::Parser;
mod sync;

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
    sync::sync().unwrap();
}

