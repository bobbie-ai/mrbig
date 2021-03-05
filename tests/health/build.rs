fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::configure().compile(&["health.proto", "helloworld.proto"], &["proto/"])?;
    Ok(())
}
