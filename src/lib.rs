pub mod client;
pub mod server;
pub mod remote;
pub mod local;
pub mod servicer;
pub mod sync;

use anyhow::Result;
use std::io::prelude::*;

pub fn read_message_len_header<T>(buf: &mut T) -> Result<u32>
where
    T: Read,
{
    let mut request_len_bytes: [u8; 4] = [0u8; 4];
    buf.read_exact(&mut request_len_bytes)?;
    let request_len = u32::from_be_bytes(request_len_bytes);
    println!("[Received] Request length of {} bytes", request_len);

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
    println!("Sending {}", message.len() as u32);

    Ok(())
}

pub fn write_message<T>(buf: &mut T, message: Vec<u8>) -> Result<()>
where
    T: Write,
{
    buf.write(&message)?;
    Ok(())
}
