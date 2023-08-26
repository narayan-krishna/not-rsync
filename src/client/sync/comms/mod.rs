use std::error::Error;

pub mod local;
pub mod remote;

pub trait Comms {
    fn create_connection(&mut self) -> Result<(), Box<dyn Error>>;
    fn request(&mut self, request: &str) -> Result<String, Box<dyn Error>>;
}
