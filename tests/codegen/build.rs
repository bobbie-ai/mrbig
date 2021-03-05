fn main() -> Result<(), Box<dyn std::error::Error>> {
    mrbig_build::compile_protos(&["proto/helloworld.proto"], &["proto"])?;
    Ok(())
}
