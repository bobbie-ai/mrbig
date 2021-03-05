#![warn(rust_2018_idioms)]

#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;
//extern crate proc_macro;

use proc_macro::TokenStream;

mod configurable;
mod grpc;
mod run;
mod service;

/// This derive macro both defines and implements a Run trait for a
/// given struct type.
///
/// The trait defined by this macro is specific and named after the
/// type of the struct it is being derived for.
///
/// So a snippet like `struct Foo { ... }` generates
/// `trait RunFoo { ... }`, and also
/// `impl RunFoo for Foo { ... }`.
///
/// It is this trait's implementation that actually sets up and runs
/// the server for your `Mr. Big` service.
///
/// The signature of the trait looks like:
/// ```ignore
/// #[allow(unused_variables, dead_code)]
/// #[tonic::async_trait]
/// trait RunFoo: ::mrbig_core::config::Configurable<'static> + Send + Sync + Sized + 'static {
///     fn init(&mut self) -> ::std::result::Result<(), ::mrbig_core::Error>;
///
///     async fn run<T1: Trait1, T2: Trait2, ...>(self, s1: T1, s2: T2, ...) -> std::result::Result<(), ::mrbig_core::Error>;
/// }
/// ```
///
/// where traits `Trait1`, `Trait2`, etc, depend on the register
/// endpoint macro attributes that were used.
///
/// The `init()` method is used for server initialization purposes,
/// mainly for parsing configuration parameters.
///
/// The `run()` method sets the server up and runs it.
///
/// # Attributes
///
/// ## Register gRPC endpoint
///
/// The `#[mrbig_register_grpc(service = "...", trait = "...",
/// reflection = true, health = true)]` is used to bind gRPC server handlers to the
/// microservice. `Mr. Big` currently depends on
/// [tonic](https://github.com/hyperium/tonic), so only types generated
/// by `tonic` are accepted.
///
/// The attribute takes three named arguments:
/// * `service`: fully qualified name of the gRPC proto service.
/// * `trait`: trait type from `tonic` generated code, that must be
/// implemented by user.
/// * `reflection`: boolean to enable/disable reflection for the gRPC
/// endpoint.
/// * `health`: boolean to enable/disable health check for the gRPC
/// endpoint.
///
/// At least one of `service` or `trait` arguments must be provided.
///
/// #### Using only `trait`
///
/// When `service` argument is not provided, the fully qualified name
/// of the gRPC proto service is inferred from the `trait`.
/// This works only if:
/// * the `trait` path has at least 3 segments.
/// * the first segment equals the name of the proto package.
/// * the last segment equals the name of the gRPC proto service.
///
/// For instance this will work:
/// * `trait` equals `helloworld::greeter_server::Greeter`
/// * proto package is called `helloworld`
/// * gRPC proto service is called `Greeter`
///
/// But this will not work:
/// * `trait` equals `health::health_server::Health`
/// * proto package is called `grpc.health.v1`
/// * gRPC proto service is called `Health`
/// * inferred `service` fully qualified name is `health.Health`,
/// which is wrong, it should be `grpc.health.v1.Health`. Both `trait`
/// and `service` arguments should be used in cases like this.
///
/// #### Using only `service`
///
/// When `trait` argument is not provided, the trait type path is
/// inferred from the fully qualified gRPC proto service.
/// This works only if:
/// * the `service` is a fully qualified path in the format
/// `<package_url>.<service>`.
/// * the `<service>` part of the path equals the name of the `trait`
/// type to infer.
/// * Both the `trait` type and the implementor type are in scope.
/// The implementor type is the trait type followed by *Server*
/// (trait `Greeter` implementor is `GreeterServer`)
///
/// ## Disable reflection
///
/// gRPC reflection can be disabled completely by adding the attribute
/// `#[mrbig_disable_reflection]`.
///
/// ## Disable health check server
///
/// gRPC health check server can be disabled completely by adding the
/// attribute `#[mrbig_disable_grpc_health]`.
#[proc_macro_derive(
    Run,
    attributes(
        mrbig_register_grpc,
        mrbig_disable_reflection,
        mrbig_disable_grpc_health
    )
)]
pub fn derive_run_fn(input: TokenStream) -> TokenStream {
    run::derive(input)
}

/// This derive macro implements `mrbig_core::config::Configurable` trait
/// for a struct type.
///
/// Refer to `mrbig_core::config::Configurable` trait's documentation for
/// more details.
///
/// This macro accepts struct fields attributes to where the `Mr. Big`'s
/// specific config, and user defined config can be deserialized.
///
/// # Attributes
///
/// ## `#[mrbig_context]`
///
/// Only one field of the struct can be marked with this attribute.
///
/// The field marked with this attribute must be of type
/// `mrbig_core::Context`, otherwise compilation fails.
/// When the microservice's `init()` method is called (from trait Run),
/// `Mr. Big`'s context is used to store configuration parameters.
///
/// ## `#[mrbig_config_extra]`
///
/// Only one field of the struct can be marked with this attribute.
///
/// The field marked with this attribute must implement the trait
/// `serde_derive::Deserialize`.
/// When the microservice's `init()` method is called (from trait Run),
/// the user defined configuration parameters are deserialized into the
/// field marked with this attribute.
#[proc_macro_derive(Configurable, attributes(mrbig_config, mrbig_config_extra))]
pub fn derive_configurable_fn(input: TokenStream) -> TokenStream {
    configurable::derive(input)
}

#[proc_macro_attribute]
pub fn service_impl(args: TokenStream, item: TokenStream) -> TokenStream {
    service::impler(args, item)
}
