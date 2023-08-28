use anyhow::{anyhow, Result};
use fast_rsync::{diff, Signature};
use rsync_rs::{local::LocalClient, remote::RemoteClient, Client};
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

enum ServerType {
    Local,
    Remote,
}

// make this a struct, impl drop for sync

/// Perform file synchronization operations between client and server.
pub fn sync(base_filepath: PathBuf, target_filepath: PathBuf) -> Result<()> {
    let server_type = ServerType::Remote;

    // connection should have trait connection
    let mut client: Box<dyn Client> = match server_type {
        ServerType::Remote => Box::new(RemoteClient::new()),
        ServerType::Local => Box::new(LocalClient::new()),
    };

    client.create_connection()?;
    let _connection_ok = check_connection(&mut client)?;
    let _send_filepath = send_filepath(&mut client, target_filepath)?;
    let signature = request_signature(&mut client)?;
    let patch: Vec<u8> = calculate_delta(base_filepath, signature)?;
    let _remote_patch_ok = send_patch(&mut client, patch)?;

    let _shutdown_ok = request_shutdown(&mut client)?;
    Ok(())
}

fn check_connection(client: &mut Box<dyn Client>) -> Result<()> {
    assert_eq!(String::from_utf8(client.request("SYN".into())?)?, "ACK");
    Ok(())
}

fn send_filepath(client: &mut Box<dyn Client>, filepath: PathBuf) -> Result<()> {
    assert_eq!(
        String::from_utf8(client.request("filepath".into())?)?,
        "ready for filepath"
    );
    assert_eq!(
        String::from_utf8(client.request(filepath.to_str().unwrap().into())?)?,
        format!("received {}", filepath.to_str().unwrap())
    );
    Ok(())
}

fn request_signature(client: &mut Box<dyn Client>) -> Result<Signature> {
    let serialized_signature = client.request("signature".into())?;
    let deserialized_signature = Signature::deserialize(serialized_signature)?;

    Ok(deserialized_signature)
}

fn calculate_delta(base_filepath: PathBuf, signature: Signature) -> Result<Vec<u8>> {
    let mut patch = vec![];
    let mut file = match File::open(base_filepath) {
        Err(e) => return Err(anyhow!("Error: {}", e)),
        Ok(file) => file,
    };
    let mut file_bytes: Vec<u8> = Vec::new();
    match file.read_to_end(&mut file_bytes) {
        Err(e) => {
            return Err(anyhow!("Error: {}", e));
        }
        Ok(_) => {
            diff(&signature.index(), &file_bytes, &mut patch)?;
        }
    };

    return Ok(patch);
}

fn send_patch(client: &mut Box<dyn Client>, patch: Vec<u8>) -> Result<()> {
    assert_eq!(
        String::from_utf8(client.request("patch".into())?)?,
        "ready for patch"
    );
    assert_eq!(String::from_utf8(client.request(patch)?)?, "received patch",);

    Ok(())
}

fn request_shutdown(client: &mut Box<dyn Client>) -> Result<()> {
    assert_eq!(
        String::from_utf8(client.request("shutdown".into())?)?,
        "Shutting down!"
    );

    Ok(())
}
