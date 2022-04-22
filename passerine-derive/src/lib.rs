use passerine;
use proc_macro::TokenStream;
use quote::{
    quote,
    quote_spanned,
};
use syn::{
    parse_macro_input,
    spanned::Spanned,
    DeriveInput,
};

#[proc_macro_derive(Inject)]
pub fn derive_inject(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let (from, into) = match input.data {
        syn::Data::Struct(ref data) => match data.fields {
            syn::Fields::Named(ref fields) => derive_struct_named(fields),
            syn::Fields::Unnamed(_) => todo!(),
            syn::Fields::Unit => {
                let from = todo!();
                let into = todo!();
                (from, into)
            },
        },
        syn::Data::Enum(ref data) => todo!(),
        syn::Data::Union(ref data) => {
            unimplemented!("Unions are not supported")
        },
    };

    // Build the output, possibly using quasi-quotation
    let expanded = quote! {
        // Data -> Item conversion
        impl<'a> TryFrom<&'a passerine::Data> for #name {
            type Error = ();
            fn try_from(param: &'a passerine::Data) -> Result<Self, ()> { #from }
        }

        // Item -> Data conversion
        impl From<#name> for passerine::Data {
            fn from(param: #name) -> Self { #into }
        }

        // With the above two implemented,
        // we can implement inject automatically.
        impl<'a> passerine::Inject<'a> for #name {}
    };

    // Hand the output tokens back to the compiler
    TokenStream::from(expanded)
}

fn derive_struct_named(
    fields: &syn::FieldsNamed,
) -> (quote::__private::TokenStream, quote::__private::TokenStream) {
    let from = fields.named.iter().enumerate().map(|(index, f)| {
        let name = &f.ident;
        quote_spanned! { f.span() =>
            #name: param.get(#index).ok_or(())?.try_into()?
        }
    });
    let into = fields.named.iter().map(|f| {
        let name = &f.ident;
        quote_spanned! { f.span() =>
            param.#name.into()
        }
    });

    let from = quote! {
        if let passerine::Data::Tuple(param) = param {
            Ok(Point { #(#from,)* })
        } else {
            Err(())
        }
    };
    let into = quote! {
        passerine::Data::Tuple(vec![#(#into,)*])
    };

    (from, into)
}
