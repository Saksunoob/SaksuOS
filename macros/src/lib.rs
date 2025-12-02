use proc_macro::TokenStream;
use quote::quote;
use syn::{LitStr};

#[proc_macro]
pub fn uefistr(input: TokenStream) -> TokenStream {
    let input = syn::parse_macro_input!(input as LitStr);
    let s = input.value();
    let len = s.encode_utf16().count()+1;

    let expanded = s.encode_utf16().map(|c| {
        quote! {#c, }
    });

    let stream = quote! {{
        static LITERAL: [u16; #len] = [#(#expanded)*0u16];
        LITERAL.as_ptr() as *mut u16
    }};

    TokenStream::from(stream)
}