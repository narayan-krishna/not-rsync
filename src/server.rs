use anyhow::Result;

pub trait Server {
    /// receive a request from the client and send a response
    fn run(&mut self) -> Result<()>;
    fn receive(&mut self) -> Result<Vec<u8>>;
    fn send(&mut self, response: Vec<u8>) -> Result<()>;
}
