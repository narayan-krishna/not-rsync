pub mod client;
pub mod local;
pub mod remote;
pub mod server;
pub mod servicer;
pub mod sync;

use anyhow::Result;
use bytes::Buf;
use log::trace;
use std::io::prelude::*;

pub mod not_rsync_pb {
    include!(concat!(env!("OUT_DIR"), "/notrsync.rs"));
}

/// Proto trait provides utilties for working with protobufs.
trait AsProto<T, U>: Buf
where
    U: prost::Message + Default,
{
    /// Decodes a buffer
    fn as_proto(&mut self) -> Result<U> {
        let decode_res = U::decode(self)?;
        trace!("decoded as proto: {:?}", decode_res);
        Ok(decode_res)
    }
}

impl<T: Buf> AsProto<T, not_rsync_pb::SignatureResponse> for T {}
impl<T: Buf> AsProto<T, not_rsync_pb::PatchResponse> for T {}
impl<T: Buf> AsProto<T, not_rsync_pb::ShutdownResponse> for T {}

pub fn read_message_len_header<T>(buf: &mut T) -> Result<u32>
where
    T: Read,
{
    let mut request_len_bytes: [u8; 4] = [0u8; 4];
    buf.read_exact(&mut request_len_bytes)?;
    let request_len = u32::from_be_bytes(request_len_bytes);
    trace!("received request length: {} bytes", request_len);

    Ok(request_len)
}

pub fn read_message<T>(buf: &mut T, message_len: usize) -> Result<Vec<u8>>
where
    T: Read,
{
    let mut read_buf = vec![0u8; message_len];
    buf.read_exact(&mut read_buf)?;

    Ok(read_buf)
}

pub fn write_message_len<T>(buf: &mut T, message: &Vec<u8>) -> Result<()>
where
    T: Write,
{
    buf.write(&(message.len() as u32).to_be_bytes())?;
    trace!("sending: {}", message.len() as u32);

    Ok(())
}

pub fn write_message<T>(buf: &mut T, message: Vec<u8>) -> Result<()>
where
    T: Write,
{
    buf.write(&message)?;
    Ok(())
}
