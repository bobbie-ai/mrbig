pub mod hello_world {
    tonic::include_proto!("helloworld");
}

use hello_world::greeter_client::GreeterClient;
use hello_world::HelloRequest;

pub async fn client(cfg: mrbig_core::config::Service) -> Result<(), Box<dyn std::error::Error>> {
    let address = format!("http://{}:{}", cfg.hostname, cfg.port);

    let mut client = GreeterClient::connect(address).await?;

    let request = tonic::Request::new(HelloRequest {
        name: "MrBig".into(),
    });

    let response = client.say_hello(request).await?;

    assert_eq!(response.into_inner().message, "Hello MrBig");

    Ok(())
}
