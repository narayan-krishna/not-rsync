use crate::client::Client;
use crate::local::LocalClient;
use crate::not_rsync_pb::*;
use crate::remote::RemoteClient;
use crate::to_proto;
use anyhow::{anyhow, Result};
use fast_rsync::{diff, Signature};
use prost::Message;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

enum ServerType {
    Local,
    Remote,
}

pub struct Location {
    username: String,
    hostname: String,
    filepath: PathBuf,
    is_remote: bool,
}

impl Location {
    pub fn new(username: &str, hostname: &str, filepath: &str) -> Location {
        let user_prefix = format!("/home/{}", username);

        Location {
            username: username.to_string(),
            hostname: hostname.to_string(),
            filepath: PathBuf::from(user_prefix).join::<PathBuf>(filepath.into()),
            is_remote: hostname != whoami::hostname() && hostname != "localhost",
        }
    }
}

/// Perform file synchronization operations between client and server.
pub fn sync(src: Location, dest: Location) -> Result<()> {
    println!("remote dest: {}", dest.is_remote);
    println!("full dest filepath: {}", dest.filepath.to_str().unwrap());
    println!("full src filepath: {}", src.filepath.to_str().unwrap());

    let server_type = match dest.is_remote {
        true => ServerType::Remote,
        false => ServerType::Remote,
    };

    let mut client: Box<dyn Client> = match server_type {
        ServerType::Remote => Box::new(RemoteClient::new(dest.username, dest.hostname)),
        ServerType::Local => Box::new(LocalClient::new()),
    };

    let files = vec![dest.filepath];

    client.create_connection()?;

    println!("requesting signatures");
    let sig_res: SignatureResponse = get_signatures(&mut client, files)?;
    let _patch_res: PatchResponse =
        patch_remote_files(&mut client, &src.filepath, sig_res.signatures)?;

    // Ask server to gracefully shutdown
    let _shutdown_response: ShutdownResponse = {
        let client_msg: ClientMessage = ClientMessage {
            message: Some(client_message::Message::ShutdownRequest(ShutdownRequest {})),
        };

        to_proto::<ShutdownResponse>(client.request(client_msg.encode_to_vec())?)?
    };

    Ok(())
}

// parallelize
/// Request that server patch files with deltas
fn get_signatures(client: &mut Box<dyn Client>, files: Vec<PathBuf>) -> Result<SignatureResponse> {
    // TODO: parallelize with rayon
    // Request file signatures from server
    let req: SignatureRequest = SignatureRequest {
        filepaths: files
            .iter()
            .map(|f| f.to_str().unwrap().to_string())
            .collect(),
    };

    let client_msg: ClientMessage = ClientMessage {
        message: Some(client_message::Message::SignatureRequest(req)),
    };

    to_proto::<SignatureResponse>(client.request(client_msg.encode_to_vec())?)
}

// parallelize with rayon
/// Request that server patch files with deltas
fn patch_remote_files(
    client: &mut Box<dyn Client>,
    base_filepath: &PathBuf,
    signatures: Vec<FileSignature>,
) -> Result<PatchResponse> {
    let req: PatchRequest = PatchRequest {
        deltas: signatures
            .into_iter()
            .map(|fs: FileSignature| Delta {
                filepath: fs.filepath.clone(),
                content: calculate_delta(
                    &base_filepath,
                    Signature::deserialize(fs.content).unwrap(),
                )
                .unwrap(),
            })
            .collect(),
    };

    let client_msg: ClientMessage = ClientMessage {
        message: Some(client_message::Message::PatchRequest(req)),
    };

    to_proto::<PatchResponse>(client.request(client_msg.encode_to_vec())?)
}

/// calculate delta of a file
fn calculate_delta(base_filepath: &PathBuf, signature: Signature) -> Result<Vec<u8>> {
    let mut delta = vec![];
    let mut file_bytes: Vec<u8> = Vec::new();

    let mut file = File::open(base_filepath.clone())?;
    file.read_to_end(&mut file_bytes)?;
    diff(&signature.index(), &file_bytes, &mut delta)?;

    return Ok(delta);
}

#[cfg(test)]
mod sync_test {
    // TODO: add sync tests
}
