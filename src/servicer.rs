use crate::Server;
/// Utils for the server, whether on a remote machine (SSH), or an adjacent thread for local transport.
use anyhow::Result;
use std::path::PathBuf;

pub struct Servicer<'a, T>
where
    T: Server,
{
    filepath: Option<PathBuf>,
    server: &'a mut T,
}

impl<'a, T> Servicer<'a, T>
where
    T: Server,
{
    pub fn new(server: &mut T) -> Servicer<T> {
        Servicer {
            filepath: None,
            server,
        }
    }

    pub fn handle(&mut self) -> Result<()> {
        let mut quit = false;
        while quit == false {
            let request = self.server.receive()?;

            let _handle_lookup = match String::from_utf8(request)?.as_str() {
                "SYN" => self.send_str_response("ACK"),
                "hello" => self.send_str_response("Hey, hows it going?"),
                "filepath" => self.receive_filepath(),
                "shutdown" => {
                    quit = true;
                    self.send_str_response("Shutting down!")
                }
                _ => self.send_str_response("Dang, I quite didn't catch that."),
            };
        }

        Ok(())
    }

    fn send_str_response(&mut self, response: &str) -> Result<()> {
        self.server.send(response.into())?;
        Ok(())
    }

    fn receive_filepath(&mut self) -> Result<()> {
        Ok(())
    }

    fn calculate_signature(&mut self) -> Result<()> {
        Ok(())
    }

    fn apply_delta(&self, delta: Vec<u8>) -> Result<()> {
        Ok(())
    }
}
