include!("hotel_head.rs");

use mrbig_derive::{Configurable, Run};

// Use macro to register endpoints
#[derive(Run, Configurable, Default)]
#[mrbig_register_grpc(service = "hotel.Hotel")]
pub struct Micro {
    context: mrbig_core::Context,
}

use tokio::process::Command;

fn gcli_ls(hostname: &str, port: u16) -> Command {
    let mut ptr = Command::new("grpc_cli");
    ptr.arg("ls")
        .arg("--l")
        .arg(&format!("{}:{}", hostname, port));
    ptr
}

fn gcli_type(hostname: &str, port: u16) -> Command {
    let mut ptr = Command::new("grpc_cli");
    ptr.arg("type")
        .arg("--l")
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

    // Test grpc service by calling ls --l
    {
        // read expected result to a string
        let exp = std::fs::read_to_string("expected/ls.txt").unwrap();

        let output = gcli_ls(&hostname, port).output().await;

        assert_eq!(std::str::from_utf8(&output.unwrap().stdout).unwrap(), exp);
    }

    // Test grpc service by calling ls --l <url> <service_method>
    {
        // read expected result to a string
        let exp = std::fs::read_to_string("expected/service_method.txt").unwrap();

        let output = gcli_ls(&hostname, port)
            .arg("hotel.Hotel/Rates")
            .output()
            .await;

        assert_eq!(std::str::from_utf8(&output.unwrap().stdout).unwrap(), exp);
    }

    // Test grpc service by calling type --l <url> <type>
    {
        // read expected result to a string
        let exp = std::fs::read_to_string("expected/specific_type.txt").unwrap();

        let output = gcli_type(&hostname, port)
            .arg("profile.Result")
            .output()
            .await;

        assert_eq!(std::str::from_utf8(&output.unwrap().stdout).unwrap(), exp);
    }

    Ok(())
}
