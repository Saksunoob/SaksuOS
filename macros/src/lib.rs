use proc_macro::TokenStream;
use std::fs;
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

fn split(line: &str) -> (&str, Vec<i64>) {
    let (name, values) = line.split_once(' ').expect("failed to split line");

    (name, values.split(' ').map(|v| v.parse::<i64>().expect("failed to parse i32")).collect())
}

fn get(name: &str, line: &str) -> Vec<i64> {
    let (got_name, values) = line.split_once(' ').expect("failed to split line");
    if got_name != name {
        panic!("{} should have name", name);
    }
    values.split(' ').map(|v| v.parse::<i64>().expect("failed to parse i32")).collect()
}

fn expect(name: &str, line: &str) {
    let (got_name, _) = line.split_once(' ').unwrap_or_else(
        || panic!("failed to split line {}", line)
    );
    if got_name != name {
        panic!("{} should have name", name);
    }
}

#[proc_macro]
pub fn bdf_font(input: TokenStream) -> TokenStream {
    let path = syn::parse_macro_input!(input as LitStr);
    let path = path.value();

    let content = fs::read_to_string(&path).expect("Failed to read file");

    let mut lines = content.lines();
    if lines.next().unwrap() != "STARTFONT 2.1" {
        panic!("Unrecognized font file");
    }

    expect("FONT", lines.next().unwrap());

    expect("SIZE", lines.next().unwrap());
    let bb = get("FONTBOUNDINGBOX", lines.next().unwrap());
    let width = bb[0];
    let height = bb[1];
    let x = bb[2];
    let y = bb[3];

    let p_count = get("STARTPROPERTIES", lines.next().unwrap());
    let mut lines = lines.skip(p_count[0] as usize + 1);

    let c_count = get("CHARS", lines.next().unwrap())[0];

    let mut chars = Vec::with_capacity(c_count as usize);
    for _ in 0..c_count {
        expect("STARTCHAR", lines.next().unwrap());

        let encoding: u32 = get("ENCODING", lines.next().unwrap())[0] as u32;
        expect("SWIDTH", lines.next().unwrap());
        let width = get("DWIDTH", lines.next().unwrap())[0];
        if width != bb[0] {
            panic!("Font is not monospace")
        }

        let bb = get("BBX", lines.next().unwrap());
        let width = bb[0];
        let height = bb[1];
        let x = bb[2];
        let y = bb[3];

        if lines.next().unwrap() != "BITMAP" {
            panic!("Expected bitmap")
        }

        let mut bitmap_data = Vec::new();

        for _ in 0..height as usize {
            let line = lines.next().unwrap();
            let value = u32::from_str_radix(line, 16).expect("failed to parse u32");

            bitmap_data.push(quote! {
                #value
            });
        }

        if lines.next().unwrap() != "ENDCHAR" {
            panic!("Expected char end")
        }

        chars.push(quote! {
            Char(#encoding, BoundingBox(#width, #height, #x, #y), &[#(#bitmap_data),*])
        });
    }

    let expanded = quote! {
        Font(
            BoundingBox(#width, #height, #x, #y),
            &[
                #(#chars),*
            ]
        )
    };

    TokenStream::from(expanded)
}