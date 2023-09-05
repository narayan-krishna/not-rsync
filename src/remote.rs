use crate::{client::Client, *};
use crate::{server::Server, servicer::Servicer};
use anyhow::{anyhow, Result};
use log::{info, warn};
use ssh2::{Channel, Session};
use std::net::TcpStream;

const LAUNCH_CMD: &str = "cd /home/knara/dev/rust/not-rsync/ && RUST_LOG=info cargo run --bin remote_server 2> /home/knara/dev/rust/not-rsync/logs/remote_server.txt &";
const SERVER_PORT: u16 = 50051;

pub struct RemoteClient {
    session_channel: Option<Channel>,
    forwarding_channel: Option<Channel>,
    server_pid: Option<u32>,
    username: String,
    hostname: String,
}

impl RemoteClient {
    pub fn new(username: String, hostname: String) -> RemoteClient {
        RemoteClient {
            session_channel: None,
            forwarding_channel: None,
            server_pid: None,
            username,
            hostname,
        }
    }

    fn start_ssh_session(&self) -> Result<Session> {
        info!("attempting to start ssh session");
        let mut sess: Session = Session::new()?;

        let tcp = TcpStream::connect(format!("{}:22", self.hostname)).unwrap();
        sess.set_tcp_stream(tcp);
        sess.handshake().unwrap();

        sess.userauth_agent(&self.username)?; // TODO: automatically determine remote username
        assert!(sess.authenticated());
        info!("session authenticated");

        Ok(sess)
    }
}

impl Client for RemoteClient {
    fn create_connection(&mut self) -> Result<()> {
        let sess = self.start_ssh_session()?;
        info!("launching server!");
        self.session_channel = Some(sess.channel_session()?);

        if let Some(session_channel) = &mut self.session_channel {
            session_channel.exec(LAUNCH_CMD)?;

            info!("checing server pid");
            let mut server_ack_pid: &mut [u8] = &mut [0; 5];
            session_channel.read_exact(&mut server_ack_pid)?;
            let server_pid = String::from_utf8_lossy(&server_ack_pid)
                .parse::<u32>()
                .map(|s| Some(s))
                .unwrap_or(None);

            self.server_pid = server_pid;
            match self.server_pid.map(|pid| pid.to_string()) {
                Some(pid) => info!("Server is running at {} with pid {}", SERVER_PORT, pid),
                None => warn!("Server is running at {}, UNABLE TO DECODE PID", SERVER_PORT),
            }

            self.forwarding_channel =
                Some(sess.channel_direct_tcpip("localhost", SERVER_PORT, None)?);
        }
        Ok(())
    }

    /// Get response from the server by sending a request
    fn request_from_bytes(&mut self, request: Vec<u8>) -> Result<Vec<u8>> {
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
        info!("attempting to handle connection...");

        let mut servicer = Servicer::new(self);
        servicer.handle()?;

        info!("finished handling connection");
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

    fn setup_local_ssh_connection() -> RemoteClient {
        let remote = RemoteClient::new("knara".to_string(), "localhost".to_string());
        remote
    }

    #[test]
    fn test_remote_server_request_shutdown() {
        let mut remote = setup_local_ssh_connection();
        remote.create_connection().unwrap();
        assert_eq!(
            "Shutting down!",
            String::from_utf8(remote.request_from_bytes("shutdown".into()).unwrap()).unwrap()
        );
        assert!(remote.request_from_bytes("hello".into()).is_err());
    }

    #[test]
    fn test_remote_server_request_ack() {
        let mut remote = setup_local_ssh_connection();
        remote.create_connection().unwrap();
        assert_eq!(
            "ACK",
            String::from_utf8(remote.request_from_bytes("SYN".into()).unwrap()).unwrap()
        );
        assert_eq!(
            "Shutting down!",
            String::from_utf8(remote.request_from_bytes("shutdown".into()).unwrap()).unwrap()
        );
    }
}
