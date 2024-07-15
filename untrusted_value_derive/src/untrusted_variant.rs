use proc_macro::TokenStream;
use quote::quote;
use syn::__private::TokenStream2;
use syn::{Data, Fields, Ident};

fn convert_struct_name_to_untrusted_variant(name: &Ident) -> Ident {
    Ident::new(&format!("{}Untrusted", name), name.span())
}

fn impl_untrusted_variant_of_struct(ast: &syn::DeriveInput) -> TokenStream2 {
    let name = &ast.ident;
    let struct_visibility = &ast.vis;
    let new_struct_name = convert_struct_name_to_untrusted_variant(name);

    let fields = match &ast.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields_named) => &fields_named.named,
            Fields::Unnamed(fields_unnamed) => &fields_unnamed.unnamed,
            Fields::Unit => panic!("Unit structs are not supported"),
        },
        _ => panic!("Only structs are supported"),
    };

    let modified_fields = fields.iter().map(|f| {
        let field_name = &f.ident;
        let field_type = &f.ty;
        let visibility = &f.vis;
        quote! {
            #visibility #field_name: untrusted_value::UntrustedValue<#field_type>,
        }
    });

    quote! {
        #struct_visibility struct #new_struct_name {
            #(#modified_fields)*
        }
    }
}

pub fn impl_untrusted_variant(ast: &syn::DeriveInput) -> TokenStream {
    let name = &ast.ident;
    let new_struct_name = convert_struct_name_to_untrusted_variant(name);

    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    let copy_fields = match &ast.data {
        Data::Struct(data_struct) => match &data_struct.fields {
            Fields::Named(fields_named) => {
                let field_names = fields_named.named.iter().map(|f| &f.ident);
                quote! {
                    #(
                        #field_names: untrusted_value::UntrustedValue::from(self.#field_names),
                    )*
                }
            }
            Fields::Unnamed(fields_unnamed) => {
                let indices = 0..fields_unnamed.unnamed.len();
                quote! {
                    #(
                        untrusted_value::UntrustedValue::from(self.#indices),
                    )*
                }
            }
            Fields::Unit => quote! {},
        },
        _ => panic!("Only structs are supported"),
    };

    let untrusted_struct = impl_untrusted_variant_of_struct(ast);

    let gen_implementation = quote! {
        impl #impl_generics untrusted_value::IntoUntrustedVariant<#new_struct_name #ty_generics, #name #ty_generics> for #name #ty_generics #where_clause {
            fn to_untrusted_variant(self) -> #new_struct_name #ty_generics {
                #new_struct_name {
                    #copy_fields
                }
            }
        }

        impl #impl_generics From<#name #ty_generics> for #new_struct_name #ty_generics #where_clause {
            fn from(value: #name #ty_generics) -> Self {
                value.to_untrusted_variant()
            }
        }

        impl #impl_generics untrusted_value::SanitizeWith<#new_struct_name #ty_generics, #name #ty_generics> for #new_struct_name #ty_generics #where_clause {
            fn sanitize_with<Sanitizer, Error>(self, sanitizer: Sanitizer) -> Result<#name #ty_generics, Error>
            where
                Sanitizer: FnOnce(Self) -> Result<#name #ty_generics, Error>
            {
                sanitizer(self)
            }
        }

        #untrusted_struct
    };

    gen_implementation.into()
}
