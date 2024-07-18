use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_quote, ItemFn, ReturnType};

pub fn impl_untrusted_output(item: TokenStream) -> TokenStream {
    let input_fn: ItemFn =
        syn::parse2(item).expect("This macro can only be used on function declaration");

    let ItemFn {
        attrs,
        vis,
        mut sig,
        block,
    } = input_fn;

    println!("{}", sig.clone().output.into_token_stream());

    let output = match &sig.output {
        ReturnType::Default => panic!(
            "Can not annotate function with #[untrusted_output] since it has no return value."
        ),
        ReturnType::Type(_, type_box) => {
            let original_type = type_box.as_ref();
            parse_quote! { -> ::untrusted_value::UntrustedValue<#original_type> }
        }
    };

    sig.output = output;

    // Split the function into its header and body
    let function_header = quote! {
        #(#attrs)* #vis #sig
    };

    quote! {
        #function_header {
            ::untrusted_value::UntrustedValue::from(#block)
        }
    }
}
