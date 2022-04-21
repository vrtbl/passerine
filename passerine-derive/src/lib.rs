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

    let result = match input.data {
        syn::Data::Struct(ref data) => match data.fields {
            syn::Fields::Named(ref fields) => {
                // Serialize as a tuple for now...
                let recurse = fields.named.iter().map(|f| {
                    let name = &f.ident;
                    quote_spanned! { f.span() =>
                        self.into()
                    }
                });

                quote! {
                    passerine::Data::Tuple(vec![#(#recurse,)*])
                }
            },
            syn::Fields::Unnamed(_) => todo!(),
            syn::Fields::Unit => todo!(),
        },
        syn::Data::Enum(ref data) => todo!(),
        syn::Data::Union(ref data) => unimplemented!(),
    };

    // // Build the output, possibly using quasi-quotation
    // let expanded = quote! {
    //     // Data -> Item conversion
    //     impl<'a> TryFrom<&'a Data> for #name {
    //         type Error = ();
    //         fn try_from($data: &'a Data) -> Result<Self, ()> { $from }
    //     }

    //     // Item -> Data conversion
    //     impl From<#name> for Data {
    //         fn from($item: $type) -> Self { $into }
    //     }

    //     // With the above two implemented,
    //     // we can implement inject automatically.
    //     impl<'a> Inject<'a> for #name {}
    // };

    // Hand the output tokens back to the compiler
    // TokenStream::from(expanded)
    todo!()
}
