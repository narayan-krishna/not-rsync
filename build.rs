use prost_build::Config;
use std::io::Result;

fn main() -> Result<()> {
    Config::new()
        // .type_attribute("MessageType", "#[derive(::prost::Enumeration)]")
        .protoc_arg("--experimental_allow_proto3_optional")
        .compile_protos(&["src/proto/not_rsync.proto"], &["src/proto"])
        .unwrap();

    Ok(())
}
