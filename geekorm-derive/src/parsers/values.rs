use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;

// impl From<&UserType> for geekorm::Value {
//     fn from(value: &UserType) -> Self {
//         match value {
//             UserType::Admin => geekorm::Value::Integer(0),
//             UserType::User => geekorm::Value::Integer(1),
//         }
//     }
// }
pub(crate) fn generate_from_value(
    ident: &syn::Ident,
    variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>,
    generics: &syn::Generics,
) -> Result<TokenStream, syn::Error> {
    let (impl_generics, ty_generics, _where_clause) = generics.split_for_impl();

    let mut stream = TokenStream::new();
    let mut from_value_stream = TokenStream::new();

    for variant in variants {
        if !matches!(variant.fields, syn::Fields::Unit) {
            return Err(syn::Error::new(
                variant.span(),
                "Only unit variants are supported",
            ));
        }
        if variant.discriminant.is_some() {
            return Err(syn::Error::new(
                variant.span(),
                "Discriminant values are not supported",
            ));
        }

        let variant_ident = variant.ident.clone();
        // TODO: Handle r# prefix better
        let variant_string = variant_ident.to_string().replace("r#", "");
        let variant_str = syn::LitStr::new(&variant_string, variant.span());

        stream.extend(quote! {
            #ident::#variant_ident => ::geekorm::Value::Text(String::from(#variant_str)),
        });
        from_value_stream.extend(quote! {
            ::geekorm::Value::Text(ref s) if s == #variant_str => #ident::#variant_ident,
        });
    }

    Ok(quote! {
        #[automatically_derived]
        impl #impl_generics From<#ident #ty_generics> for geekorm::Value {
            fn from(value: #ident #ty_generics) -> Self {
                match value {
                    #stream
                    _ => panic!("Unknown value"),
                }
            }
        }

        #[automatically_derived]
        impl #impl_generics From<&#ident #ty_generics> for geekorm::Value {
            fn from(value: &#ident #ty_generics) -> Self {
                match value {
                    #stream
                    _ => panic!("Unknown value"),
                }
            }
        }

        #[automatically_derived]
        impl #impl_generics From<geekorm::Value> for #ident #ty_generics {
            fn from(value: geekorm::Value) -> Self {
                match value {
                    #from_value_stream
                    _ => panic!("Unknown value"),
                }
            }
        }
    })
}

pub(crate) fn generate_serde(
    ident: &syn::Ident,
    variants: &syn::punctuated::Punctuated<syn::Variant, syn::token::Comma>,
    generics: &syn::Generics,
) -> Result<TokenStream, syn::Error> {
    let (_impl_generics, _ty_generics, _where_clause) = generics.split_for_impl();

    let mut stream = TokenStream::new();

    stream.extend(quote! {
        impl ::serde::Serialize for #ident {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: ::serde::Serializer,
            {
                ::geekorm::Value::from(self).serialize(serializer)
            }
        }
    });

    let mut tokens = TokenStream::new();

    for variant in variants.iter() {
        if !matches!(variant.fields, syn::Fields::Unit) {
            return Err(syn::Error::new(
                variant.span(),
                "Only unit variants are supported",
            ));
        }
        if variant.discriminant.is_some() {
            return Err(syn::Error::new(
                variant.span(),
                "Discriminant values are not supported",
            ));
        }

        let variant_ident = variant.ident.clone();
        let variant_string = variant_ident.to_string().replace("r#", "");
        let variant_str = syn::LitStr::new(&variant_string, variant.span());

        tokens.extend(quote! {
            #variant_str => Ok(#ident::#variant_ident),
        });
    }

    stream.extend(quote! {
        impl<'de> ::serde::Deserialize<'de> for #ident {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: ::serde::Deserializer<'de>,
            {

                match String::deserialize(deserializer)?.as_str() {
                    #tokens
                    _ => Err(serde::de::Error::custom("Unknown user type")),
                }
            }
        }
    });

    Ok(stream)
}
