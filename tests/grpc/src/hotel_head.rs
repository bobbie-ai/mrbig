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

#[tonic::async_trait]
impl Hotel for Booker {
    async fn rates(
        &self,
        request: tonic::Request<HotelRequest>,
    ) -> Result<tonic::Response<HotelResponse>, tonic::Status> {
	log::info!("rates were called");

        let mut fake_hotel = profile::Hotel::default();
        fake_hotel.name = request.into_inner().in_date;

        Ok(tonic::Response::new(HotelResponse {
            hotels: vec![fake_hotel],
            rate_plans: vec![],
        }))
    }
}
