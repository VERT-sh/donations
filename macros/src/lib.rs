use std::collections::HashMap;

use proc_macro2::{Span, TokenStream};
use syn::{Data, DeriveInput, parse::Parse, punctuated::Punctuated};

#[proc_macro_derive(IntoResponse, attributes(response))]
pub fn derive_into_response(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    match derive_into_response_inner(syn::parse_macro_input!(item as syn::DeriveInput)) {
        Ok(tokens) => tokens.into(),
        Err(err) => panic!("{}", err),
    }
}

#[allow(dead_code)]
struct Assignment {
    key: syn::Ident,
    eq: syn::Token![=],
    value: syn::Expr,
}

impl Parse for Assignment {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            key: input.parse()?,
            eq: input.parse()?,
            value: input.parse()?,
        })
    }
}

struct Assignments {
    assignments: Punctuated<Assignment, syn::Token![,]>,
}

impl Parse for Assignments {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            assignments: input.parse_terminated(Assignment::parse, syn::Token![,])?,
        })
    }
}

fn derive_into_response_inner(input: DeriveInput) -> Result<TokenStream, syn::Error> {
    let enum_name = &input.ident;
    let Data::Enum(ref data) = input.data else {
        panic!("IntoResponse can only be derived for enums");
    };

    let mut map: HashMap<syn::Ident, HashMap<syn::Ident, syn::Expr>> = HashMap::new();

    for variant in &data.variants {
        // get our helper attribute
        let response_attr = variant
            .attrs
            .iter()
            .find(|attr| attr.path().is_ident("response"));

        let assignments: Assignments = if let Some(attr) = response_attr {
            attr.parse_args()?
        } else {
            Assignments {
                assignments: Punctuated::from_iter(vec![
                    Assignment {
                        key: syn::Ident::new("code", Span::call_site()),
                        eq: syn::Token![=](Span::call_site()),
                        value: syn::parse_str("INTERNAL_SERVER_ERROR")?,
                    },
                    Assignment {
                        key: syn::Ident::new("hidden", Span::call_site()),
                        eq: syn::Token![=](Span::call_site()),
                        value: syn::parse_str("true")?,
                    },
                ]),
            }
        };

        let props: HashMap<syn::Ident, syn::Expr> = assignments
            .assignments
            .into_iter()
            .map(|assignment| (assignment.key, assignment.value))
            .collect();

        // insert into the map
        map.entry(variant.ident.clone()).or_default().extend(props);
    }

    let tokens = map
        .into_iter()
        .map(|(variant_name, props)| {
            let status_code = props
                .get(&syn::Ident::new("code", Span::call_site()))
                .cloned()
                .unwrap_or_else(|| syn::parse_str("INTERNAL_SERVER_ERROR").unwrap());

            let hidden = props
                .get(&syn::Ident::new("hidden", Span::call_site()))
                .cloned()
                .unwrap_or_else(|| syn::parse_str("false").unwrap());

            quote::quote! {
                Self::#variant_name { .. } => {
                    let status = ::axum::http::StatusCode::#status_code;
                    if #hidden {
                        (status, "An unknown error occurred.".to_string())
                    } else {
                        (status, self.to_string())
                    }
                }
            }
        })
        .collect::<Vec<_>>();

    Ok(quote::quote! {
        impl ::axum::response::IntoResponse for #enum_name {
            fn into_response(self) -> ::axum::response::Response {
                match self {
                    #(#tokens,)*
                    #[allow(unreachable_patterns)]
                    _ => {
                        let status = ::axum::http::StatusCode::INTERNAL_SERVER_ERROR;
                        (status, "An unknown error occurred.".to_string())
                    }
                }.into_response()
            }
        }
    })
}

#[proc_macro_attribute]
pub fn handler(
    _: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(item as syn::ItemFn);
    handler_inner(input).into()
}

fn handler_inner(mut input: syn::ItemFn) -> TokenStream {
    let mut orig = input.clone();
    orig.sig.ident = syn::Ident::new(&format!("{}_orig", input.sig.ident), input.sig.ident.span());
    let orig_ident = &orig.sig.ident;

    input.sig.output = syn::parse_quote! {
        -> impl ::axum::response::IntoResponse
    };

    let args = input
        .sig
        .inputs
        .iter()
        .map(|arg| {
            if let syn::FnArg::Typed(pat_type) = arg {
                *pat_type.pat.clone()
            } else {
                panic!("Expected a typed argument");
            }
        })
        .collect::<Vec<_>>();

    let new_body = quote::quote! {
        let result = (#orig_ident)(#(#args),*).await;
        match result {
            Ok(data) => {
                (::axum::http::StatusCode::OK, ::axum::Json(serde_json::json!({
                    "data": data
                })))
            },

            Err(e) => {
                let msg = e.to_string();
                let response = e.into_response();
                (response.status(), ::axum::Json(serde_json::json!({
                    "error": msg
                })))
            }
        }
    };

    input.block = syn::parse_quote! {
        {
            #new_body
        }
    };

    quote::quote! {
        #orig

        #input
    }
}
