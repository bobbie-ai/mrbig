- Feature Name: `context`
- Start Date: 2020-02-09
- RFC PR: [evooq/mrbig-rfcs#9](https://github.com/evooq/mrbig-rfcs/pull/9)
- MrBig Issue: [evooq/mrbig#37](https://github.com/evooq/mrbig/issues/37)

# Summary
[summary]: #summary

Incorporate a context field in the micro service for initialization and runtime purposes.

# Motivation
[motivation]: #motivation

> Why are we doing this? What use cases does it support? What is the expected outcome?

`Mr. Big` micro services need a field in the user defined struct, that allows the macros to both store and manipulate data for various purposes, at initialization and runtime. Some examples are:
* Store state and health
* Store clients to external APIs
* Store clients/handlers to pub/sub mechanisms
* Store the config when the user does not provide a field to store it.
* Store any other runtime data may be necessary in the future.

# Guide-level explanation
[guide-level-explanation]: #guide-level-explanation

> Explain the proposal as if it was already included in the language and you were teaching it to another Rust programmer. That generally means:
> 
> - Introducing new named concepts.
> - Explaining the feature largely in terms of examples.
> - Explaining how Rust programmers should *think* about the feature, and how it should impact the way they use Rust. It should explain the impact as concretely as possible.
> - If applicable, provide sample error messages, deprecation warnings, or migration guidance.
> - If applicable, describe the differences between teaching this to existing Rust programmers and new Rust programmers.

A `Mr. Big` micro service requires context, which is used to:
* Store state and health
* Store clients to external APIs
* Store clients/handlers to pub/sub mechanisms
* Store the config if necessary
* Store any other runtime data that may be necessary in the future.

Therefore, when you define a struct to hold the microservice's data, it must contain a field for holding the context:

```rust
use mrbig_derive::{Run, Configurable};

#[derive(Run, Configurable)]
#[mrbig_register_grpc(service = "helloworld.Greeter")]
struct Micro {
    context: mrbig_core::Context,
}
```

The field can have any name, as long as the type is `mrbig_core::Context` ([fully qualified name](https://en.wikipedia.org/wiki/Fully_qualified_name) is mandatory), or if the field has the attribute macro `#[mrbig_context]`:

```rust
use mrbig_core::Context;

/// (...)

struct Micro {
    `#[mrbig_context]`
    context: Context,
}
```

## Creating a service instance

If you define your specific fields, for instance:

```rust
use mrbig_derive::{Run, Configurable};

#[derive(Run, Configurable)]
#[mrbig_register_grpc(service = "helloworld.Greeter")]
struct Micro {
    context: mrbig_core::Context,
    my_field: String,
}
```

You can create an instance of the service by writing:

```rust
#[tokio::main]
async fn main() -> Result<(), String> {
    let mut service = Micro {
        my_field: "my specific text".into(),
        context: mrbig_core::Context::default(),
    };
    service.init()?;

    // Serve the endpoints
    service.run(MyGreeter::default()).await?;
    Ok(())
}
```

since the type `Context` implements the `Default` trait.

# Reference-level explanation
[reference-level-explanation]: #reference-level-explanation

> This is the technical portion of the RFC. Explain the design in sufficient detail that:
> 
> - Its interaction with other features is clear.
> - It is reasonably clear how the feature would be implemented.
> - Corner cases are dissected by example.
> 
> The section should return to the examples given in the previous section, and explain more fully how the detailed proposal makes those examples work.

There are two main issues regarding reference level explanation that must be addressed:
* Impact on configuration
  * Already defined trait `Configurable` must not change
  * Flexibility regarding user specific configuration must not change
* Dealing with the context field (implementation)

We start by addressing the impact on configuration, so that implementation becomes simpler to describe afterwards.

## Impact on configuration

The configuration feature has an edge case of the empty struct. In it, the user is not forced to define a `Mr. Big` specific config field, by writing:

```rust
#[derive(Run, Configurable)]
#[mrbig_register_grpc(service = "helloworld.Greeter")]
struct Micro {}
```

After implementing context, this example must not compile because of a missing context field. The correct example becomes:

```rust
#[derive(Run, Configurable)]
#[mrbig_register_grpc(service = "helloworld.Greeter")]
struct Micro {
    #[mrbig_context]
    context: mrbig_core::Context,
}
```

The edge case was particular because the `init()` could be skipped, since it had nowhere in the struct to place the configuration data. With context available, the derive `Configurable` trait can use a field within `Context` to store the configuration data.

The `Context` type has `pub` visibility but private fields. Changes to its inner fields should never break the API for our users. Other fields will be added as needed by new features.

```rust
pub struct Context {
    /// private field
	config: Option<Config>,
}
```

#### Deprecating attribute `#[mrbig_config]`

The macro attribute `#[mrbig_config]` for a user defined field of type `mrbig_core::config::Config` becomes pointless. Because `context` is now available for storing `Mr. Big`'s specific configuration data. There should be no particular reason for the user to need to use the attribute, since the trait `Configurable` is implemented by the micro service struct. If the user needs to change or read `Mr. Big`'s specific configuration data, he should use the trait, rather than mutating a field of type `mrbig_core::config::Config` in the user defined struct.

Even if such an edge case exists, which we advise against, the user can manually implementing the `Configurable` trait. This way, configuration data can be stored wherever the user desires.

## Dealing with the context field (implementation)

`context` field is now mandatory which means:

* Compilation must fail when it is not present in the user defined struct for the microservice
* Since the struct is identified by the use of the derive macro `Run`, the derive function must panic when the context field is not present or of the wrong type.
* Derived traits can always assume that the field exists.

The derive `Run` trait must find the context field and use it for intended purposes, except configuration related. For configuration purposes, the derive function should rely on the `Configurable` trait and use its methods to access config.

The derive `Configurable` trait must find the context field and use it to store `Mr. Big` specific configuration data. It must also disregard the macro attribute `#[mrbig_config]` and even fail in case it's found, since it is now deprecated.

The type `mrbig_core::Context` must implement the `Default` trait, allowing the user to build a default instance of the user defined struct.

# Drawbacks
[drawbacks]: #drawbacks

> Why should we *not* do this?

The main drawback is breaking the current API and having to fix a number of tests and examples already in place. Since those snippets did not consider the existence of the context field.

Another drawback is imposing a field to be of a certain type in a user defined struct, which is not a common thing to impose.

Yet another drawback is changing some of the already defined interface for configuration related purposes.

# Rationale and alternatives
[rationale-and-alternatives]: #rationale-and-alternatives

> - Why is this design the best in the space of possible designs?

It is a simple and flexible design which does not impose significant burden on the user.

> - What other designs have been considered and what is the rationale for not choosing them?

Another possibility for this feature is have context being a trait that the user defined struct must implement.

The trait would be similar to the `Configurable` trait, and look something like:

```rust
use mrbig_core::Context;

pub trait WithContext {
    fn get_context(&self) -> &Context;

    fn get_context_mut(&mut self) -> &mut Context;

    fn set_context(&mut self, ctx: Context);
}
```

This does not avoid having a field within the user defined struct to store context. This approach would simply let the user decide where and how to store it. But it imposes yet another trait in the struct which now must implement: `Run`, `Configurable`, `WithContext`.

> - What is the impact of not doing this?

There currently is no place in the user defined struct to store whatever may be necessary by `Mr. Big`. Sooner or later, not having such a place will force us to break user API to implement some features. Not implementing context now is losing an opportunity.

