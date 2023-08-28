//! Utils for the server, whether on a remote machine (SSH), or an adjacent thread for local transport.

use super::Server;
use anyhow::{anyhow, Result};
use fast_rsync::{apply, Signature, SignatureOptions};
use std::fs::{File, OpenOptions};
use std::io::{prelude::*, SeekFrom};
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

            // TODO: make this functions promise to return a response
            match String::from_utf8(request)?.as_str() {
                "SYN" => self.send_str_response("ACK"),
                "hello" => self.send_str_response("Hey, hows it going?"),
                "filepath" => self.receive_filepath(),
                "signature" => self.calculate_signature(),
                "patch" => self.apply_patch(),
                "shutdown" => {
                    quit = true;
                    self.send_str_response("Shutting down!")
                }
                _ => self.send_str_response("Dang, I quite didn't catch that."),
            }?;
        }

        Ok(())
    }

    fn send_str_response(&mut self, response: &str) -> Result<()> {
        self.server.send(response.into())?;
        Ok(())
    }

    fn receive_filepath(&mut self) -> Result<()> {
        self.send_str_response("ready for filepath")?;
        let filepath = self.server.receive()?;
        self.filepath = Some(String::from_utf8(filepath)?.into());
        self.server.send(
            format!(
                "received {}",
                self.filepath.clone().unwrap().to_str().unwrap()
            )
            .into(),
        )?;

        Ok(())
    }

    fn calculate_signature(&mut self) -> Result<()> {
        if let Some(filepath) = &self.filepath {
            let mut file = match File::open(filepath) {
                Err(e) => {
                    self.send_str_response(format!("Couldn't open from filepath {}", e).as_str())?;
                    return Err(anyhow!("Error: {}", e));
                }
                Ok(file) => file,
            };

            let mut file_bytes: Vec<u8> = Vec::new();
            match file.read_to_end(&mut file_bytes) {
                Err(e) => {
                    self.send_str_response(format!("Couldn't open from filepath {}", e).as_str())?;
                    return Err(anyhow!("Error: {}", e));
                }
                Ok(_) => {
                    let signature_options = SignatureOptions {
                        block_size: 4,
                        crypto_hash_size: 8,
                    };
                    let response_signature = Signature::calculate(&file_bytes, signature_options);
                    self.server
                        .send(Signature::serialized(&response_signature).to_vec())?;
                }
            };
        }

        self.send_str_response("failed")?;
        Ok(())
    }

    fn apply_patch(&mut self) -> Result<()> {
        self.send_str_response("ready for patch")?;
        let patch = self.server.receive()?;
        self.send_str_response("received patch")?;
        let mut patched_out = vec![];

        if let Some(filepath) = &self.filepath {
            let mut file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(false)
                .open(filepath)?;
            let mut file_bytes: Vec<u8> = Vec::new();
            file.read_to_end(&mut file_bytes)?;
            apply(&file_bytes, &patch, &mut patched_out)?;

            file.seek(SeekFrom::Start(0))?;
            file.set_len(0)
                .unwrap_or_else(|e| println!("some whack error{e}"));
            file.write_all(&patched_out)
                .unwrap_or_else(|e| println!("couldn't write to file: {e}"));
        }

        Ok(())
    }
}
