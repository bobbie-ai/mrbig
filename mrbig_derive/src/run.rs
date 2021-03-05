use proc_macro::TokenStream;
use syn::{parse_macro_input, Block, DeriveInput, Ident, ItemFn};

use crate::grpc::Arg;

pub(crate) fn derive(input: TokenStream) -> TokenStream {
    let d = parse_macro_input!(input as DeriveInput);

    Generate::new(d).into()
}

struct ServerHandler {
    stmt: syn::Stmt,
    ident: syn::Ident,
}

struct Interceptor {
    name: syn::Ident,
    flag: syn::Ident,
}

struct Generate {
    grpc_args: Vec<Arg>,
    ident: syn::Ident,
    disable_reflection: bool,
    disable_health: bool,
}

impl Generate {
    fn new(input: DeriveInput) -> Self {
        let DeriveInput { ident, attrs, .. } = input;

        let attr_disable_reflection = Ident::new("mrbig_disable_reflection", ident.span());
        let attr_disable_health = Ident::new("mrbig_disable_grpc_health", ident.span());
        let attr_register_grpc = Ident::new("mrbig_register_grpc", ident.span());

        // check if reflection is disabled
        let disable_reflection = attrs.iter().any(|a| {
            a.path
                .get_ident()
                .map(|id| *id == attr_disable_reflection)
                .unwrap_or(false)
        });

        // check if reflection is disabled
        let disable_health = attrs.iter().any(|a| {
            a.path
                .get_ident()
                .map(|id| *id == attr_disable_health)
                .unwrap_or(false)
        });

        // get the register_grpc_endpoint attribute arguments.
        let grpc_args: Vec<Arg> = attrs
            .into_iter()
            .filter(|a| {
                a.path
                    .get_ident()
                    .map(|id| *id == attr_register_grpc)
                    .unwrap_or(false)
            })
            .map(|a| {
                a.parse_args()
                    .expect("bad format for gRPC endpoint arguments")
            })
            .collect();

        Generate {
            grpc_args,
            ident,
            disable_reflection,
            disable_health,
        }
    }

    fn trait_name(&self) -> Ident {
        Ident::new(&format!("Run{}", self.ident), self.ident.span())
    }

    fn empty_run_function(&self) -> ItemFn {
        let span = self.ident.span();

        let generics: Vec<syn::GenericParam> = self
            .grpc_args
            .iter()
            .enumerate()
            .map(|(i, h)| {
                // generic type name
                let tname = Ident::new(&format!("T{}", i), span);
                let htrait = h.htrait.clone();
                parse_quote! { #tname: #htrait }
            })
            .collect();

        let fn_args: Vec<syn::ExprType> = self
            .grpc_args
            .iter()
            .enumerate()
            .map(|(i, _)| {
                // generic type name
                let tname = Ident::new(&format!("T{}", i), span);
                // service argument name
                let sname = Ident::new(&format!("s{}", i), span);
                parse_quote! { #sname: #tname }
            })
            .collect();
        parse_quote! {
            async fn run<#(#generics),*>(self, #(#fn_args),*) -> std::result::Result<(), ::mrbig_core::Error> {
                Err(::mrbig_core::Error::new("cannot run, not implemented"))
            }
        }
    }

    // Create the servers defined by the user
    fn user_servers(&self, icept: &Interceptor) -> Vec<ServerHandler> {
        let span = self.ident.span();

        let interceptor_name = &icept.name;
        let interceptor_flag = &icept.flag;

        self.grpc_args
            .iter()
            .enumerate()
            .map(|(i, h)| {
                let handler = &h.handler;
                let arg_name = Ident::new(&format!("s{}", i), span);
                let handler_name = Ident::new(&format!("handler{}", i), span);

                ServerHandler {
                    stmt: parse_quote! {
                        let #handler_name = match #interceptor_flag {
                            true => #handler::with_interceptor(#arg_name, #interceptor_name),
                            false => #handler::new(#arg_name),
                        };
                    },
                    ident: handler_name,
                }
            })
            .collect()
    }

    fn builtin_servers(&self) -> Vec<ServerHandler> {
        let mut ret = vec![];

        if !self.disable_reflection {
            ret.push(self.generate_reflection_handler());
        }

        if !self.disable_health {
            ret.push(self.generate_health_handler());
        }

        ret
    }

    fn generate_reflection_handler(&self) -> ServerHandler {
        // Create a list of services to reflect
        let reflected: Vec<&str> = self
            .grpc_args
            .iter()
            .filter(|a| a.reflect)
            .map(|a| a.service_fqn.as_str())
            .collect();

        // Create a list of builtin services to reflect
        let mut builtin_reflect: Vec<&str> = ["grpc.reflection.v1alpha.ServerReflection"]
            .iter()
            .copied()
            .collect();
        if !self.disable_health {
            builtin_reflect.push("grpc.health.v1.Health");
        }

        // Turn them into a punctuated list of string literals.
        use syn::punctuated::Punctuated;
        let mut lits: Punctuated<syn::LitStr, syn::token::Comma> = Punctuated::new();
        reflected
            .into_iter()
            .chain(builtin_reflect.into_iter())
            .map(|s| parse_quote! { #s })
            .for_each(|l| lits.push(l));

        // Turn it into a BTreeSet for filtering at run time.
        let create_filter: syn::Stmt = parse_quote! {
            let set: BTreeSet<&str> = [
                #lits
            ].iter().copied().collect();
        };

        // Generate code for creating the reflection server
        let ident = Ident::new("reflection", self.ident.span());
        let stmt: syn::Stmt = parse_quote! {
            let #ident = {
                mod reflection_descriptor {
                    use ::mrbig_core::grpc_reflection::{decode, DescriptorMap};
                    include!(concat!(env!("OUT_DIR"), concat!("/grpc_reflection_build_descriptor.rs")));
                }

                use ::mrbig_core::grpc_reflection::{Reflection, ServerReflectionServer};

                use std::collections::BTreeSet;
                #create_filter

                let services: Vec<String> = reflection_descriptor::SERVICES
                    .into_iter()
                    .filter(|&s| set.contains(s))
                    .map(|&s| s.into())
                    .collect();

                ::mrbig_core::log::trace!("reflecting services: {:?}", services);

                ServerReflectionServer::new(Reflection::new(
                    services,
                    reflection_descriptor::LazyDescriptorMap::new(),
                ))
            };
        };

        ServerHandler { ident, stmt }
    }

    fn generate_health_handler(&self) -> ServerHandler {
        use syn::punctuated::Punctuated;
        let mut lits: Punctuated<syn::LitStr, syn::token::Comma> = Punctuated::new();
        self.services_with_health()
            .into_iter()
            .map(|s| parse_quote! { #s })
            .for_each(|l| lits.push(l));

        // Generate code for creating the health server
        let ident = Ident::new("health_check", self.ident.span());
        let stmt: syn::Stmt = parse_quote! {
            let #ident = {
                use ::mrbig_core::context::WithContext;
        use ::mrbig_core::tonic_health::server::health_reporter;
        use ::mrbig_core::tonic_health::ServingStatus;

        let (mut reporter, health_server) = health_reporter();
        for svc in &[ #lits ] {
            reporter.set_service_status(svc, ServingStatus::Serving).await;
        }

                let context = micro.get_context_mut();
        context.set_health_reporter(reporter).await;

        health_server
            };
        };

        ServerHandler { ident, stmt }
    }

    fn services_with_health(&self) -> Vec<&str> {
        // List of user defined services to provide health for
        let with_health: Vec<&str> = self
            .grpc_args
            .iter()
            .filter(|a| a.health)
            .map(|a| a.service_fqn.as_str())
            .collect();

        // List of builtin services to provide health for
        let mut builtin_health: Vec<&str> = vec![];
        if !self.disable_reflection {
            builtin_health.push("grpc.reflection.v1alpha");
        };

        with_health
            .into_iter()
            .chain(builtin_health.into_iter())
            .chain(vec![""].into_iter())
            .collect()
    }

    fn init_method_block(&self, config_stmt: syn::Stmt) -> Block {
        parse_quote! {
        {
                    // Import locally to disambiguate trait methods
                    use ::mrbig_core::config::Configurable;
                    #config_stmt;

                    ::mrbig_core::init(self.get_config().unwrap())?;

                    let server = ::mrbig_core::new_grpc_server(
                        &self.get_config().unwrap().service.grpc_server
                    );

                    let context = self.get_context_mut();
                    context.set_server(server);

                    Ok(())
        }
        }
    }

    fn run_method_impl(&self) -> Block {
        let span = self.ident.span();

        let icept = Interceptor {
            name: Ident::new("interceptor", span),
            flag: Ident::new("will_intercept", span),
        };

        let servers: Vec<ServerHandler> = self
            .user_servers(&icept)
            .into_iter()
            .chain(self.builtin_servers().into_iter())
            .collect();

        // write the calls to create server handlers
        let (create_handlers, handler_list): (Vec<syn::Stmt>, Vec<syn::Ident>) =
            servers.into_iter().map(|s| (s.stmt, s.ident)).unzip();

        let Interceptor {
            name: interceptor_name,
            flag: interceptor_flag,
        } = icept;

        parse_quote! {
            {
                let mut micro = self;

                // Import locally to disambiguate trait methods
                use ::mrbig_core::config::Configurable;
                use ::mrbig_core::context::WithContext;

                let opts = match micro.take_config() {
                    Some(config) => config.service,
                    None => {
                        return Err(::mrbig_core::Error::new("service not initialized"));
                    }
                };

                let address = format!("{}:{}", opts.hostname, opts.port);

                let #interceptor_name = ::mrbig_core::default_grpc_interceptor;
                let #interceptor_flag = opts.debug;

                #(#create_handlers)*

                if opts.debug {
                    ::mrbig_core::log::debug!("serving at: {}", address);
                }

                ::mrbig_core::pre_server(&opts);

                let signal = ::mrbig_core::trap_signal();

                let mut builder = micro
                    .get_context_mut()
                    .take_server()
                    .ok_or(::mrbig_core::Error::new("no server available"))?;

                builder
                    #(.add_service(#handler_list))*
                .serve_with_shutdown(address.parse()?, signal)
                    .await
                    .map_err(|e| format!("server error: {}", e))?;

                ::mrbig_core::log::info!("gracefully shutting down");

                Ok(())
            }
        }
    }
}

impl Into<TokenStream> for Generate {
    fn into(self) -> TokenStream {
        // prepare a run trait for the service
        // an empty run implementation for the trait declaration.
        let fun_empty = self.empty_run_function();

        // a full implementation of the run method with the same signature.
        let mut fun_complete: ItemFn = fun_empty.clone();
        fun_complete.vis = syn::Visibility::Inherited;
        fun_complete.block = Box::new(self.run_method_impl());

        // the trait is specific to the struct being derived.
        let trait_name = self.trait_name();

        // prepare the init method default implementations
        let init_def = self.init_method_block(parse_quote! {
            self.load_from_args()?;
        });
        let init_with_args_def = self.init_method_block(parse_quote! {
            self.load_from_args_vec(args)?;
        });

        let ident = self.ident;

        TokenStream::from(quote! {
            #[allow(unused_variables, dead_code)]
            #[tonic::async_trait]
            trait #trait_name: ::mrbig_core::config::Configurable<'static> + ::mrbig_core::context::WithContext + Send + Sync + Sized + 'static {
            async fn init_with_args(&mut self, args: Vec<String>) -> ::std::result::Result<(), ::mrbig_core::Error> {
        #init_with_args_def
        }

            async fn init(&mut self) -> ::std::result::Result<(), ::mrbig_core::Error> {
                #init_def
        }

                #fun_empty
            }

            #[tonic::async_trait]
            impl #trait_name for #ident {
                #fun_complete
            }
        })
    }
}
