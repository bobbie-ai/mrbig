# gRPC Reflection Server Build crate

## Overview

This crate's code must be called from `build.rs`, to produce server reflection data from a set of proto files. The code generated as a result can be used together with the crate `grpc_reflection` to create a working gRPC reflection server.

Simply add this crate as a dependency and change your `build.rs`:
```toml
# Cargo.toml

[dependencies]
# (...)
grpc_reflection = "0.1.0"

[build-dependencies]
# (...)
grpc_reflection_build = "0.1.0"
```

```rust
// build.rs
fn main() -> Result<(), Box<dyn std::error::Error>> {
    grpc_reflection_build::compile_protos(
        &[
            "reflection.proto",
            "helloworld.proto",
        ],
        &["proto/"],
    )?;
    Ok(())
}
```

The signature of the `compile_protos()` method is the same of `prost_build::compile_protos()`. This crate depends on `prost_build`.

The generated code can then be included and used with:

```rust
// main.rs
use tonic::transport::Server;
use grpc_reflection::{Reflection, ServerReflectionServer};

mod descriptor {
    include!(concat!(
        env!("OUT_DIR"),
        concat!("/grpc_reflection_build/descriptor.rs")
    ));
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();

    let reflection = Reflection::new(
            descriptor::SERVICES
                .into_iter()
                .map(|&s| s.into())
                .collect(),
            descriptor::LazyDescriptorMap::new(),
        );

    // you may want to append some more `add_service(...)`
    Server::builder()
        .add_service(ServerReflectionServer::new(reflection))
        .serve(addr)
        .await?;

    Ok(())
}
```

## Known Limitations

Only protobuf version 3 is supported.
