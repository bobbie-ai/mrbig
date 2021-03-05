pub mod helloworld {
    tonic::include_proto!("helloworld");
}

use helloworld::greeter_server::Greeter;
use helloworld::{HelloReply, HelloRequest};

#[derive(Debug, Default)]
pub struct Welcome {}

#[mrbig_derive::service_impl(tracing = "true")]
impl Greeter for Welcome {
    async fn say_hello(
        &self,
        request: tonic::Request<HelloRequest>,
    ) -> Result<tonic::Response<HelloReply>, tonic::Status> {
        Ok(tonic::Response::new(HelloReply {
            message: format!("Hello {}", request.into_inner().name),
        }))
    }
}
