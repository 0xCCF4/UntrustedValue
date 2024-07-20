use crate::extract_struct_fields_from_ast;
use crate::sanitize_value::{
    impl_sanitize_value_custom, FieldInfo, SanitizeValueMacroCustomParameters,
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::Parse;
use syn::{parse2, Data, Fields, Ident, Meta, Token};

#[derive(Default)]
struct Parameters {
    derive_macros: Vec<syn::Ident>,
}

impl Parse for Parameters {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut derive_macros = Vec::new();

        while !input.is_empty() {
            // Parse the trait name as an Ident
            let ident: Ident = input.parse()?;
            derive_macros.push(ident);

            // Consume an optional comma
            let _ = input.parse::<Token![,]>().ok();
        }

        Ok(Parameters { derive_macros })
    }
}

fn convert_struct_name_to_untrusted_variant(name: &Ident) -> Ident {
    Ident::new(&format!("{name}Untrusted"), name.span())
}

#[allow(clippy::too_many_lines)] // need to refactor this in the future
fn impl_untrusted_variant_of_struct(
    parameters: &Parameters,
    ast: &syn::DeriveInput,
) -> TokenStream {
    let name = &ast.ident;
    let struct_visibility = &ast.vis;
    let new_struct_name = convert_struct_name_to_untrusted_variant(name);

    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let modified_fields = extract_struct_fields_from_ast(ast).iter().map(|f| {
        let field_name = &f.ident;
        let field_type = &f.ty;
        let visibility = &f.vis;
        quote! {
            #visibility #field_name: ::untrusted_value::UntrustedValue<#field_type>,
        }
    });

    let fields: Vec<FieldInfo> = extract_struct_fields_from_ast(ast)
        .iter()
        .map(|f| {
            let field_name = &f.ident;
            let field_type = &f.ty;
            let new_type = syn::parse_quote!(untrusted_value::UntrustedValue<#field_type>);
            FieldInfo {
                name: field_name,
                field_type: new_type,
                field_target_type: field_type.clone(),
            }
        })
        .collect();

    let new_struct_type = syn::parse_quote!(#new_struct_name #ty_generics);
    let struct_type = syn::parse_quote!(#name #ty_generics);
    let params = SanitizeValueMacroCustomParameters {
        struct_type: &new_struct_type,
        struct_type_target: &struct_type,
        fields,
        impl_generics,
        where_clause,
    };

    let sanitize_value_derive = parameters
        .derive_macros
        .iter()
        .any(|d| d == "SanitizeValue");
    let sanitize_value_derive = if sanitize_value_derive {
        let derive = impl_sanitize_value_custom(params);

        let where_clause_with_error_bound = {
            let prefix = if where_clause.is_none() {
                quote! { where }
            } else {
                quote! { #where_clause, }
            };
            quote! {
                #prefix #new_struct_name #ty_generics: ::untrusted_value::SanitizeValue<#name #ty_generics, Error = CommonSanitizationError>
            }
        };

        quote! {
            // UNTRUSTED STRUCT -> sanitize_value -> STRUCT
            #derive

            // UntrustedValue<STRUCT> -> sanitize_value -> STRUCT
            //  by STRUCT -> into_untrusted_variant -> UNTRUSTED STRUCT -> sanitize_value -> STRUCT
            #[automatically_derived]
            impl<CommonSanitizationError> ::untrusted_value::SanitizeValue<#name #ty_generics> for ::untrusted_value::UntrustedValue<#name #ty_generics> #where_clause_with_error_bound {
                type Error = CommonSanitizationError;
                fn sanitize_value(self) -> Result<#name #ty_generics, Self::Error> {
                    self.use_untrusted_value().to_untrusted_variant().sanitize_value()
                }
            }
        }
    } else {
        quote! {}
    };

    let sanitize_value_end_derive = parameters
        .derive_macros
        .iter()
        .any(|d| d == "SanitizeValueEnd");
    let sanitize_value_end_derive = if sanitize_value_end_derive {
        quote! {
            #[automatically_derived]
            impl<CommonSanitizationError> ::untrusted_value::SanitizeValue<#name> for ::untrusted_value::UntrustedValue<#name>
            where
                #name: ::untrusted_value::IntoUntrustedVariant<#new_struct_name>,
                #new_struct_name: ::untrusted_value::SanitizeValue<#name, Error=CommonSanitizationError>
            {
                type Error = CommonSanitizationError;
                fn sanitize_value(self) -> std::result::Result<#name, Self::Error> {
                    self.use_untrusted_value().to_untrusted_variant().sanitize_value()
                }
            }
        }
    } else {
        quote! {}
    };

    assert!(
        sanitize_value_end_derive.is_empty() || sanitize_value_derive.is_empty(),
        "SanitizeValueEnd derive can not be used together with SanitizeValue derive"
    );

    let derive_macros = parameters.derive_macros.iter().map(|d| {
        if d == "SanitizeValue" || d == "SanitizeValueEnd" {
            quote! {}
        } else {
            quote! {
                #[derive(#d)]
            }
        }
    });

    quote! {
        #[automatically_derived]
        #(#derive_macros)*
        #struct_visibility struct #new_struct_name #ty_generics #where_clause {
            #(#modified_fields)*
        }

        // UNTRUSTED STRUCT -> sanitize_value -> STRUCT
        // UntrustedValue<STRUCT> -> sanitize_value -> STRUCT
        #sanitize_value_derive

        // UntrustedValue<STRUCT> -> sanitize_value -> STRUCT
        #sanitize_value_end_derive
    }
}

pub fn impl_untrusted_variant_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let new_struct_name = convert_struct_name_to_untrusted_variant(name);

    let parameter = ast
        .attrs
        .iter()
        .find(|a| a.path().segments.len() == 1 && a.path().segments[0].ident == "untrusted_derive")
        .map_or_else(Parameters::default, |attribute| match attribute.meta {
            Meta::List(ref meta) => parse2::<Parameters>(meta.tokens.clone())
                .expect("Expected a list of traits to derive within #[untrusted_derive(...)]"),
            _ => Parameters::default(),
        });

    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let fields_wrap_into_untrusted = match &ast.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields_named) => {
                let field_names = fields_named.named.iter().map(|f| &f.ident);
                quote! {
                    #(
                        #field_names: ::untrusted_value::UntrustedValue::from(self.#field_names),
                    )*
                }
            }
            Fields::Unnamed(fields_unnamed) => {
                let indices = 0..fields_unnamed.unnamed.len();
                quote! {
                    #(
                        ::untrusted_value::UntrustedValue::from(self.#indices),
                    )*
                }
            }
            Fields::Unit => quote! {},
        },
        _ => panic!("Only structs are supported"),
    };

    let fields_wrap_from_untrusted = match &ast.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields_named) => {
                let field_names = fields_named.named.iter().map(|f| &f.ident);
                quote! {
                    #(
                        #field_names: self.#field_names.use_untrusted_value(),
                    )*
                }
            }
            Fields::Unnamed(fields_unnamed) => {
                let indices = 0..fields_unnamed.unnamed.len();
                quote! {
                    #(
                        self.#indices.use_untrusted_value(),
                    )*
                }
            }
            Fields::Unit => quote! {},
        },
        _ => panic!("Only structs are supported"),
    };

    let untrusted_struct = impl_untrusted_variant_of_struct(&parameter, ast);

    let sanitize_with = super::sanitize_with::impl_sanitize_with_custom(
        &new_struct_name,
        &ast.generics,
        name,
        &ast.generics,
    );

    quote! {
        // STRUCT -> into_untrusted_variant -> UNTRUSTED STRUCT
        #[automatically_derived]
        impl #impl_generics ::untrusted_value::IntoUntrustedVariant<#new_struct_name #ty_generics> for #name #ty_generics #where_clause {
            fn to_untrusted_variant(self) -> #new_struct_name #ty_generics {
                #new_struct_name {
                    #fields_wrap_into_untrusted
                }
            }
        }

        // UNTRUSTED STRUCT -> into_untrusted_variant -> UntrustedValue<STRUCT>
        #[automatically_derived]
        impl #impl_generics ::untrusted_value::IntoUntrustedVariant<::untrusted_value::UntrustedValue<#name #ty_generics>> for #new_struct_name #ty_generics #where_clause {
            fn to_untrusted_variant(self) -> ::untrusted_value::UntrustedValue<#name #ty_generics> {
                ::untrusted_value::UntrustedValue::from(
                    #name {
                        #fields_wrap_from_untrusted
                    }
                )
            }
        }

        // UntrustedValue<STRUCT> -> into_untrusted_variant -> UNTRUSTED STRUCT
        #[automatically_derived]
        impl #impl_generics ::untrusted_value::IntoUntrustedVariant<#new_struct_name #ty_generics> for ::untrusted_value::UntrustedValue<#name #ty_generics> #where_clause {
            fn to_untrusted_variant(self) -> #new_struct_name #ty_generics {
                self.use_untrusted_value().to_untrusted_variant()
            }
        }

        // STRUCT -> into -> UNTRUSTED STRUCT
        #[automatically_derived]
        impl #impl_generics From<#name #ty_generics> for #new_struct_name #ty_generics #where_clause {
            fn from(value: #name #ty_generics) -> Self {
                value.to_untrusted_variant()
            }
        }

        // UNTRUSTED STRUCT -> sanitize_with -> STRUCT
        #sanitize_with

        // UNTRUSTED STRUCT
        // SanitizeValueDerive: UNTRUSTED STRUCT -> sanitize_value -> STRUCT
        // SanitizeValueDerive: UntrustedValue<STRUCT> -> sanitize_value -> STRUCT
        #untrusted_struct
    }
}
