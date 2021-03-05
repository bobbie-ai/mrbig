pub static HEALTH: &str = r#"// Copyright 2015 The gRPC Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// The canonical version of this proto can be found at
// https://github.com/grpc/grpc-proto/blob/master/grpc/health/v1/health.proto

syntax = "proto3";

package grpc.health.v1;

option csharp_namespace = "Grpc.Health.V1";
option go_package = "google.golang.org/grpc/health/grpc_health_v1";
option java_multiple_files = true;
option java_outer_classname = "HealthProto";
option java_package = "io.grpc.health.v1";

message HealthCheckRequest {
  string service = 1;
}

message HealthCheckResponse {
  enum ServingStatus {
    UNKNOWN = 0;
    SERVING = 1;
    NOT_SERVING = 2;
    SERVICE_UNKNOWN = 3;  // Used only by the Watch method.
  }
  ServingStatus status = 1;
}

service Health {
  // If the requested service is unknown, the call will fail with status
  // NOT_FOUND.
  rpc Check(HealthCheckRequest) returns (HealthCheckResponse);

  // Performs a watch for the serving status of the requested service.
  // The server will immediately send back a message indicating the current
  // serving status.  It will then subsequently send a new message whenever
  // the service's serving status changes.
  //
  // If the requested service is unknown when the call is received, the
  // server will send a message setting the serving status to
  // SERVICE_UNKNOWN but will *not* terminate the call.  If at some
  // future point, the serving status of the service becomes known, the
  // server will send a new message with the service's serving status.
  //
  // If the call terminates with status UNIMPLEMENTED, then clients
  // should assume this method is not supported and should not retry the
  // call.  If the call terminates with any other status (including OK),
  // clients should retry the call with appropriate exponential backoff.
  rpc Watch(HealthCheckRequest) returns (stream HealthCheckResponse);
}
"#;

pub static REFLECTION: &str = r#"// Copyright 2016 gRPC authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

// Service exported by server reflection

syntax = "proto3";

package grpc.reflection.v1alpha;

service ServerReflection {
  // The reflection service is structured as a bidirectional stream, ensuring
  // all related requests go to a single server.
  rpc ServerReflectionInfo(stream ServerReflectionRequest)
      returns (stream ServerReflectionResponse);
}

// The message sent by the client when calling ServerReflectionInfo method.
message ServerReflectionRequest {
  string host = 1;
  // To use reflection service, the client should set one of the following
  // fields in message_request. The server distinguishes requests by their
  // defined field and then handles them using corresponding methods.
  oneof message_request {
    // Find a proto file by the file name.
    string file_by_filename = 3;

    // Find the proto file that declares the given fully-qualified symbol name.
    // This field should be a fully-qualified symbol name
    // (e.g. <package>.<service>[.<method>] or <package>.<type>).
    string file_containing_symbol = 4;

    // Find the proto file which defines an extension extending the given
    // message type with the given field number.
    ExtensionRequest file_containing_extension = 5;

    // Finds the tag numbers used by all known extensions of the given message
    // type, and appends them to ExtensionNumberResponse in an undefined order.
    // Its corresponding method is best-effort: it's not guaranteed that the
    // reflection service will implement this method, and it's not guaranteed
    // that this method will provide all extensions. Returns
    // StatusCode::UNIMPLEMENTED if it's not implemented.
    // This field should be a fully-qualified type name. The format is
    // <package>.<type>
    string all_extension_numbers_of_type = 6;

    // List the full names of registered services. The content will not be
    // checked.
    string list_services = 7;
  }
}

// The type name and extension number sent by the client when requesting
// file_containing_extension.
message ExtensionRequest {
  // Fully-qualified type name. The format should be <package>.<type>
  string containing_type = 1;
  int32 extension_number = 2;
}

// The message sent by the server to answer ServerReflectionInfo method.
message ServerReflectionResponse {
  string valid_host = 1;
  ServerReflectionRequest original_request = 2;
  // The server set one of the following fields accroding to the message_request
  // in the request.
  oneof message_response {
    // This message is used to answer file_by_filename, file_containing_symbol,
    // file_containing_extension requests with transitive dependencies. As
    // the repeated label is not allowed in oneof fields, we use a
    // FileDescriptorResponse message to encapsulate the repeated fields.
    // The reflection service is allowed to avoid sending FileDescriptorProtos
    // that were previously sent in response to earlier requests in the stream.
    FileDescriptorResponse file_descriptor_response = 4;

    // This message is used to answer all_extension_numbers_of_type requst.
    ExtensionNumberResponse all_extension_numbers_response = 5;

    // This message is used to answer list_services request.
    ListServiceResponse list_services_response = 6;

    // This message is used when an error occurs.
    ErrorResponse error_response = 7;
  }
}

// Serialized FileDescriptorProto messages sent by the server answering
// a file_by_filename, file_containing_symbol, or file_containing_extension
// request.
message FileDescriptorResponse {
  // Serialized FileDescriptorProto messages. We avoid taking a dependency on
  // descriptor.proto, which uses proto2 only features, by making them opaque
  // bytes instead.
  repeated bytes file_descriptor_proto = 1;
}

// A list of extension numbers sent by the server answering
// all_extension_numbers_of_type request.
message ExtensionNumberResponse {
  // Full name of the base type, including the package name. The format
  // is <package>.<type>
  string base_type_name = 1;
  repeated int32 extension_number = 2;
}

// A list of ServiceResponse sent by the server answering list_services request.
message ListServiceResponse {
  // The information of each service may be expanded in the future, so we use
  // ServiceResponse message to encapsulate it.
  repeated ServiceResponse service = 1;
}

// The information of a single service used by ListServiceResponse to answer
// list_services request.
message ServiceResponse {
  // Full name of a registered service, including its package name. The format
  // is <package>.<service>
  string name = 1;
}

// The error code and error message sent by the server when an error occurs.
message ErrorResponse {
  // This field uses the error codes defined in grpc::StatusCode.
  int32 error_code = 1;
  string error_message = 2;
}
"#;
