use std::error::Error;
use std::io::prelude::*;
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
                handle_connection(stream).unwrap();
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

fn handle_connection(mut stream: TcpStream) -> Result<(), Box<dyn Error>> {
    let mut quit = false;
    println!("Attempting to handle connection...");

    while quit == false {
        println!("Waiting for request message length.");
        let request_len = rsync_rs::read_message_len_header(&mut stream)?;
        let request = rsync_rs::read_message(&mut stream, request_len as usize)?;
        println!("[Received] Request: {}", request);

        let response = match request.as_str() {
            "SYN" => "ACK",
            "shutdown" => {
                quit = true;
                "Shutting down!"
            }
            "hello" => "Hey, hows it going?",
            _ => "Dang, I quite didn't catch that.",
        };

        rsync_rs::write_message_len(&mut stream, response)?;
        rsync_rs::write_message(&mut stream, response)?;
    }

    println!("Finished handling connection");
    Ok(())
}
