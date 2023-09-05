use chrono;
use log::{error, info, warn};
use not_rsync::remote::RemoteServer;
use not_rsync::server::Server;
use std::net::TcpListener;
use std::process;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

const PORT: u16 = 50051;

fn main() {
    env_logger::init();
    info!("{:?}", chrono::offset::Local::now());
    info!("starting server");

    let tcp_listener = TcpListener::bind(format!("localhost:{}", PORT)).unwrap();
    println!("{:0width$}", process::id(), width = 5); // needed for handshake between client and
                                                      // server
    info!("server is up and running at {}!", PORT);
    let (send, recv) = mpsc::channel();

    let listen = thread::spawn(move || {
        match tcp_listener.accept() {
            Ok((stream, addr)) => {
                info!("received a connection from {}!", addr);
                send.send(()).unwrap();
                let mut server = RemoteServer::new(stream);
                server.run().unwrap();
            }
            Err(e) => error!("connection failed! ({})", e),
        };
    });

    thread::sleep(Duration::from_secs(1));

    match recv.try_recv() {
        Ok(_) => {
            info!("found a connection after the timouet. nice!");
            listen.join().unwrap();
        }
        Err(e) => {
            warn!("listener timed out: {e}. Dropping listener.");
            drop(listen);
        }
    }

    info!("shutting down")
}
