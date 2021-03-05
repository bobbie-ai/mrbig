# Mr Big Core

The core library contains `Mr. Big`'s runtime code.

<!-- markdown-toc start - Don't edit this section. Run M-x markdown-toc-refresh-toc -->
**Table of Contents**

- [Mr Big Core](#mr-big-core)
- [Configuration](#configuration)
    - [Using default](#using-default)
    - [Specifying fields in the struct](#specifying-fields-in-the-struct)
    - [Manually implementing `mrbig_core::config::Configurable` trait](#manually-implementing-mrbigcoreconfigconfigurable-trait)
    - [From Vec of Strings](#from-vec-of-strings)
- [Context](#context)
    - [Creating a service instance](#creating-a-service-instance)

<!-- markdown-toc end -->

# Configuration

The struct you define to hold the microservice's data has to be configurable in some way. When the `init()` method is called, the configuration data is deserialized into that struct. Configuration data may come from command line arguments and from a TOML file.

You have three options to handle configuration:
* Using default values and let `Mr. Big` handle the configuration data.
* Specifying fields in the struct to hold the configuration data.
* Manually implementing the `mrbig_core::config::Configurable` trait.

## Using default

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
	service.init().await?;

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

## From Vec of Strings

You can initialize the service from a `Vec<String>` instead of loading from command line arguments (useful for testing):

```rust
// ...
async fn main() -> Result<(), String> {
    let service = Micro::default();

	let custom_args = vec![
		"micro".into(),
		"-c".into(),
		"/tmp/config.toml".into()
	];

	service.init_with_args(custom_args).await?;

    // ...
}
```

*(the first argument is assumed to be the name of the program being called)*

# Context

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
    service.init().await?;

    // Serve the endpoints
    service.run(MyGreeter::default()).await?;
    Ok(())
}
```

since the type `Context` implements the `Default` trait.
