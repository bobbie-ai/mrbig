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

#[mrbig_derive::service_impl(tracing = "true")]
impl Hotel for Booker {
    async fn rates(
        &self,
        request: tonic::Request<HotelRequest>,
    ) -> Result<tonic::Response<HotelResponse>, tonic::Status> {
        let mut fake_hotel = profile::Hotel::default();
        fake_hotel.name = request.into_inner().in_date;

        Ok(tonic::Response::new(HotelResponse {
            hotels: vec![fake_hotel],
            rate_plans: vec![],
        }))
    }
}

// Use macro to register endpoints
#[derive(mrbig_derive::Run, mrbig_derive::Configurable, Default)]
#[mrbig_register_grpc(service = "hotel.Hotel")]
pub struct Micro {
    context: mrbig_core::Context,
}

#[tokio::main]
async fn main() -> Result<(), mrbig_core::Error> {
    // New service with default configurations
    let mut service = Micro::default();

    service.init().await?;

    // Serve the endpoints
    service.run(Booker {}).await?;

    Ok(())
}
