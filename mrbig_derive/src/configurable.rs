use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput, Field, Ident, Type};

// Attribute names to look for
const ATTR_CONFIG_EXTRA: &str = "mrbig_config_extra";
const ATTR_CONTEXT: &str = "mrbig_context";

#[derive(Default)]
struct InnerConfig {
    extra: Option<Field>,
    context: Option<Field>,
}

fn assert_unique(field: syn::Field, name: &str, is_some: bool) -> Option<Field> {
    if is_some {
        panic!("only one `{}` attribute is allowed per field", name);
    }
    Some(field)
}

fn is_of_context_type(field: &syn::Field) -> bool {
    let pre_col2: syn::Type = parse_quote! { ::mrbig_core::Context };
    let no_pre: syn::Type = parse_quote! { mrbig_core::Context };

    field.ty == pre_col2 || field.ty == no_pre
}

fn parse_config_attributes(data: syn::Data) -> InnerConfig {
    // Predicate to find the fields with the attributes we're looking for
    let predicate = |a: &syn::Attribute| {
        a.path.get_ident().and_then(|o| {
            let id = o.to_string();
            // Searching for these attributes
            match id.as_str() {
                ATTR_CONFIG_EXTRA => Some(ATTR_CONFIG_EXTRA),
                ATTR_CONTEXT => Some(ATTR_CONTEXT),
                _ => None,
            }
        })
    };

    let attributes_by_field = |p: syn::punctuated::Pair<syn::Field, syn::token::Comma>| {
        // return the field and a Vec of relevant attributes
        let field = p.into_value();
        let attrs = field
            .attrs
            .iter()
            .map(predicate)
            .filter(|o| o.is_some())
            .map(|o| o.unwrap())
            .collect::<Vec<&str>>();

        (field, attrs)
    };

    let inner = match data {
        syn::Data::Struct(ds) => match ds.fields {
            syn::Fields::Named(fields) => {
                let mut ret = InnerConfig::default();

                fields
                    .named
                    .into_pairs()
                    .map(attributes_by_field)
                    .map(|tup| {
                        if tup.1.len() > 1 {
                            panic!(
                                "field {} has multiple config attributes",
                                tup.0.ident.unwrap()
                            )
                        } else {
                            tup
                        }
                    })
                    .for_each(|tup| {
                        let mut attrs = tup.1;

                        if is_of_context_type(&tup.0) {
                            ret.context = Some(tup.0);
                            return;
                        }

                        if attrs.is_empty() {
                            return;
                        }

                        match attrs.remove(0) {
                            ATTR_CONTEXT => {
                                ret.context =
                                    assert_unique(tup.0, ATTR_CONTEXT, ret.context.is_some());
                            }
                            ATTR_CONFIG_EXTRA => {
                                ret.extra =
                                    assert_unique(tup.0, ATTR_CONFIG_EXTRA, ret.extra.is_some());
                            }
                            _ => unreachable!(),
                        }
                    });
                ret
            }
            _ => InnerConfig::default(),
        },
        _ => InnerConfig::default(),
    };

    if inner.context.is_none() {
        panic!("context is not defined",);
    }
    inner
}

fn configurable_trait_impl(ident: &Ident) -> syn::ItemImpl {
    let trait_name: syn::Type = parse_quote! { ::mrbig_core::config::Configurable };
    parse_quote! {
        #[allow(unused_variables, dead_code)]
        impl #trait_name<'_> for #ident { }
    }
}

fn get_method_configurable_impl(field: &Field) -> syn::ImplItemMethod {
    let member = field.ident.clone().unwrap();
    parse_quote! {
        fn get_config(&self) -> ::std::option::Option<&::mrbig_core::config::Config> {
            self.#member.get_config()
        }
    }
}

fn set_method_configurable_impl(field: &Field) -> syn::ImplItemMethod {
    let member = field.ident.clone().unwrap();
    parse_quote! {
        fn set_config(&mut self, config: ::mrbig_core::config::Config) {
            self.#member.set_config(config);
        }
    }
}

fn take_method_configurable_impl(field: &Field) -> syn::ImplItemMethod {
    let member = field.ident.clone().unwrap();
    parse_quote! {
        fn take_config(&mut self) -> Option<::mrbig_core::config::Config> {
            self.#member.take_config()
        }
    }
}

fn set_extra_method_configurable_impl(field: &Field) -> syn::ImplItemMethod {
    let member = field.ident.clone().unwrap();
    parse_quote! {
        fn set_config_extra(&mut self, extra: Self::Extra) {
            self.#member = extra;
        }
    }
}

fn extra_type_configurable_impl(ty: Type) -> syn::ImplItemType {
    parse_quote! { type Extra = #ty; }
}

fn with_context_trait_impl(ident: &Ident, field: &Field) -> syn::ItemImpl {
    let trait_name: syn::Type = parse_quote! { ::mrbig_core::context::WithContext };
    let member = field.ident.clone().unwrap();
    parse_quote! {
        #[allow(unused_variables, dead_code)]
        impl #trait_name for #ident {
            fn get_context(&self) -> &::mrbig_core::context::Context {
                &self.#member
            }

            fn get_context_mut(&mut self) -> &mut ::mrbig_core::context::Context {
                &mut self.#member
            }
         }
    }
}

pub(crate) fn derive(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input as DeriveInput);

    let inner = parse_config_attributes(data);

    let mut trait_impl = configurable_trait_impl(&ident);

    if let Some(extra) = inner.extra {
        let extra_type = extra.ty.clone();
        trait_impl
            .items
            .push(syn::ImplItem::Type(extra_type_configurable_impl(
                extra_type,
            )));
        trait_impl
            .items
            .push(syn::ImplItem::Method(set_extra_method_configurable_impl(
                &extra,
            )));
    } else {
        trait_impl
            .items
            .push(syn::ImplItem::Type(extra_type_configurable_impl(
                parse_quote! { ::mrbig_core::config::Void },
            )));
    }

    // context must exist
    let context = inner.context.unwrap();

    trait_impl
        .items
        .push(syn::ImplItem::Method(get_method_configurable_impl(
            &context,
        )));
    trait_impl
        .items
        .push(syn::ImplItem::Method(set_method_configurable_impl(
            &context,
        )));
    trait_impl
        .items
        .push(syn::ImplItem::Method(take_method_configurable_impl(
            &context,
        )));

    let with_context_impl = with_context_trait_impl(&ident, &context);

    TokenStream::from(quote! {
        #trait_impl

        #with_context_impl
    })
}
