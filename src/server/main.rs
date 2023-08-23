use std::error::Error;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

const PORT: u16 = 50051;

fn main() {
    let tcp_listener = TcpListener::bind(format!("localhost:{}", PORT)).unwrap();
    println!("[Server main] Server is up and running at {}!", PORT);
    let (send, recv) = mpsc::channel();

    let listen = thread::spawn(move || {
        match tcp_listener.accept() {
            Ok((stream, addr)) => {
                eprintln!("Received a connection from {}!", addr);
                send.send(()).unwrap();
                handle_connection(stream).unwrap();
            }
            Err(e) => eprintln!("Connection failed! ({})", e),
        };
    });

    thread::sleep(Duration::from_secs(5));

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

    while quit == false {
        let read_buf: &mut [u8] = &mut [0; 1024];
        stream.read(read_buf)?;
        let request: String = String::from_utf8_lossy(read_buf)
            .trim_matches(char::from(0))
            .to_string();
        println!(
            "[Connection handler] Received: {:?} (len: {})",
            request,
            request.as_str().len()
        );

        let response = match request.as_str() {
            "shutdown" => {
                quit = true;
                "Shutting down!"
            }
            "hello" => "Hey, hows it going?",
            _ => "Dang, I quite didn't catch that.",
        }
        .to_string();

        stream.write_all(response.as_bytes())?;
    }

    println!("Finished handling connection");
    Ok(())
}
