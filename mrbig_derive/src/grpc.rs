use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::{Ident, Token, Type};

#[derive(Debug, Clone)]
pub struct Arg {
    pub handler: Type,
    pub htrait: Type,
    pub service_fqn: String,
    pub reflect: bool,
    pub health: bool,
}

impl Arg {
    // Assumes trait type and trait impler type are in scope,
    // and match the proto service name
    fn new_from_service_fqn(service_fqn: String, reflect: bool, health: bool) -> Result<Arg> {
        let basename = service_fqn
            .split('.')
            .last()
            .expect("service name is not fully qualified");

        let htrait = syn::parse_str::<Type>(basename)?;

        Arg::new_from_trait_and_service(htrait, service_fqn, reflect, health)
    }

    // create proto service fully qualified name from trait path
    // for a trait path "helloworld::greeter_server::Greeter" the
    // service fqn is helloworld.Greeter.
    // Only uses last two path segments
    fn new_from_trait(trt: syn::Type, reflect: bool, health: bool) -> Result<Arg> {
        let mut tpath: syn::TypePath = syn::parse_quote! {
            #trt
        };

        let msg =
            "trait path must have at least three path segments to infer service fully qualified name";

        let service = tpath.path.segments.pop().expect(msg).into_value();
        let _ = tpath.path.segments.pop().expect(msg);
        let package = tpath.path.segments.pop().expect(msg).into_value();

        let service_fqn = format!("{}.{}", package.ident, service.ident);

        Arg::new_from_trait_and_service(trt, service_fqn, reflect, health)
    }

    fn new_from_trait_and_service(
        trt: syn::Type,
        service_fqn: String,
        reflect: bool,
        health: bool,
    ) -> Result<Arg> {
        let mut tpath: syn::TypePath = syn::parse_quote! {
            #trt
        };

        {
            let mut last = tpath.path.segments.last_mut().unwrap();
            let span = last.ident.span();
            let name = format!("{}Server", last.ident);
            last.ident = Ident::new(&name, span);
        }

        let htrait: syn::Type = syn::parse_quote! { #trt };
        let handler: syn::Type = syn::parse_quote! { #tpath };

        Ok(Arg {
            htrait,
            handler,
            service_fqn,
            reflect,
            health,
        })
    }
}

impl Parse for Arg {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        // expect named arguments
        let fields: Punctuated<syn::MetaNameValue, Token![,]> =
            input.parse_terminated(syn::MetaNameValue::parse)?;

        let mut srv: Option<String> = None;
        let mut trt: Option<Type> = None;
        let mut rfl = true;
        let mut hlt = true;
        let mut ident: Option<Ident> = None;

        fields.into_iter().for_each(|mnv| {
            let left = mnv.path.get_ident().expect("LHS must be an identifier");

            // used for span purposes only
            ident = Some(left.clone());

            let right = mnv.lit;

            let arg = ident.as_ref().unwrap().to_string();

            match arg.as_str() {
                "service" => {
                    let name: syn::LitStr = parse_quote! { #right };
                    srv = Some(name.value());
                }
                "trait" => {
                    let lit = match right {
                        syn::Lit::Str(l) => l.value(),
                        t => panic!(format!("RHS argument for trait must be a literal: {:?}", t)),
                    };
                    let ty = syn::parse_str::<Type>(&lit).expect("RHS argument is not a type");
                    trt = Some(ty);
                }
                "reflection" => {
                    let boolean: syn::LitBool = parse_quote! { #right };
                    rfl = boolean.value;
                }
                "health" => {
                    let boolean: syn::LitBool = parse_quote! { #right };
                    hlt = boolean.value;
                }
                a => unimplemented!("'{}' not supported", a),
            }
        });

        let grpc_arg = match (srv, trt) {
            (None, None) => {
                return Err(syn::Error::new(
                    ident.unwrap().span(),
                    "must provide either 'service' and/or 'trait' arguments",
                ));
            }
            (Some(s), None) => Arg::new_from_service_fqn(s, rfl, hlt)?,
            (None, Some(t)) => Arg::new_from_trait(t, rfl, hlt)?,
            (Some(s), Some(t)) => Arg::new_from_trait_and_service(t, s, rfl, hlt)?,
        };

        Ok(grpc_arg)
    }
}
