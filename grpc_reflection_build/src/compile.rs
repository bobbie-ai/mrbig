// The code below code is taken from prost_build.
// This code should be kept separate from the rest so
// that we eventually import it or reimplement it later.

use prost::Message;
use prost_types::FileDescriptorSet;
use std::io::{Error, ErrorKind, Result};
use std::path::Path;
use std::process::Command;

pub fn compile<P>(protos: &[P], includes: &[P]) -> Result<FileDescriptorSet>
where
    P: AsRef<Path>,
{
    let tmp = tempfile::Builder::new().prefix("mrbig-build").tempdir()?;
    let descriptor_set = tmp.path().join("mrbig-descriptor-set");

    let mut cmd = Command::new(prost_build::protoc());
    cmd.arg("--include_imports")
        .arg("--include_source_info")
        .arg("-o")
        .arg(&descriptor_set);

    for include in includes {
        cmd.arg("-I").arg(include.as_ref());
    }

    // Set the protoc include after the user includes in case the user wants to
    // override one of the built-in .protos.
    cmd.arg("-I").arg(prost_build::protoc_include());

    for proto in protos {
        cmd.arg(proto.as_ref());
    }

    let output = cmd.output()?;
    if !output.status.success() {
        return Err(Error::new(
            ErrorKind::Other,
            format!("protoc failed: {}", String::from_utf8_lossy(&output.stderr)),
        ));
    }

    let buf = std::fs::read(descriptor_set)?;
    let fds = FileDescriptorSet::decode(&*buf)?;
    Ok(fds)
}
