include!("greeter_server_head.rs");

use mrbig_derive::{Configurable, Run};

// Use macro to register endpoints
#[derive(Run, Configurable)]
#[mrbig_register_grpc(trait = "helloworld::greeter_server::Greeter")]
pub struct Micro {
    context: mrbig_core::Context,
}

mod greeter_client;

use futures::future::FutureExt;

#[tokio::main]
async fn main() {
    // New service with default configurations
    let mut service = Micro {
        context: mrbig_core::Context::default(),
    };
    service.init().await.expect("failed to init service");

    use mrbig_core::config::Configurable;
    let cfg = service
        .get_config()
        .expect("no config available")
        .service
        .clone();

    tokio::spawn(async move {
        service
            .run(Welcome {})
            .await
            .expect("failed to run service")
    });

    async {
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
    .then(|_| greeter_client::client(cfg))
    .await
    .expect("failed to use client");
}
