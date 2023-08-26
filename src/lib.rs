use std::error::Error;
use std::io::prelude::*;

pub fn read_message_len_header<T>(buf: &mut T) -> Result<u32, Box<dyn Error>>
where
    T: Read,
{
    let mut request_len_bytes: [u8; 4] = [0u8; 4];
    buf.read_exact(&mut request_len_bytes)?;
    let request_len = u32::from_be_bytes(request_len_bytes);
    println!("[Received] Request length of {} bytes", request_len);

    Ok(request_len)
}

pub fn read_message<T>(buf: &mut T, message_len: usize) -> Result<String, Box<dyn Error>>
where
    T: Read,
{
    let mut read_buf = vec![0u8; message_len];
    buf.read_exact(&mut read_buf)?;
    let message: String = String::from_utf8(read_buf)?;

    Ok(message)
}

pub fn write_message_len<T>(buf: &mut T, message: &str) -> Result<(), Box<dyn Error>>
where
    T: Write,
{
    let bytes = message.as_bytes();
    buf.write(&(bytes.len() as u32).to_be_bytes())?;
    println!("Sending {}", bytes.len() as u32);

    Ok(())
}

pub fn write_message<T>(buf: &mut T, message: &str) -> Result<(), Box<dyn Error>>
where
    T: Write,
{
    buf.write(message.as_bytes())?;
    Ok(())
}
