fn main() -> Result<(), Box<dyn std::error::Error>> {
    mrbig_build::compile_protos(
        &[
            "hotel.proto",
            "rate.proto",
            "profile.proto",
            "helloworld.proto",
        ],
        &["proto/"],
    )?;
    Ok(())
}
