//! integration tests focused around client connecting to remote server

use assert_cmd::Command;
use std::fs;
use test_files::TestFiles;

#[test]
fn test_client_server_simple() {
    let (file1, contents1) = ("a.txt", "this is a base file");
    let (file2, contents2) = ("b.txt", "this is a modified file");

    let temp_dir = TestFiles::new();
    temp_dir.file(file1, contents1).file(file2, contents2);

    let fp1 = temp_dir.path().join(file1);
    let fp2 = temp_dir.path().join(file2);

    dbg!(fp1.clone());
    dbg!(fp2.clone());

    assert_eq!(fs::read_to_string(fp1.clone()).unwrap(), contents1);
    let mut cmd = Command::cargo_bin("client").unwrap();
    cmd.args(&[
        "--src",
        format!("knara@localhost:{}", fp1.clone().to_str().unwrap()).as_str(),
        "--dest",
        format!("knara@localhost:{}", fp2.clone().to_str().unwrap()).as_str(),
    ])
    .assert()
    .success();
    assert_eq!(
        fs::read_to_string(fp1).unwrap(),
        fs::read_to_string(fp2).unwrap()
    );
}
