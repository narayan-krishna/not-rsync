use anyhow::Result;
use rsync_rs::{local::LocalClient, remote::RemoteClient, Client};
use std::path::PathBuf;

enum ServerType {
    Local,
    Remote,
}

pub fn sync(filepath: PathBuf) -> Result<()> {
    let server_type = ServerType::Remote;

    // connection should have trait connection
    let mut client: Box<dyn Client> = match server_type {
        ServerType::Remote => Box::new(RemoteClient::init()),
        ServerType::Local => Box::new(LocalClient::init()),
    };

    client.create_connection()?;
    let _connection_ok = check_connection(&mut client)?;
    let _send_filepath = send_filepath(&mut client, filepath)?;
    let _signature = request_signature(&mut client)?;
    let _shutdown_ok = request_shutdown(&mut client)?;
    // let delta = calculate_delta(base_filepath, signature)
    // let remote_patch_ok = request_remote(client, delta)?;

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

fn request_signature(client: &mut Box<dyn Client>) -> Result<()> {
    let signature = client.request("signature".into())?;
    assert_eq!(
        String::from_utf8(signature)?,
        "calculating signature"
    );
    Ok(())
}

fn request_shutdown(client: &mut Box<dyn Client>) -> Result<()> {
    assert_eq!(
        String::from_utf8(client.request("shutdown".into())?)?,
        "Shutting down!"
    );

    Ok(())
}
