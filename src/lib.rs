extern crate proc_macro;

use syn::{
    parse_macro_input, DeriveInput, Attribute, Type, Meta, Lit, Data, Fields, Ident,
    spanned::Spanned,
};
use quote::{quote, quote_spanned};
use proc_macro2::Span;


#[proc_macro_derive(FromRemote, attributes(from_remote))]
pub fn derive_struct_from(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;
    let remote_name = get_remote_name_from_attrs(&input.attrs);

    let constructor_impl = match input.data {
        Data::Struct(data) => {
            match data.fields {
                Fields::Named(fields) => {
                    let fields_mapping = fields.named.iter().map(|f| {
                        let f_name = &f.ident;
                        quote_spanned! { f.span() =>
                            #f_name: other.#f_name.into()
                        }
                    });
                    quote! {
                        #name {
                            #(#fields_mapping),*
                        }
                    }
                },
                Fields::Unnamed(fields) => {
                    let fields_mapping = fields.unnamed.iter().enumerate().map(|(i, f)| {
                        quote_spanned! { f.span() =>
                            other.#i.into()
                        }
                    });
                    quote! {
                        #name (
                            #(#fields_mapping),*
                        )
                    }
                },
                Fields::Unit => panic!("Unit structs are not supported by #[derive(FromRemote)]"),
            }
        },
        Data::Enum(data) => {
            let variants_mapping = data.variants.iter().map(|v| {
                let v_name = &v.ident;
                match v.fields {
                    Fields::Named(ref fields) => {
                        let fields_match = fields.named.iter().map(|f| {
                            let f_name = &f.ident;
                            quote_spanned! { f.span() =>
                                #f_name
                            }
                        });
                        let fields_mapping = fields.named.iter().map(|f| {
                            let f_name = &f.ident;
                            quote_spanned! { f.span() =>
                                #f_name: #f_name.into()
                            }
                        });
                        quote_spanned! { v.span() =>
                            #remote_name::#v_name { #(#fields_match),* } => #name::#v_name {
                                #(#fields_mapping),*
                            }
                        }
                    },
                    Fields::Unnamed(ref fields) => {
                        let fields_match = fields.unnamed.iter().enumerate().map(|(i, f)| {
                            let i = Ident::new(&format!("__{}", i), Span::call_site());
                            quote_spanned! { f.span() =>
                                #i
                            }
                        });
                        let fields_mapping = fields.unnamed.iter().enumerate().map(|(i, f)| {
                            let i = Ident::new(&format!("__{}", i), Span::call_site());
                            quote_spanned! { f.span() =>
                                #i.into()
                            }
                        });
                        quote_spanned! { v.span() =>
                            #remote_name::#v_name(#(#fields_match),*) => #name::#v_name(#(#fields_mapping),*)
                        }
                    },
                    Fields::Unit => {
                        quote_spanned! { v.span() =>
                            #remote_name::#v_name => #name::#v_name
                        }
                    }
                }
            });
            quote! {
                match other {
                    #(#variants_mapping),*
                }
            }
        },
        _ => panic!("Only structs and enums are supported by #[derive(FromRemote)]"),
    };

    let expanded = quote! {
        impl From<#remote_name> for #name {
            fn from(other: #remote_name) -> Self {
                #constructor_impl
            }
        }
    };

    proc_macro::TokenStream::from(expanded)
}

fn get_remote_name_from_attrs(attrs: &[Attribute]) -> Type {
    for attr in attrs {
        let path = match attr.path.segments.first() {
            Some(p) => p,
            None => continue,
        };

        if path.value().ident.to_string() != "from_remote" {
            continue
        }

        let meta = match attr.parse_meta().unwrap() {
            Meta::NameValue(m) => m,
            _ => continue,
        };

        if let Lit::Str(lit) = meta.lit {
            return lit.parse::<Type>().unwrap();
        }
    }
    panic!("#[derive(FromRemote)] must be used with #[from_remote = \"???\"]")
}
