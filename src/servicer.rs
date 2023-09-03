//! Utils for the server, whether on a remote machine (SSH), or an adjacent thread for local transport.

use crate::not_rsync_pb::*;
use crate::server::Server;
use anyhow::Result;
use fast_rsync::{apply, Signature, SignatureOptions};
use prost::Message;
use std::fs::{File, OpenOptions};
use std::io::{prelude::*, SeekFrom};

pub struct Servicer<'a, T>
where
    T: Server,
{
    server: &'a mut T,
}

impl<'a, T> Servicer<'a, T>
where
    T: Server,
{
    pub fn new(server: &mut T) -> Servicer<T> {
        Servicer { server }
    }

    pub fn handle(&mut self) -> Result<()> {
        println!("proceeding to handler");
        let mut shutdown: bool = false;
        while !shutdown {
            let req: ClientMessage = ClientMessage::decode(self.server.receive()?.as_slice())?;

            match req.message {
                None => {
                    println!("Didn't receive a message type");
                    shutdown = true;
                    let res = ShutdownResponse {
                        error: Some("Couldn't deduce message type".to_string()),
                    };
                    self.server.send(res.encode_to_vec())?;
                }
                Some(client_message::Message::SignatureRequest(req)) => {
                    println!("Signature request");
                    let res = self.get_file_signatures(req)?;
                    self.server.send(res.encode_to_vec())?;
                }
                Some(client_message::Message::PatchRequest(req)) => {
                    println!("Patch request");
                    let res = self.patch_files(req)?;
                    self.server.send(res.encode_to_vec())?;
                }
                Some(client_message::Message::ShutdownRequest(_)) => {
                    println!("Shutdown request");
                    shutdown = true;
                    self.server
                        .send(ShutdownResponse { error: None }.encode_to_vec())?;
                }
            };
        }

        Ok(())
    }

    fn get_file_signatures(&self, req: SignatureRequest) -> Result<SignatureResponse> {
        let mut res = SignatureResponse::default();
        // parallelize
        for fp in req.filepaths.into_iter() {
            println!("Calculating signature for {}", fp);
            let mut sig: FileSignature = FileSignature::default();
            sig.filepath = fp.clone();
            sig.content = Self::calculate_signature(fp, None)?.serialized().to_vec();
            res.signatures.push(sig);
        }

        Ok(res)
    }

    // make this atomic
    fn patch_files(&self, req: PatchRequest) -> Result<PatchResponse> {
        for delta in req.deltas.into_iter() {
            Self::patch(delta.filepath, delta.content)?;
        }

        return Ok(PatchResponse { ok: true });
    }

    fn calculate_signature(
        filepath: String,
        sig_opts: Option<SignatureOptions>,
    ) -> Result<Signature> {
        let sig_opts = match sig_opts {
            None => SignatureOptions {
                block_size: 4,
                crypto_hash_size: 8,
            },
            Some(opts) => opts,
        };

        let mut file_bytes: Vec<u8> = Vec::new();

        let mut file = match File::open(filepath) {
            Ok(file) => file,
            Err(e) => {
                eprintln!("{} -- file doesn't exist. sending an empty signature", e);
                return Ok(Signature::calculate(&file_bytes, sig_opts));
            }
        };

        file.read_to_end(&mut file_bytes)?;
        return Ok(Signature::calculate(&file_bytes, sig_opts));
    }

    fn patch(filepath: String, delta: Vec<u8>) -> Result<()> {
        let mut patched_out = vec![];

        println!("Attempting to patch {} with {:?}", filepath, delta);
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(false)
            .open(filepath)?;
        let mut file_bytes: Vec<u8> = Vec::new();
        file.read_to_end(&mut file_bytes)?;
        apply(&file_bytes, &delta, &mut patched_out)?;
        println!("{:?}", String::from_utf8(patched_out.clone()));

        file.seek(SeekFrom::Start(0))?;
        file.set_len(0)
            .unwrap_or_else(|e| println!("some whack error{e}"));
        file.write_all(&patched_out)
            .unwrap_or_else(|e| println!("couldn't write to file: {e}"));

        Ok(())
    }
}

#[cfg(test)]
mod servicer_tests {
    // TODO: add servicer tests
}
