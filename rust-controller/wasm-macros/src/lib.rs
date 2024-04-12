extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn wasm_async(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    // Construct the new function
    let fn_name = &input_fn.sig.ident;
    let fn_body = &input_fn.block;
    let inputs = &input_fn.sig.inputs;
    let output = &input_fn.sig.output;

    // Generate the new function with wasm_bindgen and Promise
    let expanded = quote! {
        #[wasm_bindgen]
        pub fn #fn_name(#inputs) -> wasm_bindgen::prelude::Promise {
            let future = async move #fn_body;
            wasm_bindgen_futures::future_to_promise(future)
        }
    };

    TokenStream::from(expanded)
}
