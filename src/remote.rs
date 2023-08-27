/// Provides structs/APIs for creating remote sync clients and servers.
/// Client and server communicate through SSH session/port forwarding.
use super::{servicer::Servicer, *};
use anyhow::{anyhow, Result};
use ssh2::{Channel, Session};
use std::io::prelude::*;
use std::net::TcpStream;

const LAUNCH_CMD: &str = "cd /home/knara/dev/rust/rsync-rs/ && cargo run --bin server | tee /home/knara/dev/rust/rsync-rs/logs/output.txt &";
const SERVER_PORT: u16 = 50051;

pub struct RemoteClient {
    session_channel: Option<Channel>,
    forwarding_channel: Option<Channel>,
    server_pid: Option<u32>,
}

impl RemoteClient {
    pub fn new() -> RemoteClient {
        RemoteClient {
            session_channel: None,
            forwarding_channel: None,
            server_pid: None,
        }
    }

    fn start_ssh_session() -> Result<Session> {
        println!("Attempting to start ssh session");
        let mut sess: Session = Session::new()?;

        let tcp = TcpStream::connect("localhost:22").unwrap();
        sess.set_tcp_stream(tcp);
        sess.handshake().unwrap();

        sess.userauth_agent("knara")?; // TODO: automatically determine remote username
        assert!(sess.authenticated());
        println!("Session authenticated");

        Ok(sess)
    }
}

impl Client for RemoteClient {
    /// Run the remote server and communicate with it.
    fn create_connection(&mut self) -> Result<()> {
        let sess = Self::start_ssh_session()?;
        println!("Launching server!");
        self.session_channel = Some(sess.channel_session()?);

        if let Some(session_channel) = &mut self.session_channel {
            session_channel.exec(LAUNCH_CMD)?;

            let mut server_ack_pid: &mut [u8] = &mut [0; 5];
            session_channel.read_exact(&mut server_ack_pid)?;
            let server_pid = String::from_utf8_lossy(&server_ack_pid).parse::<u32>()?;

            self.server_pid = Some(server_pid);
            println!(
                "Server is running at {} with pid {}",
                SERVER_PORT,
                self.server_pid.unwrap()
            );

            self.forwarding_channel =
                Some(sess.channel_direct_tcpip("localhost", SERVER_PORT, None)?);
        }
        Ok(())
    }

    /// Get response from the server by sending a request
    fn request(&mut self, request: Vec<u8>) -> Result<Vec<u8>> {
        if let Some(channel) = &mut self.forwarding_channel {
            write_message_len(channel, &request)?;
            write_message(channel, request)?;
            let response_len = read_message_len_header(channel)?;
            return Ok(read_message(channel, response_len as usize)?);
        }

        Err(anyhow!("connection not established"))
    }
}

pub struct RemoteServer {
    tcp_stream: TcpStream,
}

impl RemoteServer {
    pub fn new(tcp_stream: TcpStream) -> RemoteServer {
        RemoteServer { tcp_stream }
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
        let request_len = read_message_len_header(&mut self.tcp_stream)?;
        let request = read_message(&mut self.tcp_stream, request_len as usize)?;

        Ok(request)
    }

    fn send(&mut self, response: Vec<u8>) -> Result<()> {
        write_message_len(&mut self.tcp_stream, &response)?;
        write_message(&mut self.tcp_stream, response)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_remote_server_request_shutdown() {
        let mut remote = RemoteClient::new();
        remote.create_connection().unwrap();
        assert_eq!(
            "Shutting down!",
            String::from_utf8(remote.request("shutdown".into()).unwrap()).unwrap()
        );
        assert!(remote.request("hello".into()).is_err());
    }

    #[test]
    fn test_remote_server_request_ack() {
        let mut remote = RemoteClient::new();
        remote.create_connection().unwrap();
        assert_eq!(
            "ACK",
            String::from_utf8(remote.request("SYN".into()).unwrap()).unwrap()
        );
        assert_eq!(
            "Shutting down!",
            String::from_utf8(remote.request("shutdown".into()).unwrap()).unwrap()
        );
    }
}
