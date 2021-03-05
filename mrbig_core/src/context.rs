use crate::config::Config;
#[cfg(feature = "grpc")]
use tonic_health::{ServingStatus, server::{HealthReporter as TonicHealthReporter}};
#[cfg(feature = "grpc")]
use futures::lock::Mutex;
#[cfg(feature = "grpc")]
use std::sync::Arc;

/// A `Mr. Big` micro service requires context, which is used to:
/// * Store state and health
/// * Store clients to external APIs
/// * Store clients/handlers to pub/sub mechanisms
/// * Store the config if necessary
/// * Store any other runtime data that may be necessary in the future.
///
/// Therefore, when you define a struct to hold the microservice's
/// data, it must contain a field for holding the context:
///
/// ```ignore
/// use mrbig_derive::{Run, Configurable};
///
/// #[derive(Run, Configurable)]
/// #[mrbig_register_grpc(service = "helloworld.Greeter")]
/// struct Micro {
///     context: mrbig_core::Context,
/// }
/// ```
///
/// The field can have any name, as long as the type is `mrbig_core::Context`
/// ([fully qualified name](https://en.wikipedia.org/wiki/Fully_qualified_name)
/// is mandatory), or if the field has the attribute macro `#[mrbig_context]`:
///
/// ```ignore
/// use mrbig_core::Context;
///
/// // (...)
///
/// struct Micro {
///     #[mrbig_context]
///     context: Context,
/// }
/// ```
#[derive(Debug, Default)]
pub struct Context {
    config: Option<Box<Config>>,
    #[cfg(feature = "grpc")]
    health_reporter: Arc<Mutex<Option<Box<TonicHealthReporter>>>>,
    // Server by tonic to be built later by service.
    #[cfg(feature = "grpc")]
    server: Option<Box<tonic::transport::Server>>,
}

impl Context {
    /// Gets the inner config if any.
    ///
    /// *Not supposed to be used outside of mrbig_derive macros.*
    pub fn get_config(&self) -> Option<&Config> {
        self.config.as_ref().map(|b| b.as_ref())
    }

    /// Sets the inner config.
    ///
    /// *Not supposed to be used outside of mrbig_derive macros.*
    pub fn set_config(&mut self, config: Config) {
        self.config = Some(Box::new(config));
    }

    /// Takes the inner config.
    ///
    /// *Not supposed to be used outside of mrbig_derive macros.*
    pub fn take_config(&mut self) -> Option<Config> {
        self.config.take().map(|c| *c)
    }

    /// Set the health reporter handle.
    /// The handle can be shared with a service implementation,
    /// so it can be used to set the serving status of that service.
    ///
    /// *Not supposed to be used outside of mrbig_derive macros.*
    #[cfg(feature = "grpc")]
    pub async fn set_health_reporter(&mut self, reporter: TonicHealthReporter) {
	self.health_reporter.lock().await.replace(Box::new(reporter));
    }

    /// Get a health reporter for a specific service.
    /// The handle can be shared with a service implementation,
    /// so it can be used to set the serving status of that service.
    #[cfg(feature = "grpc")]
    pub async fn get_health_reporter(&self, svc: &str) -> HealthReporter {
	HealthReporter {
	    service: svc.into(),
	    handle: self.health_reporter.clone(),
	}
    }

    /// Sets the grpc transport server.
    #[cfg(feature = "grpc")]
    pub fn set_server(&mut self, server: tonic::transport::Server) {
        self.server = Some(Box::new(server));
    }

    /// Takes the grpc transport server.
    ///
    /// *Not supposed to be used outside of mrbig_derive macros.*
    #[cfg(feature = "grpc")]
    pub fn take_server(&mut self) -> Option<tonic::transport::Server> {
        self.server.take().map(|s| *s)
    }

    /// Sets the trace function for the grpc server.
    /// Function format is defined by tonic::transport::Server::trace_fn
    ///
    /// *Not supposed to be used outside of mrbig_derive macros.*
    ///
    /// # Panics
    /// Panics if the server has not yet been set.
    #[cfg(all(feature = "tracing", feature = "grpc"))]
    pub fn set_trace_fn<F>(&mut self, f: F)
    where
        F: Fn(&http::header::HeaderMap) -> tracing::Span + Send + Sync + 'static,
    {
        let mut server = *self.server.take().unwrap();
        server = server.trace_fn(f);
        self.server = Some(Box::new(server));
    }
}

pub trait WithContext {
    fn get_context(&self) -> &Context;
    fn get_context_mut(&mut self) -> &mut Context;
}


/// HealthReporter which mirrors tonic_health::server::HealthReporter API.
/// A handle providing methods to update the health status of gRPC services. A
/// `HealthReporter` is connected to a `HealthServer` which serves the statuses
/// over the `grpc.health.v1.Health` service.
#[cfg(feature = "grpc")]
#[derive(Clone, Debug)]
pub struct HealthReporter {
    service: String,
    handle: Arc<Mutex<Option<Box<TonicHealthReporter>>>>,
}

#[cfg(feature = "grpc")]
impl HealthReporter {
    async fn set_service_status(&self, status: ServingStatus) {
	self
	    .handle
	    .lock()
	    .await
	    .as_mut()
	    .expect("health is disabled")
	    .set_service_status(&self.service, status)
	    .await;
    }

    /// Sets the status of the service to `Serving`.
    /// This notifies any watchers if there is a change in status.
    ///
    /// # Panics
    /// Panics if health is disabled
    pub async fn set_serving(&self) {
	self.set_service_status(ServingStatus::Serving).await;
    }

    /// Sets the status of the service to `NotServing`.
    /// This notifies any watchers if there is a change in status.
    ///
    /// # Panics
    /// Panics if health is disabled
    pub async fn set_not_serving(&self) {
	self.set_service_status(ServingStatus::NotServing).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_set_config() {
        let mut ctx = Context::default();

        assert!(ctx.get_config().is_none());

        let config = Config::default();

        ctx.set_config(config.clone());

        assert!(ctx.get_config().is_some());

        let mref = ctx.get_config().unwrap();

        assert_eq!(mref.service.hostname, config.service.hostname);
    }
}
