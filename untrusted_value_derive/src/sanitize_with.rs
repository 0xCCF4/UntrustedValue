use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::Generics;

pub fn impl_sanitize_with_custom(
    source_name: &Ident,
    source_types: &Generics,
    target_name: &Ident,
    target_types: &Generics,
) -> TokenStream {
    let (source_impl_generics, source_ty_generics, source_where_clause) =
        source_types.split_for_impl();
    let (target_impl_generics, target_ty_generics, target_where_clause) =
        target_types.split_for_impl();

    let mut source_impl_generics = quote! {
        #source_impl_generics
    };

    if !source_impl_generics.is_empty() {
        source_impl_generics = quote! {
            #source_impl_generics,
        };
    }

    let mut source_where_clause = quote! {
        #source_where_clause
    };

    if !source_where_clause.is_empty() {
        source_where_clause = quote! {
            #source_where_clause,
        };
    }

    let impl_generics = quote! {
        <#source_impl_generics #target_impl_generics>
    };

    let mut where_clause = quote! {
        #source_where_clause #target_where_clause
    };

    if !where_clause.is_empty() {
        where_clause = quote! {
            where #where_clause
        };
    }

    quote! {
        // SOURCE -> sanitize_with -> TARGET
        #[automatically_derived]
        impl #impl_generics ::untrusted_value::SanitizeWith<#source_name #source_ty_generics, #target_name #target_ty_generics> for #source_name #source_ty_generics #where_clause {
            fn sanitize_with<Sanitizer, Error>(self, sanitizer: Sanitizer) -> Result<#target_name #target_ty_generics, Error>
            where
                Sanitizer: FnOnce(Self) -> Result<#target_name, Error>
            {
                sanitizer(self)
            }
        }
    }
}
