# gRPC Reflection Server

## Overview

This crate provides an implementation of a gRPC reflection server.

It uses [tonic](https://github.com/hyperium/tonic) for server code scaffolding. 

It also exposes the trait `DescriptorMap` that an object must implement for creating a server instance. The object is then called when replying to `ServerReflectionRequest` (see [reflection.proto](https://github.com/grpc/grpc/blob/master/src/proto/grpc/reflection/v1alpha/reflection.proto)):
* `file_by_filename`: `DescriptorMap::by_filename()` is called
* `file_containing_symbol`: `DescriptorMap::by_symbol()` is called
* `list_services`: returns the list of services provided when the server instance was created
* `file_containing_extension`: not supported
* `all_extension_numbers_of_type`: not supported


To create an object that implements the `DescriptorMap` trait, check the `grpc_reflection_build` crate.

## Known Limitations

Protobuf version 2 extensions are not supported.
