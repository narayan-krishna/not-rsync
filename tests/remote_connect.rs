//! integration tests focused around client connecting to remote server

mod common;

use assert_cmd::Command;
use log::info;
use std::fs;
use test_files::TestFiles;

#[test]
fn test_client_server_simple() {
    common::init();

    let (file1, contents1) = ("a.txt", "this is a base file");
    let (file2, contents2) = ("b.txt", "this is a modified file");

    let temp_dir = TestFiles::new();
    temp_dir.file(file1, contents1).file(file2, contents2);

    let fp1 = temp_dir.path().join(file1);
    let fp2 = temp_dir.path().join(file2);

    info!("file 2 start: {}", fs::read_to_string(fp2.clone()).unwrap());
    assert_ne!(
        fs::read_to_string(fp1.clone()).unwrap(),
        fs::read_to_string(fp2.clone()).unwrap()
    );
    let mut cmd = Command::cargo_bin("client").unwrap();
    let assert = cmd
        .args(&[
            "--src",
            format!("knara@localhost:{}", fp1.clone().to_str().unwrap()).as_str(),
            "--dest",
            format!("knara@localhost:{}", fp2.clone().to_str().unwrap()).as_str(),
            "--ssh",
        ])
        .assert();

    info!(
        "{}",
        String::from_utf8(assert.get_output().stderr.to_owned()).unwrap()
    );
    assert.success();

    info!("file 2 end: {}", fs::read_to_string(fp2.clone()).unwrap());
    assert_eq!(
        fs::read_to_string(fp1).unwrap(),
        fs::read_to_string(fp2).unwrap()
    );
}
