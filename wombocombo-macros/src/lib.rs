use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, Data, DeriveInput, Fields};

macro_rules! err {
    ($key:expr, $err:expr) => {{
        let error = syn::Error::new($key.ident.span(), $err);
        
        return Into::<TokenStream>::into(error.to_compile_error());
    }};
}

#[proc_macro_derive(AllValues)]
/// Generates all combinations of a struct or enum.
pub fn derive_all_values(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;

    let expanded = match &input.data {
        // --- CASE 1: STRUCT HANDLING (Cartesian Product via itertools) ---
        Data::Struct(data_struct) => {
            let (field_iter, struct_type) = match &data_struct.fields {
                Fields::Named(fields) => (fields.named.iter(), quote! { @fields }),
                Fields::Unnamed(fields) => (fields.unnamed.iter(), quote! { @tuple }),

                Fields::Unit => err!(
                    input,
                    "AllValues struct derivation only supports structs with fields"
                ),
            };

            let (field_names, field_types): (Vec<_>, Vec<_>) = field_iter
                .enumerate()
                .map(|(i, field)| {
                    (
                        field.ident.clone().unwrap_or_else(|| format_ident!("p{i}")),
                        &field.ty,
                    )
                })
                .unzip();

            quote! {
                ::wombocombo::__private::generate_all_combinations!(#struct_type #name, #(#field_names => #field_types),*)
            }
        }

        // --- CASE 2: ENUM HANDLING (Variant Gathering) ---
        Data::Enum(data_enum) => {
            let mut variant_paths = Vec::new();

            for variant in &data_enum.variants {
                if !matches!(variant.fields, Fields::Unit) {
                    err!(variant, "AllValues enum derivation only supports simple variants without inner data.");
                }

                let variant_ident = &variant.ident;
                variant_paths.push(quote! { #variant_ident });
            }

            // [].into_iter() builds an ExactSizeIterator with an accurate size_hint out of the box.
            quote! {
                ::wombocombo::__private::generate_all_combinations!(@enum #name, #(#variant_paths),*)
            }
        }

        _ => err!(input, "AllValues can only be derived on Structs and Enums"),
    };

    let output = quote! {
         ::wombocombo::__private::define_values_iter!(#name, #expanded);
    };

    TokenStream::from(output)
}
