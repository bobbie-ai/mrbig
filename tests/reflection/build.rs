fn main() -> Result<(), Box<dyn std::error::Error>> {
    grpc_reflection_build::compile_protos(
        &[
            "reflection.proto",
            "hotel.proto",
            "rate.proto",
            "profile.proto",
        ],
        &["proto/"],
    )?;
    Ok(())
}
