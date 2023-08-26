mod comms;

use comms::{local::Local, remote::Remote, Comms};
use std::error::Error;
use std::path::PathBuf;

enum ServerType {
    Local,
    Remote,
}

pub fn sync() -> Result<(), Box<dyn Error>> {
    let server_type = ServerType::Remote;

    // connection should have trait connection
    let mut comms: Box<dyn Comms> = match server_type {
        ServerType::Remote => Box::new(Remote::init()),
        ServerType::Local => Box::new(Local::init()),
    };

    comms.create_connection()?;
    let _comms_ok = check_comms(&mut comms)?;

    println!("[Received] {}", comms.request("hello")?);
    println!("[Received] {}", comms.request("shutdown")?);
    // send a syn to the server, hopefully receive an acks
    // let signature = request_signature(comms)?;
    // let delta = calculate_delta(base_filepath, signature)
    // let remote_patch_ok = request_remote(comms, delta)?;

    Ok(())
}

fn check_comms(comms: &mut Box<dyn Comms>) -> Result<(), Box<dyn Error>> {
    assert_eq!(comms.request("SYN")?, "ACK");
    Ok(())
}

// fn request_signature(comms: &mut Box<dyn Comms>, filepath: PathBuf) -> Result<Signature, Box<dyn Error>> {
//     let str_path = filepath.to_str().unwrap();
//     assert_eq!(comms.request("filepath")?, "Ready for filepath");
//     assert_eq!(comms.request(str_path)?, format!("Received {}", str_path));
//     let signature = comms.request("signature")?;
//     Ok(())
// }

//
// fn request_remote(comms) {
//
// }
