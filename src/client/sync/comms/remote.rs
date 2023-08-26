use ssh2::{Channel, Session};
use std::error::Error;
use std::io::prelude::*;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

use super::Comms;

const LAUNCH_CMD: &str = "cd /home/knara/dev/rust/rsync-rs/ && cargo run --bin server | tee /home/knara/dev/rust/rsync-rs/logs/output.txt &";
const SERVER_PORT: u16 = 50051;

pub struct Remote {
    session_channel: Option<Channel>,
    forwarding_channel: Option<Channel>,
    server_pid: Option<u32>,
}

impl Remote {
    pub fn init() -> Remote {
        Remote {
            session_channel: None,
            forwarding_channel: None,
            server_pid: None,
        }
    }

    fn start_ssh_session() -> Result<Session, Box<dyn Error>> {
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

impl Comms for Remote {
    /// Run the remote server and communicate with it.
    fn create_connection(&mut self) -> Result<(), Box<dyn Error>> {
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
    fn request(&mut self, request: &str) -> Result<String, Box<dyn Error>> {
        if let Some(channel) = &mut self.forwarding_channel {
            rsync_rs::write_message_len(channel, request)?;
            rsync_rs::write_message(channel, request)?;
            let response_len = rsync_rs::read_message_len_header(channel)?;
            return Ok(rsync_rs::read_message(channel, response_len as usize)?);
        }

        Err("connection not established".into())
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_remote_server_request_shutdown() {
        let mut remote = Remote::init();
        remote.create_connection().unwrap();
        assert_eq!("Shutting down!", remote.request("shutdown").unwrap());
        assert!(remote.request("hello").is_err());
    }

    #[test]
    fn test_remote_server_request_ack() {
        let mut remote = Remote::init();
        remote.create_connection().unwrap();
        assert_eq!("ACK", remote.request("SYN").unwrap());
        assert_eq!("Shutting down!", remote.request("shutdown").unwrap());
    }
}
