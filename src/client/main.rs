use clap::Parser;
use ssh2::{Channel, Session};
use std::error::Error;
use std::io::prelude::*;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long)]
    src: String,
    #[arg(short, long)]
    dest: String,
}

/// Runs the rsync-rs client for syncing a file to a server.
fn main() {
    println!("Running client!");
    let sess = create_ssh_session();
    println!("Session is blocking: {}", sess.is_blocking());
    run_remote(&sess).unwrap();
    // comms(&sess).unwrap();
}

fn create_ssh_session() -> Session {
    let mut sess: Session = Session::new().unwrap();

    let tcp = TcpStream::connect("localhost:22").unwrap();
    sess.set_tcp_stream(tcp);
    sess.handshake().unwrap();

    sess.userauth_agent("knara").unwrap(); // TODO: automatically determine remote username
    assert!(sess.authenticated());

    sess
}

/// Run the remote server and communicate with it.
fn run_remote(sess: &Session) -> Result<(), Box<dyn Error>> {
    println!("Launching server!");
    let mut channel: Channel = sess.channel_session()?;
    channel.exec("cd /home/knara/dev/rust/rsync-rs/ && cargo run --bin server &")?;

    let server_ack: &mut [u8] = &mut [0; 128];
    channel.read(server_ack)?; // receive some arbitary acknowledgement that server is running

    comms(&sess).unwrap();

    Ok(())
}

/// Begin communication with remote process via port forwarding
fn comms(sess: &Session) -> Result<(), Box<dyn Error>> {
    let mut channel: Channel = sess.channel_direct_tcpip("localhost", 50051, None)?;

    println!("[Received] {}\n", server_response("hello", &mut channel)?);
    println!(
        "[Received] {}\n",
        server_response("you suck", &mut channel)?
    );
    // only when server shuts down this sends?
    println!(
        "[Received] {}\n",
        server_response("shutdown", &mut channel)?
    );

    Ok(())
}

fn server_response(request: &str, channel: &mut Channel) -> Result<String, Box<dyn Error>> {
    let read_buf: &mut [u8] = &mut [0; 1024];
    channel.write(request.as_bytes())?;
    channel.read(read_buf)?;

    Ok(String::from_utf8_lossy(read_buf).to_string())
}
