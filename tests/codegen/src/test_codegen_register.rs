pub mod health {
    tonic::include_proto!("grpc.health.v1");
}

use futures::Stream;
use health::health_check_response::ServingStatus;
use health::health_server::Health;
use health::{HealthCheckRequest, HealthCheckResponse};
use std::pin::Pin;
use tonic::{Request, Response, Status};

#[derive(Debug, Default)]
pub struct Sane {}

#[tonic::async_trait]
impl Health for Sane {
    type WatchStream =
        Pin<Box<dyn Stream<Item = Result<HealthCheckResponse, Status>> + Send + Sync + 'static>>;

    async fn check(
        &self,
        _request: Request<HealthCheckRequest>,
    ) -> Result<Response<HealthCheckResponse>, Status> {
        Ok(Response::new(HealthCheckResponse {
            status: ServingStatus::Serving as i32,
        }))
    }

    async fn watch(
        &self,
        _request: Request<HealthCheckRequest>,
    ) -> Result<Response<Self::WatchStream>, Status> {
        let output = async_stream::try_stream! {
            loop {
                std::thread::sleep(std::time::Duration::from_millis(5000));
                yield HealthCheckResponse {
                    status: ServingStatus::Serving as i32,
                };
            }
        };

        Ok(Response::new(Box::pin(output) as Self::WatchStream))
    }
}

use mrbig_derive::{Configurable, Run};

// Use macro to register endpoints
#[derive(Run, Configurable, Default)]
#[mrbig_register_grpc(
    trait = "health::health_server::Health",
    service = "grpc.health.v1.Health"
)]
pub struct Micro {
    context: mrbig_core::Context,
}

async fn serve() {
    // New service with default configurations
    let mut service = Micro::default();
    service.init().await.expect("failed to init service");

    // Serve the endpoints
    service.run(Sane {}).await.expect("failed to run service");
}

// Simply test if this code compiles, serve and exit.
#[tokio::main]
async fn main() {
    tokio::spawn(async move { serve().await });

    std::thread::sleep(std::time::Duration::from_millis(1000));
}
