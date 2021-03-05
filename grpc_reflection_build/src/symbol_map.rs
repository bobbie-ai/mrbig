use prost_types::{
    DescriptorProto, EnumDescriptorProto, FieldDescriptorProto, FileDescriptorProto,
    ServiceDescriptorProto,
};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) struct SymbolMap<'s> {
    inner: BTreeMap<String, &'s str>,
    services: BTreeSet<String>,
    processed: BTreeSet<&'s str>,
    dependencies: BTreeSet<&'s str>,
}

impl<'s> SymbolMap<'s> {
    pub(crate) fn new() -> Self {
        SymbolMap {
            inner: BTreeMap::new(),
            services: BTreeSet::new(),
            processed: BTreeSet::new(),
            dependencies: BTreeSet::new(),
        }
    }

    pub(crate) fn as_inner(&self) -> &BTreeMap<String, &'s str> {
        &self.inner
    }

    pub(crate) fn services(&self) -> Services {
        Services {
            inner: self.services.iter(),
        }
    }

    pub(crate) fn insert(&mut self, filename: &'s str, proto: &'s FileDescriptorProto) {
        // Check if has already been processed
        if self.processed.contains(filename) {
            return;
        }
        self.processed.insert(filename);

        // prefix for fully qualified name purposes
        let prefix = proto.package();

        // Insert messages
        proto
            .message_type
            .iter()
            .for_each(|m| self.insert_message(m, filename, prefix));

        // Insert enums
        proto
            .enum_type
            .iter()
            .for_each(|en| self.insert_enum(en, filename, prefix));

        // Insert extensions
        proto
            .extension
            .iter()
            .for_each(|ext| self.insert_field(ext, filename, prefix));

        // Insert services
        proto
            .service
            .iter()
            .for_each(|svc| self.insert_service(svc, prefix, filename));

        // Insert dependencies
        proto.dependency.iter().for_each(|dep| {
            self.dependencies.insert(dep);
        })
    }

    fn insert_symbol(&mut self, symbol: String, filename: &'s str) {
        self.inner.insert(symbol, filename);
    }

    fn insert_service(
        &mut self,
        service: &'s ServiceDescriptorProto,
        prefix: &str,
        filename: &'s str,
    ) {
        let service_name = fqn(prefix, service.name());

        // insert service as symbol
        self.insert_symbol(service_name.clone(), filename);
        // keep service name in a list for later
        self.services.insert(service_name.clone());

        service.method.iter().for_each(|method| {
            let method_name = fqn(&service_name, method.name());
            self.insert_symbol(method_name, filename)
        });
    }

    fn insert_message(&mut self, desc: &'s DescriptorProto, filename: &'s str, prefix: &str) {
        let msg_name = fqn(prefix, desc.name());

        self.insert_symbol(msg_name.clone(), filename);

        // Insert nested message types
        desc.nested_type
            .iter()
            .for_each(|nested| self.insert_message(nested, filename, &msg_name));

        // Insert enum descriptors
        desc.enum_type
            .iter()
            .for_each(|en| self.insert_enum(en, filename, &msg_name));

        // Insert extensions
        desc.extension
            .iter()
            .for_each(|ext| self.insert_field(ext, filename, &msg_name));

        // Insert fields
        desc.field
            .iter()
            .for_each(|fld| self.insert_field(fld, filename, &msg_name));

        // Insert oneofdecl
        desc.oneof_decl.iter().for_each(|oof| {
            let one_of_name = fqn(&msg_name, oof.name());
            self.insert_symbol(one_of_name, filename);
        });
    }

    fn insert_enum(&mut self, en: &'s EnumDescriptorProto, filename: &'s str, prefix: &str) {
        let en_name = fqn(prefix, en.name());
        self.insert_symbol(en_name.clone(), filename);

        en.value.iter().for_each(|v| {
            let v_name = fqn(&en_name, v.name());
            self.insert_symbol(v_name, filename);
        });
    }

    fn insert_field(&mut self, fld: &'s FieldDescriptorProto, filename: &'s str, prefix: &str) {
        let fld_name = fqn(prefix, fld.name());
        self.insert_symbol(fld_name, filename);
    }
}

// Panics if not all dependencies have been processed
impl Drop for SymbolMap<'_> {
    fn drop(&mut self) {
        self.dependencies.iter().for_each(|dep| {
            if !self.processed.contains(dep) {
                panic!("dependency {} was not found", dep);
            }
        });
    }
}

fn fqn(prefix: &str, name: &str) -> String {
    match prefix {
        "" => name.to_string(),
        _ => format!("{}.{}", prefix, name),
    }
}

pub(crate) struct Services<'a> {
    inner: std::collections::btree_set::Iter<'a, String>,
}

impl<'a> Iterator for Services<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|s| s.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use prost_types::MethodDescriptorProto;

    fn isolated_file() -> FileDescriptorProto {
        FileDescriptorProto {
            name: Some("helloworld.proto".into()),
            package: Some("helloworld".into()),
            message_type: vec![DescriptorProto {
                name: Some("HelloRequest".into()),
                field: vec![FieldDescriptorProto {
                    name: Some("name".into()),
                    type_name: Some("string".into()),
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        }
    }

    fn another_isolated_file() -> FileDescriptorProto {
        FileDescriptorProto {
            name: Some("void.proto".into()),
            package: Some("void".into()),
            message_type: vec![DescriptorProto {
                name: Some("VoidRequest".into()),
                field: vec![FieldDescriptorProto {
                    name: Some("name".into()),
                    type_name: Some("string".into()),
                    ..Default::default()
                }],
                ..Default::default()
            }],
            ..Default::default()
        }
    }

    fn with_service() -> FileDescriptorProto {
        let mut fdp = isolated_file();
        fdp.service = vec![ServiceDescriptorProto {
            name: Some("Greeter".into()),
            method: vec![MethodDescriptorProto {
                name: Some("SayHello".into()),
                input_type: Some("HelloRequest".into()),
                ..Default::default()
            }],
            ..Default::default()
        }];
        fdp
    }

    fn with_dependency() -> FileDescriptorProto {
        let mut fdp = another_isolated_file();
        fdp.dependency = vec!["helloworld.proto".into()];
        fdp
    }

    #[test]
    fn test_isolated() {
        let isol = isolated_file();
        let mut sym = SymbolMap::new();
        sym.insert(isol.name(), &isol);

        // no services
        assert_eq!(sym.services().count(), 0);

        let map = sym.as_inner();

        // fully qualified name of message type
        assert!(map.get("helloworld.HelloRequest").is_some());
    }

    #[test]
    fn test_with_service() {
        let with_srv = with_service();

        let mut sym = SymbolMap::new();
        sym.insert(with_srv.name(), &with_srv);

        // one service
        assert_eq!(sym.services().nth(0).unwrap(), "helloworld.Greeter");

        let map = sym.as_inner();

        // fully qualified name of service method
        assert!(map.get("helloworld.Greeter.SayHello").is_some());
    }

    #[test]
    fn test_ok_dependency() {
        let void = with_dependency();
        let hello = isolated_file();

        let mut sym = SymbolMap::new();
        sym.insert(void.name(), &void);
        sym.insert(hello.name(), &hello);

        // no services
        assert_eq!(sym.services().count(), 0);

        let map = sym.as_inner();

        // fully qualified name of message type
        assert!(map.get("helloworld.HelloRequest").is_some());
        // must not panic when going out of scope
    }

    #[test]
    #[should_panic(expected = "dependency helloworld.proto was not found")]
    fn test_broken_dependency() {
        let void = with_dependency();

        let mut sym = SymbolMap::new();
        sym.insert(void.name(), &void);
        // must panic when going out of scope
    }
}
