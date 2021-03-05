use std::fs;
use std::io::{Error, ErrorKind, Result, Write};
use std::path::{Path, PathBuf};

mod compile;
use compile::compile;

mod symbol_map;

mod lazy;

/// Static path to file with generated FileDescriptor code.
pub static GENERATED_DESCRIPTOR_NAME: &str = "grpc_reflection_build_descriptor.rs";

/// Compile the proto files to generate the file descriptor proto and
/// symbol map, required by the grpc reflection server.
/// The generated file contains:
/// * The list of services as an array of `&str`.
/// * A struct called `LazyDescriptorMap` that implements the
/// `DescriptorMap` trait from the `grpc_reflection` crate.
/// The generated file can be included and used to create an
/// instance of a gRPC `Reflection` server.
///
/// This method uses protoc to parse the proto files, as done by the
/// crate prost_bulid.
pub fn compile_protos<P>(protos: &[P], includes: &[P]) -> Result<()>
where
    P: AsRef<Path>,
{
    // Get a proto encoded descriptor set
    let mut descriptor_set = compile(protos, includes)?;
    descriptor_set.file.sort_by(|a, b| a.name().cmp(b.name()));

    let mut buf = String::new();
    lazy::from_descriptors(&mut buf, descriptor_set.file);

    let mut out_file: PathBuf = std::env::var_os("OUT_DIR")
        .ok_or_else(|| Error::new(ErrorKind::Other, "OUT_DIR environment variable is not set"))
        .map(Into::into)?;

    out_file.push(GENERATED_DESCRIPTOR_NAME);

    let mut out_file = fs::File::create(out_file)?;
    out_file.write_all(&buf.into_bytes())?;

    Ok(())
}
