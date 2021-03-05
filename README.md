# MrBig Just-on-Time Cloud-Native Solutions Crafting

![Rust](https://github.com/bobbie-ai/mrbig/workflows/Rust/badge.svg)

## Overview

`Mr. Big` is a (embryonic, for now) flexible and fast framework for building and deploying cloud-native applications (CNAs) in a constant flow, without the
hassle of managing cloud stacks, control plane configurations, or the containerization (or serverless computing) burden. 

`Mr. Big` includes an API to ease the design and implementation of CNAs based on micro-service architecture and Rust, and to deploy such applications in a constant flow (or constant deployment model), using a [*hatch*](mrbig_hatch/README.md) for seamlessly glue the developer's local platform and the remote cloud-based production environment.

## Architecture

The `Mr. Big` ecosystem provides several components.

Check the READMEs for each of them:
* [mrbig_core](mrbig_core/README.md)
* [mrbig_derive](mrbig_derive/README.md)
* [mrbig_build](mrbig_build/README.md)
* [mrbig_hatch](mrbig_hatch/README.md)

## Getting Started

To build various components, please enter the following command:

```bash
$ cargo build --release
```

## Example

Checkout our working examples in the [examples](./mrbig_examples) folder.

## Testing

To run all tests, `cd` into the root of the repo and run:

```bash
$ cargo make test
```

To only run the unit tests, do instead:

```bash
$ cargo make unit-tests
```

## Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, shall be licensed according to the included license, without any additional terms or conditions.
If you want to contribute to `Mr. Big`, please read our [CONTRIBUTING notes](./CONTRIBUTING.md).
