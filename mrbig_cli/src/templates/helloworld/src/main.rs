pub mod helloworld {
    tonic::include_proto!("helloworld");
}

use helloworld::greeter_server::{Greeter, GreeterServer};
use helloworld::{HelloReply, HelloRequest};

#[derive(Debug, Default)]
pub struct Contract {}

#[tonic::async_trait]
impl Greeter for Contract {
    async fn say_hello(
        &self,
        request: tonic::Request<HelloRequest>,
    ) -> Result<tonic::Response<HelloReply>, tonic::Status> {
        Ok(tonic::Response::new(HelloReply {
            message: format!("Hello {}", request.into_inner().name),
        }))
    }
}

// Use macro to register endpoints
#[derive(mrbig_derive::Run, mrbig_derive::Configurable, Default)]
#[mrbig_register_grpc(service = "helloworld.Greeter")]
pub struct Micro {
    context: mrbig_core::Context,
}

#[tokio::main]
async fn main() -> Result<(), mrbig_core::Error> {
    // New service with default configurations
    let mut service = Micro::default();

    service.init().await?;

    // Serve the endpoints
    service.run(Contract {}).await?;

    Ok(())
}
