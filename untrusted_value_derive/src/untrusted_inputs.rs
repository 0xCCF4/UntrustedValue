use proc_macro2::TokenStream;
use quote::quote;
use syn::FnArg::{Receiver, Typed};
use syn::ItemFn;

pub fn impl_untrusted_inputs(item: TokenStream) -> TokenStream {
    let input_fn: ItemFn =
        syn::parse2(item).expect("This macro can only be used on function declaration");

    let ItemFn {
        attrs,
        vis,
        sig,
        block,
    } = input_fn;

    // Split the function into its header and body
    let function_header = quote! {
        #(#attrs)* #vis #sig
    };

    let mapped_inputs = sig
        .inputs
        .iter()
        .filter_map(|arg| match arg {
            Receiver(_) => None,
            Typed(named_arg) => Some(named_arg),
        })
        .map(|arg| {
            let arg = &arg.pat;
            quote! {
                let #arg = untrusted_value::UntrustedValue::from(#arg);
            }
        });

    quote! {
        #function_header {
            #(#mapped_inputs)*

            #block
        }
    }
}
