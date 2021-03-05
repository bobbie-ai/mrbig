use crate::proto_includes::reflection;

use reflection::server_reflection_response::MessageResponse;

/// `DescriptorMap` defines a contract that a type must
/// implement to be used by a `Reflection` server
/// for replying to `ServerReflectionInfo` requests.
///
/// An implementor must provide the protobuf encoded
/// FileDescriptorProto message for a given file name
/// or for a given symbol.
pub trait DescriptorMap: Send + Sync + 'static {
    /// Return an encoded protobuf FileDescriptorProto
    /// message which contains `_symbol`.
    fn by_symbol(&self, _symbol: &str) -> Vec<Vec<u8>> {
        vec![vec![]]
    }

    /// Return an encoded protobuf FileDescriptorProto
    /// message for the file name `_filename`.
    fn by_filename(&self, _filename: &str) -> Vec<Vec<u8>> {
        vec![vec![]]
    }
}

pub struct InfoHandler<T: DescriptorMap> {
    service_names: Vec<reflection::ServiceResponse>,
    descriptor_map: T,
}

impl<T: DescriptorMap> InfoHandler<T> {
    pub fn new(service_names: Vec<String>, descriptor_map: T) -> Self {
        InfoHandler {
            service_names: service_names
                .into_iter()
                .map(|name| reflection::ServiceResponse { name })
                .collect(),
            descriptor_map,
        }
    }

    pub fn list_services(&self, _content: &str) -> MessageResponse {
        MessageResponse::ListServicesResponse(reflection::ListServiceResponse {
            service: self.service_names.clone(),
        })
    }

    pub fn file_containing_symbol(&self, symbol: &str) -> MessageResponse {
        MessageResponse::FileDescriptorResponse(reflection::FileDescriptorResponse {
            file_descriptor_proto: self.descriptor_map.by_symbol(symbol),
        })
    }

    pub fn file_by_filename(&self, filename: &str) -> MessageResponse {
        MessageResponse::FileDescriptorResponse(reflection::FileDescriptorResponse {
            file_descriptor_proto: self.descriptor_map.by_filename(filename),
        })
    }

    pub fn file_containing_extension(&self, _ext_typ: &str, _ext_num: i32) -> MessageResponse {
        MessageResponse::ErrorResponse(reflection::ErrorResponse {
            error_code: 12, // UNIMPLEMENTED
            error_message: "extensions not supported".into(),
        })
    }

    pub fn extension_numbers_of_type(&self, _ty: &str) -> MessageResponse {
        MessageResponse::ErrorResponse(reflection::ErrorResponse {
            error_code: 12, // UNIMPLEMENTED
            error_message: "extensions not supported".into(),
        })
    }
}
