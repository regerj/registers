use darling::{FromDeriveInput, FromField};
use quote::quote;
use syn::{DeriveInput, ItemStruct, parse_macro_input};

use crate::{register::Register, register32::Register32};

mod common;
mod register;
mod register32;

#[derive(Debug, darling::FromMeta)]
#[darling(derive_syn_parse)]
struct MacroArgs {
    size: usize,
}

#[derive(darling::FromDeriveInput)]
#[darling(attributes(register), forward_attrs(allow, doc, cfg))]
struct RegisterOpts {
    //ident: syn::Ident,
    //attrs: Vec<syn::Attribute>,
    size: usize,
}

//#[proc_macro_derive(Register, attributes(register, field))]
//pub fn register(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
//    let derive_input = parse_macro_input!(input as DeriveInput);
//    let struct_input = match &derive_input.data {
//        syn::Data::Struct(s) => s,
//        _ => return quote! { compile_error!("Register cannot be derived on anything other than a struct!") }.into(),
//    };
//    let l0_opts = RegisterOpts::from_derive_input(&derive_input).unwrap();
//    let field_opts: Vec<_> = struct_input
//        .fields
//        .iter()
//        .map(|f| common::Field::from_field(f).unwrap())
//        .collect();
//
//    assert_eq!(l0_opts.size, 32);
//    assert_eq!(field_opts[0].msb, 15);
//    assert_eq!(field_opts[1].msb, 31);
//    quote! {}.into()
//}

//#[proc_macro_derive(Register32, attributes(field))]
//pub fn register_32(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
//    let derive_input = parse_macro_input!(input as DeriveInput);
//    let struct_input = match &derive_input.data {
//        syn::Data::Struct(s) => s,
//        _ => return quote! { compile_error!("Register cannot be derived on anything other than a struct!") }.into(),
//    };
//
//    let reg = Register32::from(struct_input);
//
//    quote! {}.into()
//}

#[proc_macro_attribute]
pub fn register(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let strct = parse_macro_input!(item as ItemStruct);
    let reg = Register::new(strct);
    reg.implement().into()
}

#[cfg(test)]
mod tests {
    //use super::*;
}
