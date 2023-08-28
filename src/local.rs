//! Provides structs/APIs for creating local sync clients and servers.
//! Client and server communicate through mpsc message passing channel.

use super::servicer::Servicer;
use super::{Client, Server};
use anyhow::{anyhow, Result};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

pub struct LocalClient {
    pub server_t: Option<thread::JoinHandle<()>>,
    pub p_send: Option<Sender<Vec<u8>>>,
    pub p_recv: Option<Receiver<Vec<u8>>>,
}

impl LocalClient {
    pub fn new() -> LocalClient {
        LocalClient {
            server_t: None,
            p_send: None,
            p_recv: None,
        }
    }
}

impl Client for LocalClient {
    fn create_connection(&mut self) -> Result<()> {
        let (p_send, c_recv) = mpsc::channel::<Vec<u8>>();
        let (c_send, p_recv) = mpsc::channel::<Vec<u8>>();

        let server_t = thread::spawn(move || {
            let mut local_server = LocalServer::new(c_send, c_recv);
            local_server.run().unwrap();
        });

        self.server_t = Some(server_t);
        self.p_send = Some(p_send);
        self.p_recv = Some(p_recv);

        Ok(())
    }

    fn request(&mut self, request: Vec<u8>) -> Result<Vec<u8>> {
        match (&self.p_send, &self.p_recv) {
            (Some(p_send), Some(p_recv)) => {
                println!("[Client] Sending: {}", String::from_utf8(request.clone())?);
                p_send.send(request)?;
                let response = p_recv.recv_timeout(Duration::from_secs(10))?;
                println!(
                    "[Client] Got response: {}",
                    String::from_utf8(response.clone())
                        .unwrap_or("Response can't be decoded".to_string())
                );
                return Ok(response);
            }
            _ => return Err(anyhow!("Sender/receiver not initialized")),
        }
    }
}

struct LocalServer {
    c_send: Sender<Vec<u8>>,
    c_recv: Receiver<Vec<u8>>,
}

impl LocalServer {
    pub fn new(c_send: Sender<Vec<u8>>, c_recv: Receiver<Vec<u8>>) -> LocalServer {
        LocalServer { c_send, c_recv }
    }
}

impl Server for LocalServer {
    fn run(&mut self) -> Result<()> {
        let mut servicer = Servicer::new(self);
        servicer.handle()?;
        println!("Finished handling connection");
        Ok(())
    }

    fn send(&mut self, response: Vec<u8>) -> Result<()> {
        self.c_send.send(response)?;
        Ok(())
    }

    fn receive(&mut self) -> Result<Vec<u8>> {
        Ok(self.c_recv.recv_timeout(Duration::from_secs(5))?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_server_request_shutdown() {
        let mut local = LocalClient::new();
        local.create_connection().unwrap();
        assert_eq!(
            "Shutting down!",
            String::from_utf8(local.request("shutdown".into()).unwrap()).unwrap()
        );
        assert!(local.request("hello".into()).is_err());
    }

    #[test]
    fn test_local_server_request_ack() {
        let mut local = LocalClient::new();
        local.create_connection().unwrap();
        assert_eq!(
            "ACK",
            String::from_utf8(local.request("SYN".into()).unwrap()).unwrap()
        );
        assert_eq!(
            "Shutting down!",
            String::from_utf8(local.request("shutdown".into()).unwrap()).unwrap()
        );
    }
}
