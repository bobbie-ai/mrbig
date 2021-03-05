- Feature Name: `grpc-health-check`
- Start Date: 2020-02-10
- RFC PR: [evooq/mrbig-rfcs#11](https://github.com/evooq/mrbig-rfcs/pull/11)
- MrBig Issue: [evooq/mrbig#6](https://github.com/evooq/mrbig/issues/6)

# Summary
[summary]: #summary

Add support for a `Mr. Big` gRPC micro service to implement a health check contract by default. The contract is defined [here](https://github.com/grpc/grpc/blob/master/doc/health-checking.md).

# Motivation
[motivation]: #motivation

Supporting a gRPC health contract by default is important for a gRPC based microservice:

* Uses a widely accepted format defined by google protobuf: health
* Other services or tools can expect the contract to be complied with and use it for monitoring purposes.
* More specifically, meshing frameworks like gloo can use this for health checking

# Guide-level explanation
[guide-level-explanation]: #guide-level-explanation

> Explain the proposal as if it was already included in the language and you were teaching it to another Rust programmer. That generally means:
> 
> - Introducing new named concepts.
> - Explaining the feature largely in terms of examples.
> - Explaining how Rust programmers should *think* about the feature, and how it should impact the way they use Rust. It should explain the impact as concretely as possible.
> - If applicable, provide sample error messages, deprecation warnings, or migration guidance.
> - If applicable, describe the differences between teaching this to existing Rust programmers and new Rust programmers.

## Default health check

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

And can be disabled at runtime through a configuration parameter by setting `grpc_server.health_check` to `false`.

## Behavior

Our default implementation of the health server has these properties:
* Replies with the server overall state when the `service` string in the `HealthCheckRequest` message is empty.
* The server overall state is `SERVING` by default, unless the micro service is not in the `run()` phase.
* Replies with `SERVING` for any registered service, unless the micro service is not in the `run()` phase.
* Replies with gRPC status `NOT_FOUND` for any unregistered service name.
* Service names format is `package_names.ServiceName` such as `grpc.health.v1.Health` (no wildcard support).
* Sends new message when service's status changes, while replying to the streaming health check call `Watch`. Only the service name for the first `HealthCheckRequest` is considered, the ones that follow are ignored.

## Changing status

To change the health status, an instance of a `HealthSetter` object must be created first. The micro service struct, which implements the `Run` trait, is used to obtain the setter, with the call:

```rust
fn health_setter_for(&self, service_name: &str) -> Result<mrbig_core::HealthSetter>;
```

Where the `service_name` follows the format `package_names.ServiceName` (mentioned above). Use an empty string for overall service status. The call fails if the service does not exist.

The `HealthSetter` type implements the call:

```rust
fn set(&self, status: ServingStatus);
```

that can be used to set the serving status of the service given by `service_name`.

In the following example, `MyGreeter` implementation replies sets the status to `NOT_SERVING` when the `HelloRequest` comes from `"John Doe"`:

```rust
#[derive(Debug, Default)]
pub struct MyGreeter {
    health_setter: HealthSetter,
}

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: tonic::Request<HelloRequest>,
    ) -> Result<tonic::Response<HelloReply>, tonic::Status> {
        let name = request.into_inner().name;
        if name == "John Doe" {
            self.health_setter.set(ServingStatus::NotServing);
        } else {
            self.health_setter.set(ServingStatus::Serving);
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
    service.init()?;

	let my_greeter = MyGreeter {
		health_setter: service.health_setter_from("helloworld.Greeter"),
	};

    // Serve the endpoints
    service.run(my_greeter).await?;
    Ok(())
}
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

Implementing this features encompasses taking care of the following:
* Create a health thread that manages the state of the running services
  * This thread has a *receiver* channel that accepts *connections* for *getting* and *setting* the statuses.
  * A *getting connection* is made by the health check server.
  * A *setting connection* is made by a gRPC contract implementor wanting to change its status (via `HealthSetter`).
* Implement the gRPC health check server (using tonic).
  * On every `HealthRequest` a connection to the health thread must be made to query status, especially for the `Watch` call, which requires continuous status updates to be listened for.
* Define the `HealthSetter` type and implement `set()` call, which must connect to the health thread.

## Create a health thread

The health thread is used to centralize the statuses of all registered gRPC endpoints. It contains:
* A collection of the truth of all statuses (`BTreeMap` or similar)
* A *receiver* part of a channel to handle internal requests.
* The *sender* part of the same channel for cloning purposes by internal clients.
* A collection of `Watch`ers (internal clients), which must be given statuses updates when they change.

The thread should be listening on the channel, waiting for internal requests:
* Get requests, to which it answers immediately with the status of the requested service (or with not found status)
* Set requests, which it uses to update the collection of statuses. This requires no answer.
* Watch requests, which are first added to the list of watchers and then given the current status of the requested service. When status change, the watchers must be sent the new value. If the channel to a watcher is closed or send fails, then the watcher is removed from the list.

The `HealthSetter` and the health check server must contain a clone of the *sender* part of the channel. Therefore they are internal clients of the health thread.

## Implement the gRPC health check server

`tonic` can be used to create a gRPC health check server. Afterwards the handling of the health requests must be implemented.

This server must contain a clone of the *sender* end of the channel held by the health thread described above.

Whenever a new gRPC request comes in, a clone of this sender must be made to get the status of the requested service (regardless of it being a simple get or a watch request).

For `Watch` requests, the data sent through the channel to the health thread must contain a sender for a short-lived channel, whose lifetime is equal to the incoming gRPC connection. Care must be taken to gracefully close this channel, giving the health thread a chance to remove this client from the list of watchers.

*A list of available services can be kept in the health check server to avoid making requests when the `service_name` in the request is known to not exist. In this case the server replies with `NOT_FOUND` status.

## Define the `HealthSetter` type

The `HealthSetter` can be implemented to simply contain a clone of the sender for the health thread. Whenever the `set()` method is called, a message with the new status is sent through that channel.

# Drawbacks
[drawbacks]: #drawbacks

> Why should we *not* do this?

We should not do this if we think that gRPC micro service developers:
* Do not care about health checking
* Or do not care about gRPC specific health checking
* Or are OK with simply using liveness probes such as *telneting* a port, for health checking purposes.

# Rationale and alternatives
[rationale-and-alternatives]: #rationale-and-alternatives

> - Why is this design the best in the space of possible designs?

This is a rather clean design, in the sense that:
* (**low API compromise**) It makes no compromises on micro service's mutability, with regards to setting the health status. Everything depends on how we implement the `HealthSetter` type.
* (**flexibility**) Still allows the developer to handle health in any way he desires by simply disabling our implementation.
* (**future improvements**) Leaves room for future improvements if we decide to implement *smarter* health statuses, by providing mechanisms that use `HealthSetter` behind the scenes ([future](#future-possibilities)).

> - What other designs have been considered and what is the rationale for not choosing them?

We're open to consider other designs or changes to the currently proposed.

> - What is the impact of not doing this?

Not doing this heightens the burden of the developer by "forcing him" to implement a health check standard or to subdue to using *unmeaningful* heatlh monitoring techniques, like HTTP or TCP socket probe. These methods test that the port is bound, and even that the server *responds to something*, but not necessarily that it is healthy from a business or technical perspective.

# Future possibilities
[future-possibilities]: #future-possibilities

Future possibilities for this feature are:
* Letting `Mr. Big` assess overall status by monitoring the running process.
* Further leveraging on macros to make changing the status simpler than in the example shown above in [guide-level-explanation](#guide-level-explanation). Simpler in the sense that the `set()` method would be available in `self` without requiring the developer to explicitly create a `HealthSetter` for the `MyGreeter` type.
