use not_rsync::remote::RemoteServer;
use not_rsync::server::Server;
use std::net::TcpListener;
use std::process;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

const PORT: u16 = 50051;

fn main() {
    let tcp_listener = TcpListener::bind(format!("localhost:{}", PORT)).unwrap();
    println!("{:0width$}", process::id(), width = 5);
    println!("[Server main] Server is up and running at {}!", PORT);
    let (send, recv) = mpsc::channel();

    let listen = thread::spawn(move || {
        match tcp_listener.accept() {
            Ok((stream, addr)) => {
                println!("Received a connection from {}!", addr);
                send.send(()).unwrap();
                let mut server = RemoteServer::new(stream);
                server.run().unwrap();
            }
            Err(e) => println!("Connection failed! ({})", e),
        };
    });

    thread::sleep(Duration::from_secs(8));

    match recv.try_recv() {
        Ok(_) => {
            println!("[Server main] Found a connection after the timouet. Nice!");
            listen.join().unwrap();
        }
        Err(e) => {
            println!("[Server main] Listener timed out: {e}. Dropping listener.");
            drop(listen);
        }
    }

    println!("Shutting down.")
}
