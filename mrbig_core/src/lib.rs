pub mod config;
pub mod context;
pub mod error;
pub use crate::error::{Error, Inner};
#[cfg(feature = "traceable")]
pub use ansi_term;
pub use context::Context;
pub use log;

#[cfg(feature = "grpc")]
pub use grpc_reflection;
#[cfg(feature = "grpc")]
pub use tonic_health;
#[cfg(feature = "telemetry")]
pub use lazy_static::lazy_static;
#[cfg(feature = "telemetry")]
pub use prometheus;
#[cfg(feature = "telemetry")]
pub mod metrics;

/// A default grpc interceptor that simply prints debug information about the received
/// protobuf request message.
#[cfg(feature = "grpc")]
pub fn default_grpc_interceptor(
    req: tonic::Request<()>,
) -> std::result::Result<tonic::Request<()>, tonic::Status> {
    use tonic::metadata::KeyAndValueRef;

    let text = req
        .metadata()
        .iter()
        .map(|kv| match kv {
            KeyAndValueRef::Ascii(k, v) => format!("{:?}: {:?}", k, v),
            KeyAndValueRef::Binary(k, v) => format!("{:?}: {:?}", k, v),
        })
        .collect::<Vec<String>>()
        .join(", ");

    log::debug!("headers: {{ {} }}", text);
    Ok(req)
}

#[cfg(all(feature = "env_log", not(feature = "traceable")))]
fn init_logging(config: &config::Config) -> Result<(), Error> {
    use env_logger::Builder;
    use log::LevelFilter;
    let mut logger = Builder::from_default_env();

    match (config.service.trace, config.service.debug) {
        (false, false) => {}
        (true, _) => {
            logger.filter(None, LevelFilter::Trace);
        }
        _ => {
            logger.filter(None, LevelFilter::Debug);
        }
    };

    let filters = match std::env::var(env_logger::DEFAULT_FILTER_ENV) {
        Ok(filters) => filters,
        Err(_) => config.service.logger_filters.clone(),
    };

    logger.parse(&filters);

    logger.init();

    Ok(())
}

#[cfg(all(feature = "traceable", not(feature = "env_log")))]
fn init_logging(config: &config::Config) -> Result<(), Error> {
    use tracing_subscriber::filter::{EnvFilter, LevelFilter};

    let directives = match std::env::var(EnvFilter::DEFAULT_ENV) {
        Ok(default) => default,
        Err(_) => config.service.logger_filters.clone(),
    };

    let mut filter = EnvFilter::new(directives);

    filter = match (config.service.trace, config.service.debug) {
        (false, false) => filter.add_directive(LevelFilter::INFO.into()),
        (true, _) => filter.add_directive(LevelFilter::TRACE.into()),
        _ => filter.add_directive(LevelFilter::DEBUG.into()),
    };

    tracing_subscriber::FmtSubscriber::builder()
        .with_env_filter(filter)
        .init();

    Ok(())
}

#[cfg(all(feature = "traceable", feature = "env_log"))]
fn init_logging(_config: &config::Config) -> Result<(), Error> {
    Ok(())
}

#[cfg(all(not(feature = "traceable"), not(feature = "env_log")))]
fn init_logging(_config: &config::Config) -> Result<(), Error> {
    Ok(())
}

pub fn init(config: &config::Config) -> Result<(), Error> {
    init_logging(config)
}

/// Creates a tonic::transport::Server from the configuration parameters.
#[cfg(feature = "grpc")]
pub fn new_grpc_server(args: &config::GrpcServer) -> tonic::transport::Server {
    let mut builder = ::tonic::transport::Server::builder();

    if let Some(limit) = args.concurrency_limit_per_connection {
        builder = builder.concurrency_limit_per_connection(limit);
    }

    if let Some(timeout) = args.timeout {
        builder.timeout(timeout);
    }

    if let Some(tcp_keepalive) = args.tcp_keepalive {
        builder = builder.tcp_keepalive(Some(tcp_keepalive));
    }

    #[cfg(feature = "traceable")]
    {
        use rand::distributions::Alphanumeric;
        use rand::Rng;
        builder = builder.trace_fn(|header| {
            tracing::info_span!(
                "server",
                "{}",
                header
                    .get("x-request-id")
                    .and_then(|id| id.to_str().ok())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| rand::thread_rng()
                            .sample_iter(&Alphanumeric)
                            .take(10)
                            .collect::<String>())
            )
        });
    }

    builder
}

/// Preparations before starting server
pub fn pre_server(_service_config: &config::Service) {
    #[cfg(feature = "telemetry")]
    {
        start_metrics_server(_service_config.metrics.clone());
    }
}

pub async fn trap_signal() {
    use futures::future::FutureExt;
    use tokio::signal::unix::{signal, SignalKind};

    let mut stream = signal(SignalKind::terminate()).expect("unable to trap signal");

    stream.recv().map(|o| o.unwrap_or(())).await
}

/// Starts the metrics server with configuration parameters.
///
/// # Panics
///
/// Panics if the hostname is invalid or if the server fails to start.
#[cfg(feature = "telemetry")]
fn start_metrics_server(config: metrics::Config) {
    tokio::spawn(metrics::start_server(config));
}
