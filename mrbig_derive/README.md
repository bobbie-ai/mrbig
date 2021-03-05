# Mr Big Derive Crate

Procedural macro crate for `Mr. Big` services.

All the code is this crate is relevant only at compile time.

<!-- markdown-toc start - Don't edit this section. Run M-x markdown-toc-refresh-toc -->
**Table of Contents**

- [Mr Big Derive Crate](#mr-big-derive-crate)
- [Service impl macro](#service-impl-macro)
    - [Default Tracing](#default-tracing)
    - [Custom Tracing](#custom-tracing)
    - [Prometheus metrics](#prometheus-metrics)
    - [Metrics format](#metrics-format)
    - [Enabling the metrics](#enabling-the-metrics)
- [gRPC reflection](#grpc-reflection)
- [gRPC Health](#grpc-health)
    - [Disabling health check server](#disabling-health-check-server)
    - [Behavior](#behavior)
    - [Changing status](#changing-status)

<!-- markdown-toc end -->

# Service impl macro

`Mr Big` provides the `mrbig_derive::service_impl` macro for injecting code in the gRPC service implementation. The following is an example of a `Greeter` service implementation from the gRPC package `helloworld`, using the macro:

```rust
#[mrbig_derive::service_impl(tracing = "true")]
impl Greeter for Welcome {
    async fn say_hello(
        &self,
        request: tonic::Request<HelloRequest>,
    ) -> Result<tonic::Response<HelloReply>, tonic::Status> {
        Ok(tonic::Response::new(HelloReply {
            message: format!("Hello {}", request.into_inner().name),
        }))
    }
}
```

The macro implies `#[tonic::async_trait]`.

This macro allows injecting code for tracing requests and producing telemetry. To enable them the features `"traceable"` and `"telemetry"` must be enabled, respectively, when importing the crate `mrbig_core`.

## Default Tracing

When **enabled**, by default `Mr. Big` creates a unique ID for the span based on the following headers:

* `x-request-id`
* `x-b3-traceid`
* `x-b3-spanId`

*(it may just be the `x-request-id` or a combination of the above)*

In your implementation of proto contract (`Greeter` for example), when you use log calls, such as `info!()`, span data is included in the logging message.

For example, the following code, in a micro service called `greeter`, when responding to a request with `x-request-id` header as `01234babeface`:

```rust
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        info!("Inbound request.");
		// ...
	}
```

may print:

```console
Apr 30 19:02:23.301  INFO greeter{01234babeface}: say_hello: Inbound request
```

*(note that the reference to the function `say_hello` may not be available, depending on library support)*

## Custom Tracing

To specify a custom tracing span, you must define one closure in the form of:

```rust
Fn(&http::header::HeaderMap) -> tracing::Span + Send + Sync + 'static
```

and set it to the service's `Context` with:

```rust
use mrbig_core::context::WithContext;

let mut service = Micro::default();

service.init().await.expect("failed to init service");

let my_trace_fn = |_| tracing::info_span!("mytracer", "mycontract");

service.get_context_mut().set_trace_fn(my_trace_fn);

// ...
service.run(...)
```

## Prometheus metrics

When **enabled**, `Mr. Big` produces metrics supported by [prometheus](https://prometheus.io/).

The metrics are available at a configurable endpoint (http://localhost:9090 by default) at the path `/metrics` (http://localhost:9090/metrics by default). The hostname and port can be configured with `mrbig_core::Config`.

## Metrics format

The format for the metrics is subject to change.

For a micro service named `welcome`, which implements the service `helloworld.Greeter` the metrics are:

```console
welcome_greeter_requests_total
welcome_greeter_request_duration_seconds
```

honoring the format: `<micro_service_name>_<grpc_service_trait_name>_<metric_name>`.

## Enabling the metrics

Two things must be in place to enable the metrics for a specific
* The cargo feature `"telemetry"` must be enabled
* The gRPC trait implementation must be marked with a macro attribute `#[mrbig_service_impl(telemetry = "true")]` (see example below)

```rust
#[derive(Debug, Default)]
pub struct MyGreeter {}

#[mrbig_service_impl(telemetry = "true")]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
		// ...
    }
}
```

# gRPC reflection

To know more about the benefits of using gRPC server reflection, please refer to [grpc-reflection](https://github.com/grpc/grpc/blob/master/doc/server-reflection.md).

gRPC server reflection is enabled by default in `Mr. Big`, assuming your microservice crate has a `build.rs` that compiles your protos with `mrbig_build::compile_protos(...)`:

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    mrbig_build::compile_protos(
        &["proto/helloworld.proto", "proto/other.proto"],
        &["proto/"],
    )?;
    Ok(())
}
```

And that you register at least one endpoint:

```rust
#[derive(Run, Configurable)]
#[mrbig_register_grpc(service = "helloworld.Greeter")]
pub struct Micro {}
```

You can **disable reflection for a single endpoint** when you register using a named argument:
```rust
#[mrbig_register_grpc(service = "helloworld.Greeter", reflection=false)]
```

You can also **disable reflection completely** with:
```rust
#[derive(Run, Configurable)]
#[mrbig_disable_reflection]
pub struct Micro {}
```

Reflection can be tested by using [grpc_cli](https://github.com/grpc/grpc/blob/master/doc/command_line_tool.md).

# gRPC Health

When you register at least one gRPC endpoint in a `Mr. Big` service, some default options are assumed.

One of them is that a gRPC health contract should be exposed, according to [this](https://github.com/grpc/grpc/blob/master/doc/health-checking.md). Make sure you've read the linked document to fully understand this feature. This allows your service to comply with approaches such as [this one for kubernetes](https://kubernetes.io/blog/2018/10/01/health-checking-grpc-servers-on-kubernetes/), or [this one for Gloo](https://docs.solo.io/gloo/latest/api/github.com/solo-io/gloo/projects/gloo/api/external/envoy/api/v2/core/health_check.proto.sk/#grpchealthcheck).

## Disabling health check server

This health check feature can be disabled globally at compile time:

```rust
use mrbig_derive::{Run, Configurable};

#[derive(Run, Configurable)]
#[mrbig_register_grpc(service = "helloworld.Greeter")]
#[mrbig_disable_grpc_health]
struct Micro {
    context: mrbig_core::Context,
}
```

Can also be disabled for a particular endpoint at compile time:
```rust
use mrbig_derive::{Run, Configurable};

#[derive(Run, Configurable)]
#[mrbig_register_grpc(service = "helloworld.Greeter", health = false)]
struct Micro {
    context: mrbig_core::Context,
}
```

## Behavior

Our default implementation of the health server has these properties:
* Replies with the server overall state when the `service` string in the `HealthCheckRequest` message is empty.
* The server overall state is `SERVING` by default, unless the micro service is not in the `run()` phase.
* Replies with `SERVING` for any registered service, unless the micro service is not in the `run()` phase.
* Replies with gRPC status `NOT_FOUND` for any unregistered service name.
* Service names format is `package_names.ServiceName` such as `grpc.health.v1.Health` (no wildcard support).
* Sends new message when service's status changes, while replying to the streaming health check call `Watch`.

## Changing status

To change the health status, you can create an instance of a `mrbig_core::HealthReporter` object. The micro service struct, which implements the `Run` trait, is used to obtain the reporter:

```rust
service.init().await.expect("failed to init service");

let reporter = service.get_context().get_health_reporter("helloworld.Greeter");

// To set the status to Serving
reporter.set_serving().await;
// To set the status to NotServing
reporter.set_not_serving().await;
```

Where the `service_name` ("helloworld.Greeter" in the example above) follows the format `package_names.ServiceName` (mentioned above). Use an empty string for overall service status. The call fails if the service does not exist.

In the following example, `MyGreeter` implementation replies sets the status to `NOT_SERVING` when the `HelloRequest` comes from `"John Doe"`:

```rust
#[derive(Debug, Default)]
pub struct MyGreeter {
    health_reporter: HealthReporter,
}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: tonic::Request<HelloRequest>,
    ) -> Result<tonic::Response<HelloReply>, tonic::Status> {
        let name = request.into_inner().name;
        if name == "John Doe" {
            self.health_reporter.set_not_serving().await;
        } else {
            self.health_reporter.set_serving().await;
        }

        Ok(tonic::Response::new(HelloReply {
            message: format!("Hello {}", name),
        }))
    }
}


#[derive(Run, Configurable, Default)]
#[mrbig_register_grpc(service = "helloworld.Greeter")]
struct Micro {
    context: mrbig_core::Context,
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let mut service = Micro::default();
    service.init().await?;

	let my_greeter = MyGreeter {
		health_reporter: service.get_context().get_health_reporter("helloworld.Greeter"),
	};

    // Serve the endpoints
    service.run(my_greeter).await?;
    Ok(())
}
```
