use crate::not_rsync_pb::ClientMessage;
use anyhow::Result;
use prost::Message;

pub trait Client {
    /// create a connection to a local or remote server
    fn create_connection(&mut self) -> Result<()>;
    /// make a request to the server and receive a response
    fn request_from_bytes(&mut self, request: Vec<u8>) -> Result<Vec<u8>>;
    fn request_from_proto(&mut self, request: ClientMessage) -> Result<Vec<u8>> {
        self.request_from_bytes(request.encode_to_vec())
    }
}
