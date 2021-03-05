include!("hotel_head.rs");
include!("../../codegen/src/greeter_server_head.rs");

use mrbig_derive::{Configurable, Run};

use helloworld::greeter_server::GreeterServer;

// Use macro to register endpoints
#[derive(Run, Configurable, Default)]
#[mrbig_register_grpc(service = "hotel.Hotel", health = false)]
#[mrbig_register_grpc(service = "helloworld.Greeter")]
pub struct Micro {
    context: mrbig_core::Context,
}

use tokio::process::Command;

fn gcli_basic() -> Command {
    Command::new("grpc_cli")
}

fn gcli_ls(hostname: &str, port: u16) -> Command {
    let mut ptr = gcli_basic();
    ptr.arg("ls").arg(&format!("{}:{}", hostname, port));
    ptr
}

fn gcli_call(hostname: &str, port: u16) -> Command {
    let mut ptr = gcli_basic();
    ptr.arg("call").arg(&format!("{}:{}", hostname, port));
    ptr
}

#[tokio::main]
async fn main() {
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

    tokio::spawn(async move {
        service
            .run(Booker {}, Welcome {})
            .await
            .expect("failed to run service")
    });

    // Test grpc multiple service with ls
    {
        // read expected result to a string
        let exp = std::fs::read_to_string("expected/multiple_ls.txt").unwrap();

        let output = gcli_ls(&hostname, port).output().await;

        assert_eq!(std::str::from_utf8(&output.unwrap().stdout).unwrap(), exp);
    }

    // Test grpc multiple service with call
    {
        // read expected result to a string
        let exp = std::fs::read_to_string("expected/health_of_hello.txt").unwrap();

        let output = gcli_call(&hostname, port)
            .arg("Check")
            .arg("service: 'helloworld.Greeter'")
            .output()
            .await;

        assert_eq!(std::str::from_utf8(&output.unwrap().stdout).unwrap(), exp);
    }

    // Test grpc multiple service with call, hotel.Hotel must not exist
    {
        let output = gcli_call(&hostname, port)
            .arg("Check")
            .arg("service: 'hotel.Hotel'")
            .output()
            .await;

        assert!(std::str::from_utf8(&output.unwrap().stderr)
            .unwrap()
            .find("status code 5")
            .is_some());
    }

    // Test grpc multiple with helloworld.Greeter service call
    {
        // read expected result to a string
        let exp = std::fs::read_to_string("expected/hello_reply.txt").unwrap();

        let output = gcli_call(&hostname, port)
            .arg("--metadata")
            .arg("x-request-id:fakerequestid")
            .arg("helloworld.Greeter/SayHello")
            .arg("name: 'John Doe'")
            .output()
            .await;

        assert_eq!(std::str::from_utf8(&output.unwrap().stdout).unwrap(), exp);
    }
}
