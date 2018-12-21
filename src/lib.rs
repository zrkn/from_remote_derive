extern crate proc_macro;

use syn::{
    parse_macro_input, parse_quote,
    DeriveInput, Attribute, Type, Meta, Lit, Data, Fields, Ident, Field,
    NestedMeta,
    spanned::Spanned,
};
use quote::{quote, quote_spanned};
use proc_macro2::TokenStream;


#[proc_macro_derive(FromRemote, attributes(from_remote))]
pub fn derive_struct_from(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let impls = get_remote_name_from_attrs(&input.attrs)
        .into_iter()
        .map(|remote_name| from_impl(&input, remote_name));

    let expanded = quote! {
        #(#impls)*
    };

    proc_macro::TokenStream::from(expanded)
}

fn from_impl(input: &DeriveInput, remote_name: Type) -> TokenStream {
    let name = &input.ident;
    let constructor_impl = match &input.data {
        Data::Struct(data) => {
            match &data.fields {
                Fields::Named(fields) => {
                    let fields_match = named_match(fields.named.iter());
                    let fields_mapping = named_mapping(fields.named.iter());
                    quote! {
                        let #remote_name {
                            #fields_match
                        } = other;
                        #name {
                            #fields_mapping
                        }
                    }
                },
                Fields::Unnamed(fields) => {
                    let fields_match = unnamed_match(fields.unnamed.iter());
                    let fields_mapping = unnamed_mapping(fields.unnamed.iter());
                    quote! {
                        let #remote_name (
                            #fields_match
                        ) = other;
                        #name (
                            #fields_mapping
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
                        let fields_match = named_match(fields.named.iter());
                        let fields_mapping = named_mapping(fields.named.iter());
                        quote_spanned! { v.span() =>
                            #remote_name::#v_name { #fields_match } => #name::#v_name {
                                #fields_mapping
                            }
                        }
                    },
                    Fields::Unnamed(ref fields) => {
                        let fields_match = unnamed_match(fields.unnamed.iter());
                        let fields_mapping = unnamed_mapping(fields.unnamed.iter());
                        quote_spanned! { v.span() =>
                            #remote_name::#v_name(#fields_match) => #name::#v_name(
                                #fields_mapping
                            )
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

    quote! {
        impl From<#remote_name> for #name {
            fn from(other: #remote_name) -> Self {
                #constructor_impl
            }
        }
    }
}

fn named_match<'a>(fields: impl Iterator<Item = &'a Field>) -> TokenStream {
    let fields_match = fields.map(|f| {
        let f_name = &f.ident;
        quote_spanned! { f.span() =>
            #f_name
        }
    });
    quote! {
        #(#fields_match),*
    }
}

fn unnamed_match<'a>(fields: impl Iterator<Item = &'a Field>) -> TokenStream {
    let fields_match = fields.enumerate().map(|(i, f)| {
        let i = Ident::new(&format!("__{}", i), f.span());
        quote_spanned! { f.span() =>
            #i
        }
    });
    quote! {
        #(#fields_match),*
    }
}

fn named_mapping<'a>(fields: impl Iterator<Item = &'a Field>) -> TokenStream {
    let fields_mapping = fields.map(|f| {
        let f_name = &f.ident;
        if is_collection_seq(f) {
            quote_spanned! { f.span() =>
                #f_name: #f_name.into_iter().map(Into::into).collect()
            }
        } else if is_collection_map(f) {
            quote_spanned! { f.span() =>
                #f_name: #f_name.into_iter().map(|(x, y)| (x.into(), y.into())).collect()
            }
        } else if is_monadic(f) {
            quote_spanned! { f.span() =>
                #f_name: #f_name.map(Into::into)
            }
        } else {
            quote_spanned! { f.span() =>
                #f_name: #f_name.into()
            }
        }
    });
    quote! {
        #(#fields_mapping),*
    }
}

fn unnamed_mapping<'a>(fields: impl Iterator<Item = &'a Field>) -> TokenStream {
    let fields_mapping = fields.enumerate().map(|(i, f)| {
        let i = Ident::new(&format!("__{}", i), f.span());
        if is_collection_seq(f) {
            quote_spanned! { f.span() =>
                #i.into_iter().map(Into::into).collect(),
            }
        } else if is_collection_map(f) {
            quote_spanned! { f.span() =>
                #i.into_iter().map(|(x, y)| (x.into(), y.into())).collect(),
            }
        } else if is_monadic(f) {
            quote_spanned! { f.span() =>
                #i.map(Into::into)
            }
        } else {
            quote_spanned! { f.span() =>
                #i.into()
            }
        }
    });
    quote! {
        #(#fields_mapping),*
    }
}

fn is_collection_seq(field: &Field) -> bool {
    let path = match &field.ty {
        Type::Path(p) => p,
        _ => return false,
    };
    let ident = match &path.path.segments.first() {
        Some(v) => &v.value().ident,
        None => return false,
    };
    let collection_idents: [Ident; 5] = [
        parse_quote!(Vec),
        parse_quote!(VecDeque),
        parse_quote!(LinkedList),
        parse_quote!(HashSet),
        parse_quote!(BTreeSet),
    ];
    collection_idents.contains(ident)
}

fn is_collection_map(field: &Field) -> bool {
    let path = match &field.ty {
        Type::Path(p) => p,
        _ => return false,
    };
    let ident = match &path.path.segments.first() {
        Some(v) => &v.value().ident,
        None => return false,
    };
    let collection_idents: [Ident; 2] = [
        parse_quote!(HashMap),
        parse_quote!(BTreeMap),
    ];
    collection_idents.contains(ident)
}

fn is_monadic(field: &Field) -> bool {
    let path = match &field.ty {
        Type::Path(p) => p,
        _ => return false,
    };
    let ident = match &path.path.segments.first() {
        Some(v) => &v.value().ident,
        None => return false,
    };
    let monadic_idents: [Ident; 2] = [
        parse_quote!(Option),
        parse_quote!(Result),
    ];
    monadic_idents.contains(ident)
}

fn get_remote_name_from_attrs(attrs: &[Attribute]) -> Vec<Type> {
    for attr in attrs {
        let path = match attr.path.segments.first() {
            Some(p) => p,
            None => continue,
        };

        if path.value().ident.to_string() != "from_remote" {
            continue
        }

        let list = match attr.parse_meta().unwrap() {
            Meta::List(l) => l,
            _ => continue,
        };

        return list.nested.iter()
            .filter_map(|meta| {
                match meta {
                    NestedMeta::Literal(Lit::Str(lit)) => {
                        Some(lit.parse::<Type>().unwrap())
                    }
                    _ => None
                }
            })
            .collect();
    }
    panic!("#[derive(FromRemote)] must be used with #[from_remote = \"???\"]")
}
