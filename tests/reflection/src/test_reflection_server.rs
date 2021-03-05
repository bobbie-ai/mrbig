use grpc_reflection::{Reflection, ServerReflectionServer};
use tonic::transport::Server;

mod descriptor {
    use grpc_reflection::{decode, DescriptorMap};
    include!(concat!(
        env!("OUT_DIR"),
        concat!("/grpc_reflection_build_descriptor.rs")
    ));
}

static BIND_ADDRESS: &str = "0.0.0.0:49999";

async fn serve() {
    let addr = BIND_ADDRESS.parse().unwrap();

    let reflection = Reflection::new(
        descriptor::SERVICES.iter().map(|&s| s.into()).collect(),
        descriptor::LazyDescriptorMap::new(),
    );

    Server::builder()
        .add_service(ServerReflectionServer::new(reflection))
        .serve(addr)
        .await
        .unwrap();
}

use tokio::process::Command;

fn gcli_ls() -> Command {
    let mut ptr = Command::new("grpc_cli");
    ptr.arg("ls").arg("--l").arg(BIND_ADDRESS);
    ptr
}

fn gcli_type() -> Command {
    let mut ptr = Command::new("grpc_cli");
    ptr.arg("type").arg("--l").arg(BIND_ADDRESS);
    ptr
}

#[tokio::main]
async fn main() {
    tokio::spawn(async move { serve().await });

    // Test reflection server by calling ls --l
    {
        // read expected result to a string
        let exp = std::fs::read_to_string("expected/ls.txt").unwrap();

        let output = gcli_ls().output().await;

        assert_eq!(std::str::from_utf8(&output.unwrap().stdout).unwrap(), exp);
    }

    // Test reflection server with ls hotel.Hotel/Rates
    {
        // read expected result to a string
        let exp = std::fs::read_to_string("expected/service_method.txt").unwrap();

        let output = gcli_ls().arg("hotel.Hotel/Rates").output().await;

        assert_eq!(std::str::from_utf8(&output.unwrap().stdout).unwrap(), exp);
    }

    // Test reflection server with type --l
    {
        // read expected result to a string
        let exp = std::fs::read_to_string("expected/specific_type.txt").unwrap();

        let output = gcli_type().arg("profile.Result").output().await;

        assert_eq!(std::str::from_utf8(&output.unwrap().stdout).unwrap(), exp);
    }
}
