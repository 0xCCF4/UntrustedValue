use crate::extract_struct_fields_from_ast;
use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{ImplGenerics, Type, TypeGenerics};

#[derive(Clone)]
pub struct FieldInfo<'a> {
    pub field_name: &'a Option<Ident>,
    pub field_type: Type,
    pub field_target_type: Type,
}

#[derive(Clone)]
pub struct SanitizeValueCustomParameters<'a> {
    pub struct_type: &'a Ident,
    pub struct_type_target: &'a Ident,
    pub fields: Vec<FieldInfo<'a>>,

    pub impl_generics: ImplGenerics<'a>,
    pub ty_generics: TypeGenerics<'a>,
    pub where_clause: Option<&'a syn::WhereClause>,
}

pub fn impl_sanitize_value_custom(params: SanitizeValueCustomParameters) -> TokenStream {
    let SanitizeValueCustomParameters {
        struct_type,
        struct_type_target,
        fields,
        impl_generics,
        ty_generics,
        where_clause,
    } = params;

    let where_fields = fields.iter().map(|f| {
        let field_type = &f.field_type;
        let new_field_type = &f.field_target_type;
        quote! {
            #field_type: untrusted_value::SanitizeValue<#new_field_type, Error = E_UNTRUSTED>,
        }
    });

    let where_clause = if where_clause.is_none() {
        quote! {
            where #(#where_fields)*
        }
    } else {
        let where_clause = where_clause.unwrap();
        quote! {
            #where_clause #(#where_fields)*
        }
    };

    let impl_generics = quote! {
        <#impl_generics E_UNTRUSTED>
    };

    let create_struct = {
        #[cfg(not(feature = "harden_sanitize"))]
        {
            let mutate_fields = fields.iter().map(|f| {
                let field_name = f.field_name;
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
                let field_name = f.field_name;
                quote! {
                    let #field_name = self.#field_name.sanitize_value(),
                }
            });

            let error = fields.iter().map(|f| {
                let field_name = f.field_name;
                quote! {
                    let #field_name = #field_name?;
                }
            });

            let struct_fields = fields.iter().map(|f| {
                let field_name = f.field_name;
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
        #[automatically_derived]
        impl #impl_generics untrusted_value::SanitizeValue<#struct_type_target #ty_generics> for #struct_type #ty_generics #where_clause {
            type Error = E_UNTRUSTED;
            fn sanitize_value(self) -> Result<#struct_type_target #ty_generics, Self::Error> {
                #create_struct
            }
        }
    }
}

pub fn impl_sanitize_value(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;

    let modified_fields: Vec<FieldInfo> = extract_struct_fields_from_ast(ast)
        .iter()
        .map(|f| {
            let field_name = &f.ident;
            let field_type = &f.ty;
            FieldInfo {
                field_name,
                field_target_type: field_type.clone(),
                field_type: field_type.clone(),
            }
        })
        .collect();

    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let parameters = SanitizeValueCustomParameters {
        struct_type: name,
        struct_type_target: name,
        fields: modified_fields,
        impl_generics,
        ty_generics,
        where_clause,
    };

    impl_sanitize_value_custom(parameters)
}
