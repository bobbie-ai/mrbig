# mrbig_build

Library for `build.rs` script for `Mr. Big` services.

Compiles proto files using:
* [tonic-build](https://github.com/hyperium/tonic/tree/master/tonic-build) for client and server.
* [grpc_reflection_build](https://github.com/bobbie-ai/mrbig/tree/master/grpc_reflection_build) for the reflection server.

## Features

- `grpc`: enables the use of `tonic-build` to generate code for client and server.
- `reflection`: enables the use of `grpc_reflection_build` to generate code for the reflection server.

Required dependencies

```toml
[build-dependencies]
mrbig_build = <mrbig-version>
```

## Examples

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    mrbig_build::compile_protos(["proto/service.proto"], ["proto/"])?;
    Ok(())
}
```
