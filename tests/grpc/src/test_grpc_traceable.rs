pub mod hotel {
    tonic::include_proto!("hotel");
}
pub mod profile {
    tonic::include_proto!("profile");
}
pub mod rate {
    tonic::include_proto!("rate");
}

use hotel::hotel_server::{Hotel, HotelServer};
use hotel::{Request as HotelRequest, Response as HotelResponse};

#[derive(Debug, Default)]
pub struct Booker {}

#[mrbig_derive::service_impl(tracing = "true", telemetry = "true")]
impl Hotel for Booker {
    async fn rates(
        &self,
        request: tonic::Request<HotelRequest>,
    ) -> Result<tonic::Response<HotelResponse>, tonic::Status> {
        log::info!("rates were called");

        let mut fake_hotel = profile::Hotel::default();
        fake_hotel.name = request.into_inner().in_date;

        if fake_hotel.name == "never" {
            return Err(tonic::Status::new(
                tonic::Code::NotFound,
                "never is never accepted",
            ));
        }

        Ok(tonic::Response::new(HotelResponse {
            hotels: vec![fake_hotel],
            rate_plans: vec![],
        }))
    }
}

use mrbig_derive::{Configurable, Run};

// Use macro to register endpoints
#[derive(Run, Configurable, Default)]
#[mrbig_register_grpc(service = "hotel.Hotel")]
pub struct Micro {
    context: mrbig_core::Context,
}

use tokio::process::Command;

fn gcli_basic() -> Command {
    Command::new("grpc_cli")
}

fn gcli_ls(hostname: &str, port: u16) -> Command {
    let mut ptr = gcli_basic();
    ptr.arg("ls")
        .arg("--l")
        .arg(&format!("{}:{}", hostname, port));
    ptr
}

fn gcli_call(hostname: &str, port: u16) -> Command {
    let mut ptr = gcli_basic();
    ptr.arg("call").arg(&format!("{}:{}", hostname, port));
    ptr
}

async fn rates_call(hostname: &str, port: u16) {
    gcli_call(&hostname, port)
        .arg("--metadata")
        .arg("x-request-id:fakerequestid")
        .arg("Rates")
        .arg("inDate: 'foo'")
        .output()
        .await
        .unwrap();
}

async fn bad_rates_call(hostname: &str, port: u16) {
    gcli_call(&hostname, port)
        .arg("--metadata")
        .arg("x-request-id:fakebadrequestid")
        .arg("Rates")
        .arg("inDate: 'never'")
        .output()
        .await
        .unwrap();
}

#[tokio::main]
async fn main() -> Result<(), String> {
    // New service with default configurations
    let mut service = Micro {
        context: mrbig_core::Context::default(),
    };
    service.init().await.expect("failed to init service");

    use mrbig_core::config::Configurable;
    let mrbig_core::config::Service {
        hostname,
        port,
        metrics,
        ..
    } = service
        .get_config()
        .expect("config not available")
        .service
        .clone();

    tokio::spawn(async move { service.run(Booker {}).await.expect("failed to run service") });

    // Test grpc service by calling ls --l
    {
        gcli_ls(&hostname, port).output().await.unwrap();
    }

    // Call rates three times
    rates_call(&hostname, port).await;
    rates_call(&hostname, port).await;
    rates_call(&hostname, port).await;

    // Get the metrics
    {
        use hyper::{body::Buf, Body, Client, Request};

        let req = Request::builder()
            .method("GET")
            .uri(&format!("http://{}:{}", metrics.hostname, metrics.port))
            .body(Body::empty())
            .expect("request builder failed");

        let body = Client::new()
            .request(req)
            .await
            .expect("metrics request failed")
            .into_body();

        let buf = hyper::body::to_bytes(body)
            .await
            .expect("body to bytes failed");

        // Simply test that the response is non empty
        assert!(!buf.bytes().is_empty());
    }

    // Get error on purpose
    bad_rates_call(&hostname, port).await;

    Ok(())
}
