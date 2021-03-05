use tonic_health::{ServingStatus, server::{health_reporter, HealthReporter}};
use tonic::transport::Server;
use futures::lock::Mutex;
use std::sync::Arc;

pub mod helloworld {
    tonic::include_proto!("helloworld");
}

use helloworld::greeter_server::{Greeter, GreeterServer};
use helloworld::{HelloReply, HelloRequest};

#[derive(Debug)]
pub struct Welcome {
    reporter: Arc<Mutex<HealthReporter>>,
}

#[tonic::async_trait]
impl Greeter for Welcome {
    async fn say_hello(
        &self,
        request: tonic::Request<HelloRequest>,
    ) -> Result<tonic::Response<HelloReply>, tonic::Status> {
        let name = request.into_inner().name;
	{
	    let mut ptr = self.reporter.lock().await;
            match name.as_str() {
		"John Doe" => ptr.set_not_serving::<GreeterServer<Welcome>>().await,
		_ => ptr.set_serving::<GreeterServer<Welcome>>().await
	    }
        }

        Ok(tonic::Response::new(HelloReply {
            message: format!("Hello {}", name),
        }))
    }
}

static BIND_ADDRESS: &str = "0.0.0.0:49999";
static TEST_SERVICE_NAME: &str = "helloworld.Greeter";

// This is pseudo main function
async fn serve() {
    let addr = BIND_ADDRESS.parse().unwrap();

    let (mut reporter, health_server) = health_reporter();
    reporter.set_service_status("", ServingStatus::Serving).await;
    reporter.set_service_status("helloworld.Greeter", ServingStatus::Serving).await;

    let hello = Welcome { reporter: Arc::new(Mutex::new(reporter)) };

    Server::builder()
        .add_service(health_server)
        .add_service(GreeterServer::new(hello))
        .serve(addr)
        .await
        .unwrap();
}

use tokio::process::Command;

fn gcli_basic() -> Command {
    let mut ptr = Command::new("grpc_cli");
    ptr.arg("--noremotedb").arg("-proto_path").arg("proto/");
    ptr
}

fn gcli_health() -> Command {
    let mut ptr = gcli_basic();
    ptr.arg("-protofiles")
        .arg("health.proto")
        .arg("call")
        .arg(BIND_ADDRESS);
    ptr
}

fn gcli_hello() -> Command {
    let mut ptr = gcli_basic();
    ptr.arg("-protofiles")
        .arg("helloworld.proto")
        .arg("call")
        .arg(BIND_ADDRESS);
    ptr
}

fn gcli_hello_foo() -> Command {
    let mut ptr = gcli_hello();
    ptr.arg("SayHello").arg("name: 'Foobar'");
    ptr
}

#[tokio::main]
async fn main() {
    tokio::spawn(async move { serve().await });

    tokio::time::delay_for(std::time::Duration::new(1, 0)).await;

    // Test health server by calling Check service: ''
    {
        // read expected result to a string
        let exp = std::fs::read_to_string("expected/check.txt").unwrap();
        let output = gcli_health().arg("Check").arg(&format!("service: '{}'", TEST_SERVICE_NAME)).output().await;

        assert_eq!(std::str::from_utf8(&output.unwrap().stdout).unwrap(), exp);
    }

    // Test health server by calling Watch service: ''
    {
        let exp = std::fs::read_to_string("expected/watcher.txt").unwrap();
        let mut watcher = gcli_health()
            .arg("Watch")
            .arg(&format!("service: '{}'", TEST_SERVICE_NAME))
            .stdout(std::process::Stdio::piped())
            .spawn()
            .expect("failed to spawn watcher");

        let mut stdout = watcher.stdout.take().expect("no handle to stdout");

        use tokio::io::AsyncReadExt;

        // Now call SayHello service with names
        {
	    tokio::time::delay_for(std::time::Duration::new(1, 0)).await;

	    gcli_hello()
                .arg("SayHello")
                .arg("name: 'John Doe'")
                .output().await.unwrap();

	    gcli_hello_foo().output().await.unwrap();

	    tokio::time::delay_for(std::time::Duration::new(1, 0)).await;

	    watcher.kill().unwrap();

            let mut res = String::new();
            stdout.read_to_string(&mut res).await.unwrap();

            // let mut res = "".to_string();
            // stdout.read_to_string(&mut res).await.unwrap();

            assert_eq!(exp, res);
        }
    }
}
