// use passerine;
use proc_macro::TokenStream;
use quote::{
    quote,
    quote_spanned,
};
use syn::{
    parse_macro_input,
    spanned::Spanned,
    DeriveInput,
    Ident,
    Index,
};

#[proc_macro_derive(Inject)]
pub fn derive_inject(input: TokenStream) -> TokenStream {
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
                    if let passerine::Data::Unit = param {
                        Ok(#type_name)
                    } else {
                        Err(())
                    }
                };
                let into = quote! { passerine::Data::Unit };
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
        // Data -> Item conversion
        impl TryFrom<passerine::Data> for #type_name {
            type Error = ();
            fn try_from(param: passerine::Data) -> Result<Self, ()> { #from }
        }

        // Item -> Data conversion
        impl From<#type_name> for passerine::Data {
            fn from(param: #type_name) -> Self { #into }
        }

        // With the above two implemented,
        // we can implement inject automatically.
        impl passerine::Inject for #type_name {}
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
            #name: param.pop().ok_or(())?.try_into()?
        }
    });
    let into = fields.named.iter().map(|f| {
        let name = &f.ident;
        quote_spanned! { f.span() =>
            param.#name.into()
        }
    });

    let from = quote! {
        if let passerine::Data::Tuple(mut param) = param {
            if param.len() != #num_fields { return Err(()); }
            Ok(#type_name { #(#from,)* })
        } else {
            Err(())
        }
    };
    let into = quote! {
        passerine::Data::Tuple(vec![#(#into,)*])
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
            param.pop().ok_or(())?.try_into()?
        }
    });
    let into = fields.unnamed.iter().enumerate().map(|(index, f)| {
        let index = Index::from(index);
        quote_spanned! { f.span() =>
            param.#index.into()
        }
    });

    let from = quote! {
        if let passerine::Data::Tuple(mut param) = param {
            if param.len() != #num_fields { return Err(()); }
            Ok(#type_name (#(#from,)*))
        } else {
            Err(())
        }
    };
    let into = quote! {
        passerine::Data::Tuple(vec![#(#into,)*])
    };

    (from, into)
}
