use proc_macro::TokenStream;
use quote::{quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned, DeriveInput, Ident, Index};

/// A derive macro that generates an implementation of the `Inject` trait,
/// which allows a Rust type to be converted to Passerine data and back again.
/// This type is very important for building interfaces between Rust and
/// Passerine using system injection.
#[proc_macro_derive(Effect)]
pub fn derive_effect(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    let type_name = input.ident;

    let (from, into) = match input.data {
        syn::Data::Struct(ref data) => match data.fields {
            syn::Fields::Named(ref fields) => {
                derive_struct_named(&type_name, fields)
            },
            syn::Fields::Unnamed(ref fields) => {
                derive_struct_unnamed(&type_name, fields)
            },
            syn::Fields::Unit => {
                let from = quote! {
                    if let passerine_common::Data::Unit = param {
                        Some(#type_name)
                    } else {
                        None
                    }
                };
                let into = quote! { passerine_common::Data::Unit };
                (from, into)
            },
        },
        syn::Data::Enum(ref _data) => todo!(),
        syn::Data::Union(ref _data) => {
            unimplemented!("Unions are not supported")
        },
    };

    // Build the output, possibly using quasi-quotation
    let expanded = quote! {
        impl passerine_common::Inject for #type_name {
            fn serialize(param: Self) -> passerine_common::Data { #into }
            fn deserialize(param: passerine_common::Data) -> Option<Self> { #from }
        }

        // // Data -> Item conversion
        // impl TryFrom<passerine_common::Data> for #type_name {
        //     type Error = ();
        //     fn try_from(param: passerine_common::Data) -> Result<Self, ()> { #from }
        // }

        // // Item -> Data conversion
        // impl From<#type_name> for passerine_common::Data {
        //     fn from(param: #type_name) -> Self { #into }
        // }

        // // With the above two implemented,
        // // we can implement inject automatically.
        // impl passerine_common::Inject for #type_name {}
    };

    // Hand the output tokens back to the compiler
    TokenStream::from(expanded)
}

fn derive_struct_named(
    type_name: &Ident,
    fields: &syn::FieldsNamed,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let num_fields = fields.named.len();
    let from = fields.named.iter().rev().map(|f| {
        let name = &f.ident;
        quote_spanned! { f.span() =>
            #name: passerine_common::Inject::deserialize(param.pop()?)?
        }
    });
    let into = fields.named.iter().map(|f| {
        let name = &f.ident;
        quote_spanned! { f.span() =>
            passerine_common::Inject::serialize(param.#name)
        }
    });

    let from = quote! {
        if let passerine_common::Data::Tuple(mut param) = param {
            if param.len() != #num_fields { return None; }
            Some(#type_name { #(#from,)* })
        } else {
            None
        }
    };
    let into = quote! {
        passerine_common::Data::Tuple(vec![#(#into,)*])
    };

    (from, into)
}

fn derive_struct_unnamed(
    type_name: &Ident,
    fields: &syn::FieldsUnnamed,
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    let num_fields = fields.unnamed.len();
    let from = fields.unnamed.iter().rev().map(|f| {
        quote_spanned! { f.span() =>
            passerine_common::Inject::deserialize(param.pop()?)?
        }
    });
    let into = fields.unnamed.iter().enumerate().map(|(index, f)| {
        let index = Index::from(index);
        quote_spanned! { f.span() =>
            passerine_common::Inject::serialize(param.#index)
        }
    });

    let from = quote! {
        if let passerine_common::Data::Tuple(mut param) = param {
            if param.len() != #num_fields { return None; }
            Some(#type_name (#(#from,)*))
        } else {
            None
        }
    };
    let into = quote! {
        passerine_common::Data::Tuple(vec![#(#into,)*])
    };

    (from, into)
}
