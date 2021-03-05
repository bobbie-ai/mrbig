pub mod hotel {
    tonic::include_proto!("hotel");
}
pub mod profile {
    tonic::include_proto!("profile");
}
pub mod rate {
    tonic::include_proto!("rate");
}

use hotel::hotel_client::HotelClient;
use hotel::Request as HotelRequest;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = mrbig_core::config::Config::from_args()?.service;

    let address = format!("http://{}:{}", config.hostname, config.port);

    eprintln!("connecting to {}", address);

    use std::convert::TryFrom;
    let mut client = HotelClient::connect(tonic::transport::Endpoint::try_from(address).unwrap())
        .await
        .map_err(|e| format!("{:?}", e))?;

    let request = tonic::Request::new(HotelRequest {
        in_date: "Tonic".into(),
        out_date: "FooBar".into(),
    });

    let response = client
        .rates(request)
        .await
        .map_err(|e| format!("{:?}", e))?;

    println!("{:?}", response);

    Ok(())
}
