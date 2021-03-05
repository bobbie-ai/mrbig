fn main() -> Result<(), Box<dyn std::error::Error>> {
    // compile grpc.reflection proto spec with tonic to get server and client.
    tonic_build::compile_protos("proto/reflection.proto")?;

    Ok(())
}
