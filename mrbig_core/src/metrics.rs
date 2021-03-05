use serde_derive::Deserialize;
use prometheus::{Encoder, TextEncoder};
use hyper::{
    header::CONTENT_TYPE,
    service::{make_service_fn, service_fn},
    Body, Request, Response, Server,
};

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub hostname: String,
    pub port: u16,
}

impl Default for Config {
    fn default() -> Self {
	Self {
	    hostname: "0.0.0.0".into(),
	    port: 9090,
	}
    }
}

async fn serve_req(_req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
    let encoder = TextEncoder::new();

    let metric_families = prometheus::gather();
    let mut buffer = vec![];
    encoder.encode(&metric_families, &mut buffer).unwrap();

    let response = Response::builder()
        .status(200)
        .header(CONTENT_TYPE, encoder.format_type())
        .body(Body::from(buffer))
        .unwrap();

    Ok(response)
}

pub(crate) async fn start_server(config: Config) {
    let address = format!("{}:{}", config.hostname, config.port).parse().expect("bad address");

    Server::bind(&address)
	.serve(make_service_fn(|_| async {
            Ok::<_, hyper::Error>(service_fn(serve_req))
	}))
	.await
	.expect("server error");
}
