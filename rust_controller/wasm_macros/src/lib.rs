extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};


#[proc_macro_attribute]
pub fn wasm_async(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);

    let fn_name = &input_fn.sig.ident;
    let fn_body = &input_fn.block;
    let inputs = &input_fn.sig.inputs;

    let expanded = quote! {
        #[wasm_bindgen]
        pub fn #fn_name(#inputs) -> js_sys::Promise {
            let future = async move #fn_body;
            wasm_bindgen_futures::future_to_promise(future)
        }
    };

    TokenStream::from(expanded)
}
