use anyhow::Result;

pub trait Client {
    /// create a connection to a local or remote server
    fn create_connection(&mut self) -> Result<()>;
    /// make a request to the server and receive a response
    fn request(&mut self, request: Vec<u8>) -> Result<Vec<u8>>;
}
