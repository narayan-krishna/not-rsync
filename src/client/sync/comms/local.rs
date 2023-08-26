use super::Comms;
use std::error::Error;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

pub struct Local {
    pub server_t: Option<thread::JoinHandle<()>>,
    pub sender: Option<Sender<String>>,
    pub receiver: Option<Receiver<String>>,
}

impl Local {
    pub fn init() -> Local {
        Local {
            server_t: None,
            sender: None,
            receiver: None,
        }
    }

    pub fn run_local_server(sender: Sender<String>, receiver: Receiver<String>) {
        let mut quit: bool = true;
        while quit == false {
            let request: String = receiver.recv_timeout(Duration::from_secs(5)).unwrap();

            let response = match request.as_str() {
                "shutdown" => {
                    quit = true;
                    "Shutting down!"
                }
                "hello" => "Hey, hows it going?",
                _ => "Dang, I quite didn't catch that.",
            }
            .to_string();

            sender.send(response).unwrap();
        }
    }
}

impl Comms for Local {
    fn create_connection(&mut self) -> Result<(), Box<dyn Error>> {
        let (p_send, c_recv) = mpsc::channel::<String>();
        let (c_send, p_recv) = mpsc::channel::<String>();

        let server_t = thread::spawn(move || Self::run_local_server(c_send, p_recv));

        self.server_t = Some(server_t);
        self.sender = Some(p_send);
        self.receiver = Some(c_recv);

        Ok(())
    }

    fn request(&mut self, request: &str) -> Result<String, Box<dyn Error>> {
        match (&self.sender, &self.receiver) {
            (Some(sender), Some(receiver)) => {
                sender.send(request.to_string())?;
                let response = receiver.recv_timeout(Duration::from_secs(5))?;

                return Ok(response);
            }
            _ => return Err("Sender/receiver not initialized".into()),
        }
    }
}
