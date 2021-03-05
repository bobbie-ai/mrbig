- Feature Name: `tracing`
- Start Date: 2020-04-30
- RFC PR: [mrbig-cloud/mrbig-rfcs#1](https://github.com/bobbie-ai/mrbig-rfcs/pull/1)
- MrBig Issue: [mrbig-cloud/mrbig#?](https://github.com/bobbie-ai/mrbig/issues/?)

# Summary
[summary]: #summary

Incorporate a configurable mechanism for tracing requests.

# Motivation
[motivation]: #motivation

> Why are we doing this? What use cases does it support? What is the expected outcome?

`Mr. Big` micro services must adopt good practices when it comes to observability. One important aspect of observability is tracing. An incoming external request must be uniquely traced from the moment it enters our cluster to when a replied given back to it.

We expect the text logs in `Mr. Big` services to contain unique IDs which can allow the developers to easily follow the path described by the request.

A more in depth description of how tracing helps can be found [here](https://docs.lightstep.com/docs/understand-distributed-tracing).

How the service mesh istio deals with it, is described [here](https://istio.io/faq/distributed-tracing/).

# Guide-level explanation
[guide-level-explanation]: #guide-level-explanation

> Explain the proposal as if it was already included in the language and you were teaching it to another Rust programmer. That generally means:
> 
> - Introducing new named concepts.
> - Explaining the feature largely in terms of examples.
> - Explaining how Rust programmers should *think* about the feature, and how it should impact the way they use Rust. It should explain the impact as concretely as possible.
> - If applicable, provide sample error messages, deprecation warnings, or migration guidance.
> - If applicable, describe the differences between teaching this to existing Rust programmers and new Rust programmers.

By default a `Mr. Big` micro service comes with tracing disabled.

To enable tracing, the `feature` `"tracing"` must be enabled and the tracing config parameter must be enabled.

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

service.init().expect("failed to init service");

let my_trace_fn = |_| tracing::info_span!("mytracer", "mycontract");

service.get_context_mut().set_trace_fn(my_trace_fn);

// ...
service.run(...)
```

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
* Choosing tracing libraries
* Enable only when `tracing` feature is present
* Allowing `env_logger` filter format
* Expanding `Context` to support setting the trace_fn closure
* Ignoring traces from builtin servers

## Choosing tracing libraries

The most mature libraries for tracing in rust are the [tracing](https://crates.io/crates/tracing) crates, by the guys from [tokio-rs](https://github.com/tokio-rs). They provide the necessary features and flexibility for the suggested changes.

## Enable only when tracing feature is present

We use Rust's feature `#[cfg()]` macros to enable the default trace function, which uses `tracing` crates functionalities.

When the user disables tracing by `Config`, or provides his custom trace function, the `"tracing"` cargo feature can be disabled and the `tracing` crates need not be imported.

## Allowing env_logger filter format

The crate `tracing_subscriber` supports the `env_logger` filter format by using the subscriber `tracing_subscriber::filter::EnvFilter`. Moreover, when the macros such `logger::info!()` are used, the span information is injected, so it becomes equivalent to using `tracing::info!()` macros.

## Expanding Context to support setting the trace_fn closure

This requires further investigation and some testing.

The point is to use the `Context` struct to hold a reference to a closure of the type: `Fn(&http::header::HeaderMap) -> tracing::Span + Send + Sync + 'static`. Since we're using Rust, this may require boxing and other tricks, but should be possible to do.

The function to which this reference points to when tracing is disabled, is a no-op.

The developer can change the function this reference points to, we can call that custom tracing.

When tracing is enabled, the default function is:
```rust
let trace_fn = |header: &HeaderMap| -> Span {
	let request_id = header
		.get("x-request-id")
		.unwrap_or(uuid::Uuid::new_v4().to_hyphenated());
	tracing::info_span!(service_name, "{}", request_id);
};
```

## Ignoring traces from builtin servers

The builtin servers at the time of writing are `health_check` and `grpc_reflection` servers. The traces coming from these services shall be ignored by default. However, the possibility shall remain to not ignore.

To do this, we use the configuration parameter `logger_filters` string handed to `tracing_subscriber::filter::EnvFilter` (or `env_logger` if we decide to use that instead). We can set the default value to be `"h2=warn,mrbig::grpc_reflection=warn,mrbig::health_check=warn"`. This translates to: only print warnings or errors for the modules `h2`, `mrbig::grpc_reflection` and `mrbig::health_check`.

# Drawbacks
[drawbacks]: #drawbacks

> Why should we *not* do this?

The main drawback is the increase the dependencies footprint. The `tracing` and `tracing_subscriber` crates bring added burden to both compile times and final binary size:

| Crate | version | size |
|------ | ------- | ---- |
| tracing | 0.1.3 | 53.59 kB |
| tracing_subscriber | 0.2.5 | 72.28 kB |

It's why it is important to leave introduce this as a cargo feature which can be disabled.

# Rationale and alternatives
[rationale-and-alternatives]: #rationale-and-alternatives

> - Why is this design the best in the space of possible designs?

It is a simple and flexible design, leaves choices available to the user.

> - What other designs have been considered and what is the rationale for not choosing them?

The libraries chosen are the most mature for distributed tracing functionality.

Another possibility would be developing this ourselves, which implies many hours of work with potentially none or very short benefits.

> - What is the impact of not doing this?

There may not be an immediate urgency to implement this. However, the moment we lack understanding of what is going on in our backend, and why a certain request is being handled improperly, not having distributed tracing will make things extremely hard. Moreover, it is not only a matter of being able to observe events within `Mr. Big` micro services, but also across the service mesh and down to the ingress gateway.
