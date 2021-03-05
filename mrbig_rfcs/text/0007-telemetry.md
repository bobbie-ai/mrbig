- Feature Name: `telemetry`
- Start Date: 2020-05-01
- RFC PR: [mrbig-cloud/mrbig-rfcs#?](https://github.com/bobbie-ai/mrbig-rfcs/pull/?)
- MrBig Issue: [mrbig-cloud/mrbig#3](https://github.com/bobbie-ai/mrbig/issues/3)

# Summary
[summary]: #summary

Incorporate a configurable mechanism for producing telemetry metrics.

# Motivation
[motivation]: #motivation

> Why are we doing this? What use cases does it support? What is the expected outcome?

`Mr. Big` micro services must adopt good practices when it comes to observability.
An important aspect of observability is telemetry.
Incoming external requests must be used to produce relevant metrics.
Relevant metrics include counting the number of requests, measuring their durations, measuring error rates.

# Guide-level explanation
[guide-level-explanation]: #guide-level-explanation

> Explain the proposal as if it was already included in the language and you were teaching it to another Rust programmer. That generally means:
> 
> - Introducing new named concepts.
> - Explaining the feature largely in terms of examples.
> - Explaining how Rust programmers should *think* about the feature, and how it should impact the way they use Rust. It should explain the impact as concretely as possible.
> - If applicable, provide sample error messages, deprecation warnings, or migration guidance.
> - If applicable, describe the differences between teaching this to existing Rust programmers and new Rust programmers.

By default a `Mr. Big` micro service comes with telemetry disabled.

To enable telemetry, the `feature` `"telemetry"` must be enabled.

When **enabled**, `Mr. Big` produces metrics supported by [prometheus](https://prometheus.io/).

The metrics are available at a configurable endpoint (http://localhost:9090 by default) at the path `/metrics` (http://localhost:9090/metrics by default).

## Metrics format

The format for the metrics is subject to change.

For a micro service named `welcome`, which implements the service `helloworld.Greeter` the metrics are:

```console
welcome_greeter_requests_total
welcome_greeter_request_duration_seconds
welcome_greeter_errors_total
```

honoring the format: `<micro_service_name>_<grpc_service_trait_name>_<metric_name>`.

## Enabling the metrics

Two things must be in place to enable the metrics for a specific
* The cargo feature `"telemetry"` must be enabled
* The gRPC trait implementation must be marked with a macro attribute `#[mrbig_service_impl]` (see example below)

```rust
#[derive(Debug, Default)]
pub struct MyGreeter {}

#[mrbig_service_impl]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
		// ...
    }
}
```

*(the use of the macro above implies `#[tonic::async_trait]`)*

# Reference-level explanation
[reference-level-explanation]: #reference-level-explanation

> This is the technical portion of the RFC. Explain the design in sufficient detail that:
> 
> - Its interaction with other features is clear.
> - It is reasonably clear how the feature would be implemented.
> - Corner cases are dissected by example.
> 
> The section should return to the examples given in the previous section, and explain more fully how the detailed proposal makes those examples work.

The main issues regarding reference level explanation are:
* Choosing a telemetry library
* Instrumenting the trait impl
* Serving the `/metrics` endpoint

## Choosing a telemetry library

The reasons for choosing [prometheus](https://prometheus.io/) as the tool for collecting metrics are:
* It is one of the most widely adopted tools for this purpose.
* In the kubernetes and service mesh ecosystems, many tools readily support exporting metrics to prometheus.
* Other visualization tools, such as grafana integrate well with it.

The most mature library for exporting metrics to prometheus in the Rust ecosystem seems to be [rust-prometheus](https://github.com/tikv/rust-prometheus) by [tikv](https://tikv.org/).

This crate is included as a dependency only when the cargo feature `"telemetry"` is enabled.

## Instrumenting the trait impl

The idea behind instrumenting the gRPC service trait implementation with macros, is allowing code to be injected in each implemented method.

Concretly, turning code such as:

```rust
#[mrbig_service_impl]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
		Ok(Response::new(HelloReply { name: request.into_inner().name }))
    }
}
```

into:

```rust
lazy_static::lazy_static! {
    static ref REQ_COUNTER: Counter = register_counter!(opts!(
        "welcome_greeter_requests_total",
        "Total number of requests made.",
        labels! {"handler" => "say_hello",}
    ))
    .unwrap();
    static ref REQ_HISTOGRAM: HistogramVec = register_histogram_vec!(
        "welcome_greeter_request_duration_seconds",
        "The request latencies in seconds.",
        &["handler"],
		vec![0.0001, 0.0002, 0.0003, 0.0005, 0.001]
    )
    .unwrap();
}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
		REQ_COUNTER.inc();
		let _timer = REQ_HISTOGRAM.with_label_values(&["say_hello"]).start_timer();

		Ok(Response::new(HelloReply { name: request.into_inner().name }))
    }
}
```

This approach registers the metrics in the global registry, using `lazy_static`, as suggested in the `rust-prometheus` example [here](https://github.com/tikv/rust-prometheus/blob/master/examples/example_hyper.rs).

# Serving the `/metrics` endpoint

Ideally, the same router used in `Mr. Big` (`tonic::transport::server::Router`) would be used route requests to `/metrics`.

The problem is that this router is HTTP2 only.
The server that `tonic` uses is not programmed to support HTTP requests upgrade (from 0.9 or even 1.1).

For instance, if using `curl` to perform the request, we must use the flag `--http2-prior-knowledge`.
Without this flag, the outcome is:
```console
* Received HTTP/0.9 when not allowed

* Closing connection 0
curl: (1) Received HTTP/0.9 when not allowed
```

Because of this, the port used by the micro service cannot be used for serving the metrics.
Prometheus supports upgrade to `http2`, but does not pull with it using prior knowledge.

Our suggestion here is to use an approach similar to [this example](https://github.com/tikv/rust-prometheus/blob/master/examples/example_hyper.rs).

The port bound for serving metrics must be configurable.

# Drawbacks
[drawbacks]: #drawbacks

> Why should we *not* do this?

The main drawback is increasing the dependencies footprint. The `rust-prometheus` and `lazy_static` crates bring added burden to both compile times and final binary size:

| Crate | version | size |
|------ | ------- | ---- |
| prometheus | 0.8.0 | 69.57 kB |
| lazy_static | 1.4.0 | 10.44 kB (mrbig already depends on this) |

It's why it is important to introduce this as a cargo feature which can be disabled.

# Rationale and alternatives
[rationale-and-alternatives]: #rationale-and-alternatives

> - Why is this design the best in the space of possible designs?

It is a simple and flexible design, leaves choices available to the user.

> - What other designs have been considered and what is the rationale for not choosing them?

The libraries chosen are the most mature for telemetry functionality.

Another design would be to rely on the service mesh for the metrics suggested above.

> - What is the impact of not doing this?

There is not an immediate urgency to implement this.

However, as the requests volume grows, the need to understand where the bottlenecks may be, grows as well.

Another added advantage of this undertaking is the instrumentation of the trait implementation.
Working on this opens room for further improvements and simplification.
The macro described above can also be used to inject [tracing](https://github.com/bobbie-ai/mrbig-rfcs/pull/1) calls, for instance.
