pub mod helloworld {
    tonic::include_proto!("helloworld");
}

use helloworld::greeter_server::Greeter;
use helloworld::{HelloReply, HelloRequest};

#[derive(Debug, Default)]
pub struct Welcome {
    greeting: String,
}

#[tonic::async_trait]
impl Greeter for Welcome {
    async fn say_hello(
        &self,
        request: tonic::Request<HelloRequest>,
    ) -> Result<tonic::Response<HelloReply>, tonic::Status> {
        Ok(tonic::Response::new(HelloReply {
            message: format!("{} {}", self.greeting, request.into_inner().name),
        }))
    }
}

use mrbig_derive::Run;

// User configuration parameters
#[derive(Default, serde_derive::Deserialize)]
pub struct User {
    my_port: u16,
    greeting: String,
}

use helloworld::greeter_server::GreeterServer;
use mrbig_core::config::Config;
use mrbig_core::{context::WithContext, Context};

// Use macro to register endpoints
#[derive(Run, Default)]
#[mrbig_register_grpc(service = "helloworld.Greeter")]
pub struct Micro {
    context: mrbig_core::Context,
    user: User,
    config: Config,
}

mod greeter_client;

use futures::future::FutureExt;
use mrbig_core::config::Configurable;

static TOML_CONFIG: &str = "/tmp/mrbig_test_greeter_extra.toml";

impl Configurable<'_> for Micro {
    type Extra = User;

    fn get_config(&self) -> Option<&Config> {
        Some(&self.config)
    }

    fn set_config(&mut self, config: Config) {
        self.config = config;
    }

    fn set_config_extra(&mut self, extra: Self::Extra) {
        self.user = extra;
    }

    fn load_from_args(&mut self) -> Result<(), mrbig_core::Error> {
        let mut config = Config::from_args_vec(vec![
            "--port".into(),
            "49999".into(),
            "--config".into(),
            TOML_CONFIG.into(),
        ])?;
        self.set_config_extra(config.try_raw_into()?);
        self.set_config(config);
        Ok(())
    }
}

impl WithContext for Micro {
    fn get_context(&self) -> &Context {
        &self.context
    }

    fn get_context_mut(&mut self) -> &mut Context {
        &mut self.context
    }
}

#[tokio::main]
async fn main() {
    // Create config for testing
    let mut file =
        std::fs::File::create(TOML_CONFIG).expect("failed to create temporary config file");

    use std::io::Write;
    writeln!(file, "my_port = 39999")
        .and(writeln!(file, r#"greeting = "Hello""#))
        .expect("failed to write to temporary config file");

    // New service with default configurations
    let mut service = Micro::default();

    service.init().await.expect("failed to init service");

    // override port with user configured port
    let mut cfg = service.get_config().expect("no config available").clone();

    cfg.service.port = service.user.my_port;
    service.set_config(cfg.clone());
    let greeting = service.user.greeting.clone();

    tokio::spawn(async move {
        service
            .run(Welcome { greeting })
            .await
            .expect("failed to run service")
    });

    async {
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
    .then(|_| greeter_client::client(cfg.service))
    .await
    .expect("failed to use client");
}
