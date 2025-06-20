use crate::extract_struct_fields_from_ast;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{parse_quote, ImplGenerics, Type};

#[derive(Clone)]
pub struct FieldInfo<'a> {
    pub name: &'a Option<Ident>,
    pub field_type: Type,
    pub field_target_type: Type,
}

#[derive(Clone)]
pub struct SanitizeValueMacroCustomParameters<'a> {
    pub struct_type: &'a Type,
    pub struct_type_target: &'a Type,
    pub fields: Vec<FieldInfo<'a>>,

    pub impl_generics: ImplGenerics<'a>,
    pub where_clause: Option<&'a syn::WhereClause>,
}

pub fn impl_sanitize_value_custom(params: SanitizeValueMacroCustomParameters) -> TokenStream {
    let SanitizeValueMacroCustomParameters {
        struct_type,
        struct_type_target,
        fields,
        impl_generics,
        where_clause,
    } = params;

    let where_fields = fields.iter().map(|f| {
        let field_type = &f.field_type;
        let new_field_type = &f.field_target_type;
        quote! {
            #field_type: ::untrusted_value::SanitizeValue<#new_field_type, Error = CommonSanitizationError>,
        }
    });
    
    let where_clause = if let Some(where_clause) = where_clause {
        quote! {
            #where_clause #(#where_fields)*
        }
    } else {
        quote! {
            where #(#where_fields)*
        }
    };

    let impl_generics = quote! {
        <#impl_generics CommonSanitizationError>
    };

    let create_struct = {
        #[cfg(not(feature = "harden_sanitize"))]
        {
            let mutate_fields = fields.iter().map(|f| {
                let field_name = f.name;
                quote! {
                    #field_name: self.#field_name.sanitize_value()?,
                }
            });

            quote! {
                Ok(#struct_type_target {
                    #(#mutate_fields)*
                })
            }
        }
        #[cfg(feature = "harden_sanitize")]
        {
            let mutate_fields = fields.iter().map(|f| {
                let field_name = f.name;
                quote! {
                    let #field_name = self.#field_name.sanitize_value();
                }
            });

            let error = fields.iter().map(|f| {
                let field_name = f.name;
                quote! {
                    let #field_name = #field_name?;
                }
            });

            let struct_fields = fields.iter().map(|f| {
                let field_name = f.name;
                quote! {
                    #field_name,
                }
            });

            quote! {
                #(#mutate_fields)*
                #(#error)*

                Ok(#struct_type_target {
                    #(#struct_fields)*
                })
            }
        }
    };

    quote! {
        // STRUCT -> sanitize_value -> TARGET
        #[automatically_derived]
        impl #impl_generics ::untrusted_value::SanitizeValue<#struct_type_target> for #struct_type #where_clause {
            type Error = CommonSanitizationError;
            fn sanitize_value(self) -> Result<#struct_type_target, Self::Error> {
                #create_struct
            }
        }
    }
}

pub fn impl_sanitize_value_macro(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;

    let modified_fields: Vec<FieldInfo> = extract_struct_fields_from_ast(ast)
        .iter()
        .map(|f| {
            let field_name = &f.ident;
            let field_type = &f.ty;
            FieldInfo {
                name: field_name,
                field_target_type: field_type.clone(),
                field_type: field_type.clone(),
            }
        })
        .collect();

    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let name_wrap = parse_quote!(::untrusted_value::UntrustedValue<#name #ty_generics>);
    let name_source = &name_wrap;

    let name = parse_quote!(#name #ty_generics);
    let name_target = &name;

    let parameters = SanitizeValueMacroCustomParameters {
        struct_type: name_source,
        struct_type_target: name_target,
        fields: modified_fields,
        impl_generics,
        where_clause,
    };

    impl_sanitize_value_custom(parameters)
}
