use heck::{CamelCase, SnakeCase};
use proc_macro::TokenStream;
use syn::parse::{Parse, ParseStream, Result};
use syn::punctuated::Punctuated;
use syn::{parse_macro_input, Ident};

#[derive(Debug)]
pub struct Args {
    pub telemetry: bool,
    pub tracing: bool,
}

impl Parse for Args {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        // expect named arguments
        let fields: Punctuated<syn::MetaNameValue, Token![,]> =
            input.parse_terminated(syn::MetaNameValue::parse)?;

        let mut ident: Option<Ident> = None;
        let mut telemetry = false;
        let mut tracing = false;

        fields.into_iter().for_each(|meta| {
            let left = meta.path.get_ident().expect("LHS must be an identifier");

            // used for span purposes only
            ident = Some(left.clone());

            let right = meta.lit;

            let arg = ident.as_ref().unwrap().to_string();

            match arg.as_str() {
                "telemetry" => {
                    let value: syn::LitStr = parse_quote! { #right };
                    if value.value() == "true" {
                        telemetry = true;
                    }
                }
                "tracing" => {
                    let value: syn::LitStr = parse_quote! { #right };
                    if value.value() == "true" {
                        tracing = true;
                    }
                }
                a => unimplemented!("'{}' not supported", a),
            }
        });

        Ok(Args { telemetry, tracing })
    }
}

pub(crate) fn impler(args: TokenStream, input: TokenStream) -> TokenStream {
    let args: Args = parse_macro_input!(args as Args);
    let mut trait_impl = parse_macro_input!(input as syn::ItemImpl);

    // Allow only in trait impls
    if trait_impl.trait_.is_none() {
        panic!("macro only works for trait impl");
    }

    let trait_ident = trait_impl
        .trait_
        .as_ref()
        .unwrap()
        .1
        .segments
        .last()
        .expect("no name found for the trait type")
        .ident
        .clone();

    let trait_name = trait_ident.to_string();

    let impler_name = match trait_impl.self_ty.as_ref() {
        syn::Type::Path(ref inner) => inner
            .path
            .segments
            .last()
            .expect("no name found for the impler type")
            .ident
            .to_string(),
        _ => panic!("unsupported impler type: {:?}", trait_impl.self_ty),
    };

    // Create a metrics object even if telemetry is not enabled
    let metrics = Metrics::new(
        &format!(
            "{}_{}",
            impler_name.to_snake_case(),
            trait_name.to_snake_case()
        ),
        trait_ident,
    );

    for item in trait_impl.items.iter_mut() {
        if let syn::ImplItem::Method(ref mut method) = item {
            if args.tracing {
                add_tracing(method, &trait_name);
            }

            if args.telemetry {
                metrics.instrument(method);
            }
        }
    }

    let declarations = if args.telemetry {
        metrics.into_declarations()
    } else {
        vec![]
    };

    TokenStream::from(quote! {
    #(#declarations)*

        #[tonic::async_trait]
        #trait_impl
    })
}

#[derive(Debug)]
struct Metrics {
    // Used for span only
    ident: syn::Ident,
    counter_name: String,
    histogram_name: String,
    prefix: String,
}

impl Metrics {
    fn new(prefix: &str, ident: syn::Ident) -> Metrics {
        Metrics {
            ident,
            counter_name: format!("{}_REQ_COUNTER", prefix.to_ascii_uppercase()),
            histogram_name: format!("{}_REQ_HISTOGRAM", prefix.to_ascii_uppercase()),
            prefix: prefix.into(),
        }
    }

    fn instrument(&self, method: &mut syn::ImplItemMethod) {
        let inner_block = &method.block;

        let method_str = method.sig.ident.to_string();
        let method_lit = syn::LitStr::new(&method_str, method.sig.ident.span());

        let counter_ident = syn::Ident::new(&self.counter_name, method.sig.ident.span());
        let histogram_ident = syn::Ident::new(&self.histogram_name, method.sig.ident.span());

        method.block = parse_quote! {
            {
        #counter_ident.with(&::mrbig_core::prometheus::labels! {
            "method" => #method_lit,
        }).inc();

        let _histogram_timer = #histogram_ident.with(
            &::mrbig_core::prometheus::labels! {
            "method" => #method_lit,
            }).start_timer();

        #inner_block
            }
        };
    }

    fn into_declarations(self) -> Vec<syn::Stmt> {
        let counter_ident = syn::Ident::new(&self.counter_name, self.ident.span());
        let counter_literal = syn::LitStr::new(
            &format!("{}_requests_total", self.prefix.to_ascii_lowercase()),
            self.ident.span(),
        );

        let histogram_ident = syn::Ident::new(&self.histogram_name, self.ident.span());
        let histogram_literal = syn::LitStr::new(
            &format!(
                "{}_request_duration_seconds",
                self.prefix.to_ascii_lowercase()
            ),
            self.ident.span(),
        );

        vec![parse_quote! {
            ::mrbig_core::lazy_static! {
        static ref #counter_ident: ::mrbig_core::prometheus::IntCounterVec = ::mrbig_core::prometheus::register_int_counter_vec!(
            #counter_literal,
            "Total number of gRPC requests made per method.",
            &[ "method" ]
        )
            .unwrap();
        static ref #histogram_ident: ::mrbig_core::prometheus::HistogramVec = ::mrbig_core::prometheus::register_histogram_vec!(
            #histogram_literal,
            "Request latencies in seconds.",
            &[ "method" ]
        )
            .unwrap();
            }
        }]
    }
}

fn add_tracing(method: &mut syn::ImplItemMethod, trait_name: &str) {
    let inner_block = &method.block;

    let method_name_str = method.sig.ident.to_string();

    let method_lit = syn::LitStr::new(
        &format!("{}/{}", trait_name, &method_name_str.to_camel_case()),
        method.sig.ident.span(),
    );

    let outer_block: syn::Block = parse_quote! {
    {
            let _log_timer = ::std::time::Instant::now();
        let _wrap_response: ::std::result::Result<_, ::tonic::Status> = async move { #inner_block }.await;
            let _log_prefix = format!("{} ({:?})",
                      #method_lit,
                      _log_timer.elapsed());

            {
        use ::mrbig_core::log;
        use ::mrbig_core::ansi_term::Colour;

        match _wrap_response {
            Ok(_) => {
            log::debug!("{} -- {}", _log_prefix, Colour::Green.paint("OK"));
            }
            Err(ref e) => {
            log::debug!("{} -- {}: {}", _log_prefix, Colour::Red.paint("ERR"), e);
            }
        }
            }

            _wrap_response
    }
    };

    // Drop inner_block so we can mutate method
    let _ = inner_block;

    method.block = outer_block;
}
