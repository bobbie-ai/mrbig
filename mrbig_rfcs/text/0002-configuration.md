- Feature Name: `configuration`
- Start Date: 2020-01-27
- RFC PR: [evooq/mrbig-rfcs#4](https://github.com/evooq/mrbig-rfcs/pull/4)
- MrBig Issue: [evooq/mrbig#2](https://github.com/evooq/mrbig/issues/2)

# Summary
[summary]: #summary

Allow configuring the micro service using struct fields and macros.

# Motivation
[motivation]: #motivation

> Why are we doing this? What use cases does it support? What is the expected outcome?

The developer must be able to configure his micro service in an easy and nearly seamless way.

Given that the developer defines his own struct to hold the micro service's data, the option must be open for that struct to include the configuration fields. Furthermore, deserializing the config's fields into that object should be doable with macros.

# Guide-level explanation
[guide-level-explanation]: #guide-level-explanation

> Explain the proposal as if it was already included in the language and you were teaching it to another Rust programmer. That generally means:
> - Introducing new named concepts.
> - Explaining the feature largely in terms of examples.
> - Explaining how Rust programmers should *think* about the feature, and how it should impact the way they use Rust. It should explain the impact as concretely as possible.
> - If applicable, provide sample error messages, deprecation warnings, or migration guidance.
> - If applicable, describe the differences between teaching this to existing Rust programmers and new Rust programmers.

The struct you define to hold the microservice's data has to be configurable in some way. When the `init()` method is called, the configuration data is deserialized into that struct. Configuration data may come from command line arguments and from a TOML file.

You have three options to handle configuration:
* Using default values and let `Mr. Big` handle the configuration data.
* Specifying fields in the struct to hold the configuration data.
* Manually implementing the `mrbig_core::config::Configurable` trait.

## Using the default configuration

Simply add the `Configurable` derive macro to your struct declaration and call the `init()` method:

```rust
use mrbig_derive::{Run, Configurable};

#[derive(Run, Configurable, Default)]
#[mrbig_register_grpc(service = "helloworld.Greeter")]
struct Micro {
    context: mrbig_core::Context,
}

// Then make sure you call the init() method
#[tokio::main]
async fn main() -> Result<(), String> {
    let service = Micro::default();
	service.init()?;

    // Serve the endpoints
    service.run(MyGreeter::default()).await?;
    Ok(())
}
```

The actual configuration data is held within context in this example.

## Specifying fields in the struct

You can specify fields in your micro service struct which hold the configuration data, using macro attributes:

```rust
use mrbig_derive::{Run, Configurable};

#[derive(Run, Configurable)]
#[mrbig_register_grpc(service = "helloworld.Greeter")]
struct Micro {
	context: mrbig_core::Context,
	#[mrbig_config_extra]
	pub extra_config: MyExtras,
}
```

`Mr. Big`'s reserved configuration fields are deserialized into the `context` field, and your micro service's specific configuration fields are deserialized into the `extra_config` field. Therefore, the type `MyExtras` must implement the trait `serde_derive::Deserialize`.

The `Configurable` trait is still derived for you.

## Manually implementing `mrbig_core::config::Configurable` trait

The `Configurable` trait follows:

```rust
trait Configurable<'de> {
	fn get_config<'a>(&'a self) -> Option<&'a Config>;
	
	fn set_config(&mut self, config: Config);

	type Extra: serde::de::Deserialize<'de>;
	fn set_config_extra(&mut self, extra: Self::Extra);
}
```

Implementing the trait can be done for instance like:

```rust
#[derive(Deserialize)]
struct MyExtras {
	my_field: String;
}

#[derive(mrbig_derive::Run)]
#[mrbig_register_grpc(service = "helloworld.Greeter")]
struct Micro {
  context: mrbig_core::Context,
	rc: Arc<mrbig_core::config::Config>,
	spec: Arc<MyExtras>,
}

impl Configurable<'_> for Micro {
    type Extra = MyExtras;

    fn get_config<'a>(&'a self) -> Option<&'a Config> {
        Some(self.rc.as_ref())
    }

    fn set_config(&mut self, config: Config) {
        self.rc = Arc::new(config);
    }

    fn set_config_extra(&mut self, extra: Self::Extra) {
        self.spec = Arc::new(extra);
    }
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

The RFC imposes two main things on the API for our users:

* That the struct they define for the microservice implements the `Configurable` trait.
* That the `init()` is called before the `run()` (otherwise default config values are used).

## Deriving Configurable

The `Configurable` trait must be defined in `Mr. Big` core library with empty implementations for all methods.

To the `mrbig_derive` library we must add a call to derive `Configurable` while parsing the attribute `#[mrbig_config_extra]`.

If the attribute does not exist, an empty trait implementation is added:

```rust
#[derive(Deserialize)]
struct Void {};
impl Configurable<'_> for Micro { type Extra = Void; }
```

If the attribute **does** exist, then the field marked with `#[mrbig_config_extra]` shall be asserted to be of a type that implements `Deserialize`.

## The init() method

The already existing `Run` trait shall have a new method called `init()` with a signature such as:

```rust
fn init(&mut self) -> Result<(), Box<dyn Error>>;
```

(*we should consider `async` for a later use of a configuration API*)

The returned `Error` type may differ.

The `init()` method must first call `Configurable::get_config()`. If the `None` is returned, then the default values are stored in context. Otherwise, it must parse configuration from command line args and config file. Then it must use the trait calls `Configurable::set_config` and `Configurable::set_config_extra` to set the config fields.

When the `run()` method is called afterwards, the `Configurable::get_config()` cannot return `None`, in which case initialization was not performed and the service should panic.

# Drawbacks
[drawbacks]: #drawbacks

Keeping things extremely simple would be letting the user handle configuration for himself. The `run()` method would retrieve and parse configuration and be the only scope in the code where configuration is available. This would in turn prevent the user from accessing the configuration.

# Rationale and alternatives
[rationale-and-alternatives]: #rationale-and-alternatives

## Why is this design the best in the space of possible designs?

It may not be the best design but we believe it is the simplest and less intrusive one.

It allows the user to specify is own specific configuration type while still having access to `Mr. Big`'s config object. This object includes the server's port and hostname which may be useful to the user.

Default values and nearly *codeless* options are still available thanks to macros.

# What other designs have been considered and what is the rationale for not choosing them?

We can think of approaches that encapsulate rather than exposing, like:

```rust
// this object would include the config in `Mr. Big` core
let service = mrbig_core::Service::new();
// Extra type would have to implement Deserialize
let extra: Extra = service.get_extra();
```

But from a lifetime perspective it would not much total sense, since the config object in our `run()` method exists only for bootstrapping. At server runtime it is useless for `Mr. Big`, while the same may definitely not be true for our user. Our user may need to access configuration values when replying to server requests, therefore letting the user have **ownership** over the config objects seems more logical.

This would also force us to invert our current way of registering endpoints (which uses macros) and force us to do something like:
```rust
service
	.register(MyGreeter::new())
	.register(MyOtherGreeter::new())
```

which would be ok but would lead us to carbon copy a lot of code from the `tonic` crate (see how [tonic::transport::server::Router](https://docs.rs/tonic/0.1.1/tonic/transport/server/struct.Router.html) is implemented).

# What is the impact of not doing this?

The users will not be able to access `Mr. Big`'s or their own defined configuration objects.

# Future possibilities
[future-possibilities]: #future-possibilities

It is important to consider that later on we may have a means of exposing a config API (gRPC or REST), which would be up during the init phase. With the suggested approach, `Mr. Big`'s library API for our users would not change at all, the only difference would be that the `init()` may hold for an indefinite amount of time, until the configuration is ready. All `Mr. Big`'s configuration related work would be handled *behind the scenes* using macros.
