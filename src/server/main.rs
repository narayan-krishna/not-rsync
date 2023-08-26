use anyhow::Result;
use rsync_rs::servicer::Servicer;
use rsync_rs::Server;
use std::net::{TcpListener, TcpStream};
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

struct RemoteServer {
    tcp_stream: TcpStream,
}

impl RemoteServer {
    pub fn new(tcp_stream: TcpStream) -> RemoteServer {
        RemoteServer {
            tcp_stream,
        }
    }
}

impl Server for RemoteServer {
    fn run(&mut self) -> Result<()> {
        println!("Attempting to handle connection...");

        let mut servicer = Servicer::new(self);
        servicer.handle()?;

        println!("Finished handling connection");
        Ok(())
    }

    fn receive(&mut self) -> Result<Vec<u8>> {
        let request_len = rsync_rs::read_message_len_header(&mut self.tcp_stream)?;
        let request = rsync_rs::read_message(&mut self.tcp_stream, request_len as usize)?;

        Ok(request)
    }

    fn send(&mut self, response: Vec<u8>) -> Result<()> {
        rsync_rs::write_message_len(&mut self.tcp_stream, &response)?;
        rsync_rs::write_message(&mut self.tcp_stream, response)?;

        Ok(())
    }
}

// two servers, a local server and a remote server
// both should provide the same interface for receiving and sending requests, so that
// they can send their first request off to a servicer and the servicer should be able to handle
// everything
//
// i have trait request for comms. request allows universal request from clients specifically
//
// trait:
//  client
//  server
//
// proj hierarchy:
//  client
//  server
//  sync
//  lib.rs
//      - client trait
//      - server trait
//
