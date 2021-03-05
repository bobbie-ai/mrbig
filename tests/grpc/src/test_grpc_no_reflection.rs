include!("hotel_head.rs");

use mrbig_derive::{Configurable, Run};

// Use macro to register endpoints
#[derive(Run, Configurable, Default)]
#[mrbig_register_grpc(service = "hotel.Hotel")]
#[mrbig_disable_reflection]
pub struct Micro {
    context: mrbig_core::Context,
}

use tokio::process::Command;

fn gcli_call(hostname: &str, port: u16) -> Command {
    let mut ptr = Command::new("grpc_cli");
    ptr.arg("-proto_path")
        .arg("proto/")
        .arg("-protofiles")
        .arg("hotel.proto")
        .arg("call")
        .arg("--noremotedb")
        .arg(&format!("{}:{}", hostname, port));
    ptr
}

#[tokio::main]
async fn main() -> Result<(), String> {
    // New service with default configurations
    let mut service = Micro {
        context: mrbig_core::Context::default(),
    };
    service.init().await.expect("failed to init service");

    use mrbig_core::config::Configurable;
    let mrbig_core::config::Service { hostname, port, .. } = service
        .get_config()
        .expect("config not available")
        .service
        .clone();

    tokio::spawn(async move { service.run(Booker {}).await.expect("failed to run service") });

    // Test grpc without reflection with call hotel.Hotel.Rates
    {
        // read expected result to a string
        let exp = std::fs::read_to_string("expected/no_reflection.txt").unwrap();

        let output = gcli_call(&hostname, port)
            .arg("hotel.Hotel.Rates")
            .arg("inDate: 'FooBar'")
            .output()
            .await;

        assert_eq!(std::str::from_utf8(&output.unwrap().stdout).unwrap(), exp);
    }

    Ok(())
}
