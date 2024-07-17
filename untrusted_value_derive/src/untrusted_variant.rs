use crate::extract_struct_fields_from_ast;
use crate::sanitize_value::{impl_sanitize_value_custom, FieldInfo, SanitizeValueCustomParameters};
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
    Ident::new(&format!("{}Untrusted", name), name.span())
}

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
                field_name,
                field_type: new_type,
                field_target_type: field_type.clone(),
            }
        })
        .collect();

    let params = SanitizeValueCustomParameters {
        struct_type: &new_struct_name,
        struct_type_target: name,
        fields,
        impl_generics,
        ty_generics: ty_generics.clone(),
        where_clause,
    };

    let sanitize_value_derive = parameters
        .derive_macros
        .iter()
        .any(|d| d == "SanitizeValue");
    let sanitize_value_derive = if sanitize_value_derive {
        impl_sanitize_value_custom(params)
    } else {
        quote! {}
    };

    let derive_macros = parameters.derive_macros.iter().map(|d| {
        if d == "SanitizeValue" {
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

        #sanitize_value_derive
    }
}

pub fn impl_untrusted_variant(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let new_struct_name = convert_struct_name_to_untrusted_variant(name);

    let parameter = ast
        .attrs
        .iter()
        .find(|a| a.path().segments.len() == 1 && a.path().segments[0].ident == "untrusted_derive")
        .map(|attribute| match attribute.meta {
            Meta::List(ref meta) => parse2::<Parameters>(meta.tokens.clone())
                .expect("Expected a list of traits to derive within #[untrusted_derive(...)]"),
            _ => Parameters::default(),
        })
        .unwrap_or_else(Parameters::default);

    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let copy_fields = match &ast.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields_named) => {
                let field_names = fields_named.named.iter().map(|f| &f.ident);
                quote! {
                    #(
                        #field_names: ::untrusted_value::UntrustedValue::from(copy.#field_names),
                    )*
                }
            }
            Fields::Unnamed(fields_unnamed) => {
                let indices = 0..fields_unnamed.unnamed.len();
                quote! {
                    #(
                        ::untrusted_value::UntrustedValue::from(copy.#indices),
                    )*
                }
            }
            Fields::Unit => quote! {},
        },
        _ => panic!("Only structs are supported"),
    };

    let untrusted_struct = impl_untrusted_variant_of_struct(&parameter, ast);

    quote! {
        #[automatically_derived]
        impl #impl_generics ::untrusted_value::IntoUntrustedVariant<#new_struct_name #ty_generics, #name #ty_generics> for #name #ty_generics #where_clause {
            fn to_untrusted_variant(self) -> #new_struct_name #ty_generics {
                let copy = self;
                #new_struct_name {
                    #copy_fields
                }
            }
        }

        #[automatically_derived]
        impl #impl_generics ::untrusted_value::IntoUntrustedVariant<#new_struct_name #ty_generics, #name #ty_generics> for ::untrusted_value::UntrustedValue<#name #ty_generics> #where_clause {
            fn to_untrusted_variant(self) -> #new_struct_name #ty_generics {
                fn no_sanitize<T>(value: T) -> Result<T, ()> {
                    Ok(value)
                }
                let copy = self.sanitize_with(no_sanitize).unwrap();
                #new_struct_name {
                    #copy_fields
                }
            }
        }

        #[automatically_derived]
        impl #impl_generics From<#name #ty_generics> for #new_struct_name #ty_generics #where_clause {
            fn from(value: #name #ty_generics) -> Self {
                value.to_untrusted_variant()
            }
        }

        #[automatically_derived]
        impl #impl_generics ::untrusted_value::SanitizeWith<#new_struct_name #ty_generics, #name #ty_generics> for #new_struct_name #ty_generics #where_clause {
            fn sanitize_with<Sanitizer, Error>(self, sanitizer: Sanitizer) -> Result<#name #ty_generics, Error>
            where
                Sanitizer: FnOnce(Self) -> Result<#name #ty_generics, Error>
            {
                sanitizer(self)
            }
        }

        #untrusted_struct
    }
}
